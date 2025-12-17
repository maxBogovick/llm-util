use crate::{
    config::Config,
    error::{Error, Result},
    splitter::Chunk,
    template::TemplateEngine,
};
use serde::Serialize;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
use tracing::{debug, info};

/// Summary of written output files.
#[derive(Debug, Serialize)]
pub(crate) struct WriteSummary {
    /// Total number of chunks written
    pub total_chunks: usize,

    /// Total number of files across all chunks
    pub total_files: usize,

    /// Total token count across all chunks
    pub total_tokens: usize,

    /// Execution duration in seconds
    pub duration_secs: f64,

    /// Output directory path
    pub output_directory: String,

    /// Output format used
    pub format: String,

    /// Individual chunk summaries
    pub chunks: Vec<ChunkSummary>,

    /// Generation timestamp
    pub generated_at: String,
}

/// Summary of a single chunk.
#[derive(Debug, Serialize)]
pub(crate) struct ChunkSummary {
    /// Chunk index (1-based for user display)
    pub index: usize,

    /// Number of files in chunk
    pub files: usize,

    /// Token count in chunk
    pub tokens: usize,

    /// Output filename
    pub filename: String,
}

/// Writes chunks to output files with atomic operations.
pub(crate) struct Writer {
    output_dir: PathBuf,
    output_pattern: String,
    format: crate::config::OutputFormat,
    backup_existing: bool,
    template_engine: TemplateEngine,
    custom_extension: Option<String>,
}

impl Writer {
    /// Creates a new writer from configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if template engine initialization fails.
    pub(crate) fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            output_dir: config.output_dir.clone(),
            output_pattern: config.output_pattern.clone(),
            format: config.format,
            backup_existing: config.backup_existing,
            template_engine: TemplateEngine::new(config)?,
            custom_extension: config.custom_extension.clone(),
        })
    }

    /// Writes all chunks to output files.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Output directory cannot be created
    /// - Template rendering fails
    /// - File write operations fail
    pub(crate) fn write_chunks(&self, chunks: &[Chunk]) -> Result<()> {
        // Create output directory
        fs::create_dir_all(&self.output_dir)
            .map_err(|e| Error::io(&self.output_dir, e))?;

        info!("Writing {} chunks to {}", chunks.len(), self.output_dir.display());

        // Write each chunk
        for chunk in chunks {
            self.write_chunk(chunk, chunks.len())?;
        }

        info!("Successfully wrote {} chunk files", chunks.len());
        Ok(())
    }

    /// Writes a single chunk to file.
    fn write_chunk(&self, chunk: &Chunk, total_chunks: usize) -> Result<()> {
        let content = self.template_engine.render(chunk, total_chunks)?;
        let path = self.get_output_path(chunk.index);

        self.write_file_atomic(&path, &content)?;

        debug!(
            "Wrote chunk {}/{} ({} files, {} tokens) to {}",
            chunk.index + 1,
            total_chunks,
            chunk.files.len(),
            chunk.total_tokens,
            path.display()
        );

        Ok(())
    }

    /// Generates the output file path for a chunk.
    fn get_output_path(&self, index: usize) -> PathBuf {
        use crate::config::OutputFormat;

        // Determine extension based on format
        let extension = match self.format {
            OutputFormat::Custom => self
                .custom_extension
                .as_deref()
                .unwrap_or("txt"),
            _ => self.format.extension(),
        };

        let filename = self
            .output_pattern
            .replace("{index:03}", &format!("{:03}", index + 1))
            .replace("{index:02}", &format!("{:02}", index + 1))
            .replace("{index}", &(index + 1).to_string())
            .replace("{ext}", extension);

        self.output_dir.join(filename)
    }

    /// Writes a file atomically with optional backup.
    ///
    /// # Process
    ///
    /// 1. Creates backup if file exists and backup is enabled
    /// 2. Writes content to temporary file
    /// 3. Syncs temporary file to disk
    /// 4. Atomically renames temporary file to target path
    ///
    /// This ensures no data loss if the write is interrupted.
    fn write_file_atomic(&self, path: &Path, content: &str) -> Result<()> {
        // Create backup if needed
        if path.exists() && self.backup_existing {
            self.backup_file(path)?;
        }

        // Write to temporary file
        let temp_path = path.with_extension("tmp");
        let mut temp_file = fs::File::create(&temp_path)
            .map_err(|e| Error::io(&temp_path, e))?;

        temp_file
            .write_all(content.as_bytes())
            .map_err(|e| Error::io(&temp_path, e))?;

        // Ensure data is flushed to disk
        temp_file
            .sync_all()
            .map_err(|e| Error::io(&temp_path, e))?;

        drop(temp_file);

        // Atomic rename
        fs::rename(&temp_path, path)
            .map_err(|e| Error::io(path, e))?;

        Ok(())
    }

    /// Creates a timestamped backup of an existing file.
    fn backup_file(&self, path: &Path) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_nanos();

        let filename = path
            .file_name()
            .ok_or_else(|| Error::config("Invalid file path"))?
            .to_string_lossy();

        let backup_name = format!("{}.backup.{}", filename, timestamp);
        let backup_path = path
            .parent()
            .ok_or_else(|| Error::config("Invalid file path"))?
            .join(backup_name);

        fs::copy(path, &backup_path)
            .map_err(|e| Error::io(&backup_path, e))?;

        debug!("Created backup: {}", backup_path.display());
        Ok(())
    }

    /// Writes a summary JSON file with metadata about all chunks.
    ///
    /// # Errors
    ///
    /// Returns an error if the summary file cannot be written.
    pub(crate) fn write_summary(&self, chunks: &[Chunk], duration: Duration) -> Result<()> {
        let summary = WriteSummary {
            total_chunks: chunks.len(),
            total_files: chunks.iter().map(|c| c.files.len()).sum(),
            total_tokens: chunks.iter().map(|c| c.total_tokens).sum(),
            duration_secs: duration.as_secs_f64(),
            output_directory: self.output_dir.display().to_string(),
            format: format!("{:?}", self.format),
            chunks: chunks
                .iter()
                .map(|c| ChunkSummary {
                    index: c.index + 1,
                    files: c.files.len(),
                    tokens: c.total_tokens,
                    filename: self
                        .get_output_path(c.index)
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                })
                .collect(),
            generated_at: chrono::Local::now()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        };

        let summary_path = self.output_dir.join("summary.json");
        let file = fs::File::create(&summary_path)
            .map_err(|e| Error::io(&summary_path, e))?;

        serde_json::to_writer_pretty(file, &summary)
            .map_err(Error::from)?;

        info!("Wrote summary to {}", summary_path.display());
        Ok(())
    }

    /// Cleans up old backup files (optional utility method).
    ///
    /// Removes backup files older than the specified duration.
    #[allow(dead_code)]
    pub(crate) fn cleanup_old_backups(&self, max_age: Duration) -> Result<usize> {
        let mut removed = 0;
        let now = SystemTime::now();

        for entry in fs::read_dir(&self.output_dir)
            .map_err(|e| Error::io(&self.output_dir, e))?
        {
            let entry = entry.map_err(|e| Error::io(&self.output_dir, e))?;
            let path = entry.path();

            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.contains(".backup.") {
                    let metadata = fs::metadata(&path)
                        .map_err(|e| Error::io(&path, e))?;

                    if let Ok(modified) = metadata.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age > max_age {
                                fs::remove_file(&path)
                                    .map_err(|e| Error::io(&path, e))?;
                                removed += 1;
                                debug!("Removed old backup: {}", path.display());
                            }
                        }
                    }
                }
            }
        }

        if removed > 0 {
            info!("Cleaned up {} old backup files", removed);
        }

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::FileData;
    use assert_fs::prelude::*;
    use std::path::PathBuf;

    fn create_test_config(output_dir: &Path) -> Config {
        use assert_fs::TempDir;
        let temp = TempDir::new().unwrap();

        Config::builder()
            .root_dir(temp.path())
            .output_dir(output_dir)
            .build()
            .unwrap()
    }

    fn create_test_chunk(index: usize) -> Chunk {
        Chunk::new(
            index,
            vec![FileData::new_text(
                PathBuf::from("test.rs"),
                "test.rs".to_string(),
                "fn main() {}".to_string(),
                100,
            )],
            100,
        )
    }

    #[test]
    fn test_writer_creates_output_directory() {
        let temp = assert_fs::TempDir::new().unwrap();
        let output_dir = temp.child("output");

        let config = create_test_config(output_dir.path());
        let writer = Writer::new(&config).unwrap();

        let chunks = vec![create_test_chunk(0)];
        writer.write_chunks(&chunks).unwrap();

        assert!(output_dir.exists());
    }

    #[test]
    fn test_writer_creates_chunk_files() {
        let temp = assert_fs::TempDir::new().unwrap();
        let output_dir = temp.child("output");

        let config = create_test_config(output_dir.path());
        let writer = Writer::new(&config).unwrap();

        let chunks = vec![create_test_chunk(0), create_test_chunk(1)];
        writer.write_chunks(&chunks).unwrap();

        assert!(output_dir.child("prompt_001.md").exists());
        assert!(output_dir.child("prompt_002.md").exists());
    }

    #[test]
    fn test_writer_creates_summary() {
        let temp = assert_fs::TempDir::new().unwrap();
        let output_dir = temp.child("output");

        let config = create_test_config(output_dir.path());
        let writer = Writer::new(&config).unwrap();

        let chunks = vec![create_test_chunk(0)];
        writer.write_chunks(&chunks).unwrap();
        writer.write_summary(&chunks, Duration::from_secs(1)).unwrap();

        assert!(output_dir.child("summary.json").exists());
    }

    #[test]
    fn test_writer_creates_backup() {
        let temp = assert_fs::TempDir::new().unwrap();
        let output_dir = temp.child("output");
        output_dir.create_dir_all().unwrap();

        let existing_file = output_dir.child("prompt_001.md");
        existing_file.write_str("old content").unwrap();

        let config = create_test_config(output_dir.path());
        let writer = Writer::new(&config).unwrap();

        let chunks = vec![create_test_chunk(0)];
        writer.write_chunks(&chunks).unwrap();

        // Check backup was created
        let entries: Vec<_> = fs::read_dir(output_dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();

        assert!(entries.iter().any(|name| name.contains(".backup.")));
    }

    #[test]
    fn test_get_output_path() {
        let temp = assert_fs::TempDir::new().unwrap();
        let config = create_test_config(temp.path());
        let writer = Writer::new(&config).unwrap();

        let path = writer.get_output_path(0);
        assert!(path.to_string_lossy().contains("prompt_001.md"));

        let path = writer.get_output_path(9);
        assert!(path.to_string_lossy().contains("prompt_010.md"));
    }

    #[test]
    fn test_cleanup_old_backups() {
        use std::thread;
        use std::time::Duration as StdDuration;

        let temp = assert_fs::TempDir::new().unwrap();
        let output_dir = temp.child("output");
        output_dir.create_dir_all().unwrap();

        // Create old backup
        let old_backup = output_dir.child("file.backup.123");
        old_backup.write_str("old").unwrap();

        // Wait a bit
        thread::sleep(StdDuration::from_millis(100));

        // Create recent backup
        let new_backup = output_dir.child("file.backup.456");
        new_backup.write_str("new").unwrap();

        let config = create_test_config(output_dir.path());
        let writer = Writer::new(&config).unwrap();

        // Clean up backups older than 50ms
        let removed = writer
            .cleanup_old_backups(StdDuration::from_millis(50))
            .unwrap();

        assert_eq!(removed, 1);
        assert!(!old_backup.exists());
        assert!(new_backup.exists());
    }
}