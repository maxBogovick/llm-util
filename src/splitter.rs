use crate::{
    config::Config,
    error::{Error, Result},
    file::{FileContent, FileData},
    token::TokenEstimator,
};
use std::sync::Arc;
use tracing::{debug, trace, warn};

/// Represents a chunk of files with associated metadata.
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Sequential chunk index (0-based)
    pub index: usize,

    /// Files included in this chunk
    pub files: Vec<FileData>,

    /// Total token count across all files
    pub total_tokens: usize,
}

impl Chunk {
    /// Creates a new chunk.
    #[must_use]
    pub fn new(index: usize, files: Vec<FileData>, total_tokens: usize) -> Self {
        Self {
            index,
            files,
            total_tokens,
        }
    }

    /// Returns the number of files in this chunk.
    #[must_use]
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Returns true if this chunk is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Returns the utilization percentage (0.0 to 1.0).
    #[must_use]
    pub fn utilization(&self, max_tokens: usize) -> f64 {
        if max_tokens == 0 {
            return 0.0;
        }
        self.total_tokens as f64 / max_tokens as f64
    }
}

/// Splits files into optimally-sized chunks based on token limits.
pub struct Splitter {
    max_chunk_tokens: usize,
    overlap_tokens: usize,
    prefer_line_boundaries: bool,
    tokenizer: Arc<dyn TokenEstimator>,
}

impl Splitter {
    /// Creates a new splitter from configuration.
    pub fn new(config: &Config) -> Self {
        Self {
            max_chunk_tokens: config.effective_chunk_size(),
            overlap_tokens: config.overlap_tokens,
            prefer_line_boundaries: config.prefer_line_boundaries,
            tokenizer: config.tokenizer.create(),
        }
    }

    /// Splits files into chunks respecting token limits.
    ///
    /// # Algorithm
    ///
    /// 1. Files that fit within limits are grouped together
    /// 2. Large files are split across multiple chunks with overlap
    /// 3. Chunks are optimized to maximize token utilization
    ///
    /// # Errors
    ///
    /// Returns an error if a binary file exceeds token limits.
    pub fn split(&self, files: Vec<FileData>) -> Result<Vec<Chunk>> {
        if files.is_empty() {
            return Ok(Vec::new());
        }

        let mut chunks = Vec::new();
        let mut current_builder = ChunkBuilder::new(0, self.max_chunk_tokens);

        for file in files {
            self.process_file(file, &mut current_builder, &mut chunks)?;
        }

        // Finalize last chunk
        if let Some(chunk) = current_builder.build() {
            chunks.push(chunk);
        }

        self.log_split_results(&chunks);

        Ok(chunks)
    }

    /// Processes a single file, adding it to chunks.
    fn process_file(
        &self,
        file: FileData,
        current_builder: &mut ChunkBuilder,
        chunks: &mut Vec<Chunk>,
    ) -> Result<()> {
        // File fits completely within limits
        if file.token_count <= self.max_chunk_tokens {
            if current_builder.can_fit(file.token_count) {
                current_builder.add_file(file);
                Ok(())
            } else {
                // Finalize current chunk and start new one
                let old_builder = std::mem::replace(
                    current_builder,
                    ChunkBuilder::new(chunks.len(), self.max_chunk_tokens)
                );

                if let Some(chunk) = old_builder.build() {
                    chunks.push(chunk);
                }

                current_builder.add_file(file);
                Ok(())
            }
        } else {
            // File too large - needs splitting
            self.handle_large_file(file, current_builder, chunks)
        }
    }

    /// Handles files that exceed chunk size limits.
    fn handle_large_file(
        &self,
        file: FileData,
        current_builder: &mut ChunkBuilder,
        chunks: &mut Vec<Chunk>,
    ) -> Result<()> {
        debug!(
            "File '{}' exceeds limit ({} tokens), splitting into parts",
            file.relative_path, file.token_count
        );

        // Finalize current chunk
        let old_builder = std::mem::replace(
            current_builder,
            ChunkBuilder::new(chunks.len(), self.max_chunk_tokens)
        );

        if let Some(chunk) = old_builder.build() {
            chunks.push(chunk);
        }

        // Split the large file
        let parts = self.split_large_file(&file)?;

        // Create chunks for each part
        for part in parts {
            let mut builder = ChunkBuilder::new(chunks.len(), self.max_chunk_tokens);
            builder.add_file(part);

            if let Some(chunk) = builder.build() {
                chunks.push(chunk);
            }
        }

        // Update current builder to new empty one
        *current_builder = ChunkBuilder::new(chunks.len(), self.max_chunk_tokens);

        Ok(())
    }

    /// Splits a large file into multiple parts with overlap.
    fn split_large_file(&self, file: &FileData) -> Result<Vec<FileData>> {
        let content = match &file.content {
            FileContent::Text(text) => text,
            FileContent::Binary { size } => {
                return Err(Error::FileTooLarge {
                    path: file.absolute_path.clone(),
                    size: file.token_count,
                    limit: self.max_chunk_tokens,
                });
            }
        };

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        if total_lines == 0 {
            return Ok(vec![file.clone()]);
        }

        let params = self.calculate_split_parameters(&lines, file.token_count);

        // Pre-allocate с запасом
        let mut parts = Vec::with_capacity(params.estimated_parts + 1);

        // Переиспользуем буфер для всех частей
        let estimated_chunk_size = content.len() / params.estimated_parts.max(1);
        let mut chunk_buffer = String::with_capacity(estimated_chunk_size + 1024);

        let mut start_line = 0;
        let mut part_number = 1;

        while start_line < total_lines {
            let end_line = (start_line + params.lines_per_chunk).min(total_lines);

            // Очищаем буфер вместо создания новой строки
            chunk_buffer.clear();

            // Эффективное добавление строк
            for (i, line) in lines[start_line..end_line].iter().enumerate() {
                if i > 0 {
                    chunk_buffer.push('\n');
                }
                chunk_buffer.push_str(line);
            }

            let token_count = self.tokenizer.estimate(&chunk_buffer);

            if token_count > self.max_chunk_tokens {
                warn!(
                    "Part {}/{} of '{}' has {} tokens (exceeds limit of {})",
                    part_number,
                    params.estimated_parts,
                    file.relative_path,
                    token_count,
                    self.max_chunk_tokens
                );
            }

            // Клонируем только финальный результат
            parts.push(FileData::new_text(
                file.absolute_path.clone(),
                format!(
                    "{} [Part {}/{}]",
                    file.relative_path, part_number, params.estimated_parts
                ),
                chunk_buffer.clone(),
                token_count,
            ));

            if end_line >= total_lines {
                break;
            }

            start_line = end_line.saturating_sub(params.overlap_lines);
            part_number += 1;
        }

        trace!(
            "Split '{}' into {} parts (estimated {})",
            file.relative_path,
            parts.len(),
            params.estimated_parts
        );

        Ok(parts)
    }

    /// Оптимизированный расчет параметров разбиения
    fn calculate_split_parameters(&self, lines: &[&str], total_tokens: usize) -> SplitParameters {
        let total_lines = lines.len();

        // Используем адаптивный размер выборки
        let sample_size = total_lines.min(100);

        // Избегаем лишних аллокаций при подсчете токенов
        let mut sample_buffer = String::with_capacity(sample_size * 80); // ~80 chars per line

        for (i, line) in lines[..sample_size].iter().enumerate() {
            if i > 0 {
                sample_buffer.push('\n');
            }
            sample_buffer.push_str(line);
        }

        let sample_tokens = self.tokenizer.estimate(&sample_buffer);

        let avg_tokens_per_line = if sample_size > 0 {
            (sample_tokens as f64 / sample_size as f64).max(1.0)
        } else {
            1.0
        };

        let lines_per_chunk = (self.max_chunk_tokens as f64 / avg_tokens_per_line) as usize;
        let lines_per_chunk = lines_per_chunk.max(1);

        let overlap_lines = (self.overlap_tokens as f64 / avg_tokens_per_line) as usize;
        let overlap_lines = overlap_lines.min(lines_per_chunk / 2);

        let estimated_parts = if lines_per_chunk > 0 {
            (total_lines + lines_per_chunk - overlap_lines - 1) / (lines_per_chunk - overlap_lines)
        } else {
            1
        };

        SplitParameters {
            lines_per_chunk,
            overlap_lines,
            estimated_parts,
        }
    }

    /// Logs results of the splitting operation.
    fn log_split_results(&self, chunks: &[Chunk]) {
        if chunks.is_empty() {
            return;
        }

        let total_files: usize = chunks.iter().map(|c| c.file_count()).sum();
        let avg_utilization = chunks
            .iter()
            .map(|c| c.utilization(self.max_chunk_tokens))
            .sum::<f64>()
            / chunks.len() as f64;

        debug!(
            "Created {} chunks from {} files (avg utilization: {:.1}%)",
            chunks.len(),
            total_files,
            avg_utilization * 100.0
        );
    }
}

/// Parameters for splitting a large file.
#[derive(Debug)]
struct SplitParameters {
    lines_per_chunk: usize,
    overlap_lines: usize,
    estimated_parts: usize,
}

/// Builder for constructing chunks incrementally.
struct ChunkBuilder {
    index: usize,
    files: Vec<FileData>,
    current_tokens: usize,
    max_tokens: usize,
}

impl ChunkBuilder {
    /// Creates a new chunk builder.
    fn new(index: usize, max_tokens: usize) -> Self {
        Self {
            index,
            files: Vec::new(),
            current_tokens: 0,
            max_tokens,
        }
    }

    /// Checks if a file can fit in the current chunk.
    fn can_fit(&self, tokens: usize) -> bool {
        self.current_tokens + tokens <= self.max_tokens
    }

    /// Adds a file to the chunk.
    fn add_file(&mut self, file: FileData) {
        self.current_tokens += file.token_count;
        self.files.push(file);
    }

    /// Builds the final chunk if not empty.
    fn build(self) -> Option<Chunk> {
        if self.files.is_empty() {
            None
        } else {
            Some(Chunk::new(self.index, self.files, self.current_tokens))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_config(max_tokens: usize) -> Config {
        use assert_fs::prelude::*;
        let temp = assert_fs::TempDir::new().unwrap();

        Config::builder()
            .root_dir(temp.path())
            .output_dir(temp.path().join("out"))
            .max_tokens(max_tokens)
            .overlap_tokens(100)
            .build()
            .unwrap()
    }

    #[test]
    fn test_splitter_single_file() {
        let config = create_test_config(3000);
        let splitter = Splitter::new(&config);

        let files = vec![FileData::new_text(
            PathBuf::from("test.rs"),
            "test.rs".to_string(),
            "fn main() {}".to_string(),
            300,
        )];

        let chunks = splitter.split(files).unwrap();

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].files.len(), 1);
        assert_eq!(chunks[0].total_tokens, 300);
    }

    #[test]
    fn test_splitter_multiple_files_single_chunk() {
        let config = create_test_config(3000);
        let splitter = Splitter::new(&config);

        let files = vec![
            FileData::new_text(
                PathBuf::from("file1.rs"),
                "file1.rs".to_string(),
                "fn main() {}".to_string(),
                300,
            ),
            FileData::new_text(
                PathBuf::from("file2.rs"),
                "file2.rs".to_string(),
                "pub fn test() {}".to_string(),
                300,
            ),
        ];

        let chunks = splitter.split(files).unwrap();

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].files.len(), 2);
        assert_eq!(chunks[0].total_tokens, 600);
    }

    #[test]
    fn test_splitter_multiple_chunks() {
        let config = create_test_config(2500);
        let splitter = Splitter::new(&config);

        let files = vec![
            FileData::new_text(
                PathBuf::from("file1.rs"),
                "file1.rs".to_string(),
                "fn main() {}".to_string(),
                300,
            ),
            FileData::new_text(
                PathBuf::from("file2.rs"),
                "file2.rs".to_string(),
                "pub fn test() {}".to_string(),
                300,
            ),
        ];

        let chunks = splitter.split(files).unwrap();

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].files.len(), 1);
        assert_eq!(chunks[1].files.len(), 1);
    }

    #[test]
    fn test_splitter_large_file() {
        let config = create_test_config(2500);
        let splitter = Splitter::new(&config);

        let large_content = (0..1000)
            .map(|i| format!("fn function_{}() {{}}", i))
            .collect::<Vec<_>>()
            .join("\n");

        let files = vec![FileData::new_text(
            PathBuf::from("large.rs"),
            "large.rs".to_string(),
            large_content.clone(),
            3000, // Exceeds limit
        )];

        let chunks = splitter.split(files).unwrap();

        assert!(chunks.len() > 1, "Large file should be split");

        // Check all parts
        for chunk in &chunks {
            assert_eq!(chunk.files.len(), 1);
            assert!(
                chunk.total_tokens <= config.max_tokens,
                "Chunk {} exceeds max tokens",
                chunk.index
            );
        }
    }

    #[test]
    fn test_chunk_utilization() {
        let chunk = Chunk::new(
            0,
            vec![],
            500,
        );

        assert_eq!(chunk.utilization(1000), 0.5);
        assert_eq!(chunk.utilization(500), 1.0);
    }

    #[test]
    fn test_splitter_empty_files() {
        let config = create_test_config(3000);
        let splitter = Splitter::new(&config);

        let chunks = splitter.split(vec![]).unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_splitter_binary_file_too_large() {
        let config = create_test_config(2500);
        let splitter = Splitter::new(&config);

        let files = vec![FileData::new_binary(
            PathBuf::from("large.bin"),
            "large.bin".to_string(),
            10000,
        )];

        // Override token count to exceed limit
        let mut files = files;
        files[0].token_count = 1000;

        let result = splitter.split(files);
        assert!(result.is_err());
    }
}