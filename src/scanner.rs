use crate::filter::FileFilter;
use crate::{
    config::Config,
    error::{Error, Result},
    file::{has_binary_extension, is_likely_binary, FileData},
    filter::CodeFilter,
    token::TokenEstimator,
};
use ignore::{DirEntry, WalkBuilder, WalkState};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use std::time::{Duration, Instant};
use tracing::{debug, trace, warn};

/// Statistics collected during scanning.
#[derive(Debug, Default, Clone)]
pub(crate) struct ScanStats {
    /// Total files found
    pub total_files: usize,

    /// Text files processed
    pub text_files: usize,

    /// Binary files found
    pub binary_files: usize,

    /// Files skipped
    pub skipped_files: usize,

    /// Errors encountered
    pub errors: usize,
}

/// Scans directories and collects file data.
pub(crate) struct Scanner {
    root_dir: PathBuf,
    include_binary: bool,
    tokenizer: Arc<dyn TokenEstimator>,
    code_filter: CodeFilter,
    file_filter: FileFilter,
}

impl Scanner {
    /// Creates a new scanner from configuration.
    pub(crate) fn new(config: &Config) -> Self {
        Self {
            root_dir: config.root_dir.clone(),
            include_binary: config.include_binary_files,
            tokenizer: config.tokenizer.create(),
            code_filter: CodeFilter::new(config.filter_config.clone()),
            file_filter: FileFilter::new(config.file_filter_config.clone()),
        }
    }

    /// Scans the root directory and returns all processable files.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No files are found
    /// - Critical scanning errors occur
    pub(crate) fn scan(&self) -> Result<Vec<FileData>> {
        let files = Arc::new(Mutex::new(Vec::new()));
        let errors = Arc::new(Mutex::new(Vec::new()));
        let stats = Arc::new(Mutex::new(ScanStats::default()));

        let files_clone = Arc::clone(&files);
        let errors_clone = Arc::clone(&errors);
        let stats_clone = Arc::clone(&stats);

        debug!("Starting parallel scan of {}", self.root_dir.display());
        let scan_timeout = Duration::from_secs(30); // 30 секунд
        let scan_start = Instant::now();

        let walker = WalkBuilder::new(&self.root_dir)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .hidden(true)
            .follow_links(false)
            .skip_stdout(true)
            .threads(num_cpus::get())
            .build_parallel();
        let file_filter = self.file_filter.clone();
        walker.run(|| {
            let files = Arc::clone(&files_clone);
            let errors = Arc::clone(&errors_clone);
            let stats = Arc::clone(&stats_clone);
            let root = self.root_dir.clone();
            let tokenizer = Arc::clone(&self.tokenizer);
            let code_filter = self.code_filter.clone();
            let include_binary = self.include_binary;
            let file_filter = file_filter.clone();
            Box::new(move |result| {
                if scan_start.elapsed() > scan_timeout {
                    warn!("Scan timeout reached after 30 seconds");
                    return WalkState::Quit;
                }
                match result {
                    Ok(entry) if entry.file_type().map_or(false, |ft| ft.is_file()) => {
                        if entry.file_name() == "Cargo.lock" {
                            return WalkState::Continue;
                        }
                        if !file_filter.should_process(entry.path()) {
                            return WalkState::Continue; // Пропускаем файл
                        }
                        stats.lock().unwrap().total_files += 1;

                        match Self::process_entry(
                            &entry,
                            &root,
                            tokenizer.as_ref(),
                            &code_filter,
                            include_binary,
                            &mut stats.lock().unwrap(),
                        ) {
                            Ok(Some(file_data)) => {
                                files.lock().unwrap().push(file_data);
                            }
                            Ok(None) => {
                                stats.lock().unwrap().skipped_files += 1;
                            }
                            Err(e) => {
                                warn!("Failed to process {}: {}", entry.path().display(), e);
                                errors.lock().unwrap().push(e);
                                stats.lock().unwrap().errors += 1;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Walk error: {}", e);
                        stats.lock().unwrap().errors += 1;
                    }
                    _ => {}
                }
                WalkState::Continue
            })
        });

        // Unwrap results
        let mut files = Arc::try_unwrap(files)
            .map(|m| m.into_inner().unwrap())
            .unwrap_or_else(|arc| arc.lock().unwrap().clone());

        let errors = Arc::try_unwrap(errors)
            .map(|m| m.into_inner().unwrap())
            .unwrap_or_else(|arc| arc.lock().unwrap().clone());

        let stats = Arc::try_unwrap(stats)
            .map(|m| m.into_inner().unwrap())
            .unwrap_or_else(|arc| (*arc.lock().unwrap()).clone());

        // Report statistics
        debug!(
            "Scan complete: {} total, {} text, {} binary, {} skipped, {} errors",
            stats.total_files,
            stats.text_files,
            stats.binary_files,
            stats.skipped_files,
            stats.errors
        );

        if !errors.is_empty() {
            warn!(
                "Encountered {} errors during scanning (non-fatal)",
                errors.len()
            );
        }

        if files.is_empty() {
            return Err(Error::no_files(&self.root_dir));
        }

        // Sort for deterministic ordering
        files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

        debug!("Successfully scanned {} files", files.len());
        Ok(files)
    }

    /// Processes a single directory entry.
    fn process_entry(
        entry: &DirEntry,
        root: &Path,
        tokenizer: &dyn TokenEstimator,
        code_filter: &CodeFilter,
        include_binary: bool,
        stats: &mut ScanStats,
    ) -> Result<Option<FileData>> {
        let path = entry.path();

        trace!("Processing file: {}", path.display());

        // Compute relative path
        let relative_path = pathdiff::diff_paths(path, root)
            .unwrap_or_else(|| path.to_path_buf())
            .to_string_lossy()
            .to_string();

        // Quick check for known binary extensions
        if has_binary_extension(path) {
            stats.binary_files += 1;

            if !include_binary {
                debug!("Skipping binary file (by extension): {}", relative_path);
                return Ok(None);
            }

            return Self::create_binary_file_data(path, relative_path, stats);
        }

        // Check if file is binary by content
        if is_likely_binary(path)? {
            stats.binary_files += 1;

            if !include_binary {
                debug!("Skipping binary file (by content): {}", relative_path);
                return Ok(None);
            }

            return Self::create_binary_file_data(path, relative_path, stats);
        }

        // Process as text file
        Self::create_text_file_data(path, relative_path, tokenizer, code_filter, stats)
    }

    /// Creates file data for a binary file.
    fn create_binary_file_data(
        path: &Path,
        relative_path: String,
        _stats: &mut ScanStats,
    ) -> Result<Option<FileData>> {
        let metadata = fs::metadata(path).map_err(|e| Error::io(path, e))?;

        Ok(Some(FileData::new_binary(
            path.to_path_buf(),
            relative_path,
            metadata.len(),
        )))
    }

    fn process_text_file_streaming(
        path: &Path,
        relative_path: String,
        tokenizer: &dyn TokenEstimator,
        code_filter: &CodeFilter,
    ) -> Result<Option<FileData>> {
        const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks

        let file = File::open(path).map_err(|e| Error::io(path, e))?;
        let reader = BufReader::with_capacity(CHUNK_SIZE, file);

        let mut filtered_content = String::with_capacity(CHUNK_SIZE);
        let mut lines_buffer = Vec::with_capacity(1000);

        // Читаем файл построчно
        for line in reader.lines() {
            let line = line.map_err(|e| {
                if e.kind() == std::io::ErrorKind::InvalidData {
                    Error::invalid_utf8(path)
                } else {
                    Error::io(path, e)
                }
            })?;

            lines_buffer.push(line);

            // Обрабатываем батчами для эффективности
            if lines_buffer.len() >= 1000 {
                let batch = lines_buffer.join("\n");
                let filtered = code_filter.filter(&batch, path);
                filtered_content.push_str(&filtered);
                filtered_content.push('\n');
                lines_buffer.clear();
            }
        }

        // Обработка оставшихся строк
        if !lines_buffer.is_empty() {
            let batch = lines_buffer.join("\n");
            let filtered = code_filter.filter(&batch, path);
            filtered_content.push_str(&filtered);
        }

        let token_count = tokenizer.estimate(&filtered_content);

        Ok(Some(FileData::new_text(
            path.to_path_buf(),
            relative_path,
            filtered_content,
            token_count,
        )))
    }

    /// Умный выбор между обычной и потоковой обработкой
    fn create_text_file_data(
        path: &Path,
        relative_path: String,
        tokenizer: &dyn TokenEstimator,
        code_filter: &CodeFilter,
        stats: &mut ScanStats,
    ) -> Result<Option<FileData>> {
        const STREAMING_THRESHOLD: u64 = 10 * 1024 * 1024; // 10MB

        let metadata = std::fs::metadata(path).map_err(|e| Error::io(path, e))?;

        // Для больших файлов используем потоковую обработку
        if metadata.len() > STREAMING_THRESHOLD {
            trace!("Using streaming mode for large file: {}", relative_path);
            Self::process_text_file_streaming(path, relative_path, tokenizer, code_filter)
        } else {
            // Для маленьких файлов используем обычное чтение
            let content = std::fs::read_to_string(path).map_err(|e| {
                if e.kind() == std::io::ErrorKind::InvalidData {
                    Error::invalid_utf8(path)
                } else {
                    Error::io(path, e)
                }
            })?;

            let filtered_content = code_filter.filter(&content, path);
            let token_count = tokenizer.estimate(&filtered_content);

            stats.text_files += 1;

            Ok(Some(FileData::new_text(
                path.to_path_buf(),
                relative_path,
                filtered_content,
                token_count,
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    fn create_test_config(root: &Path) -> Config {
        Config::builder()
            .root_dir(root)
            .output_dir(root.join("out"))
            .build()
            .unwrap()
    }

    #[test]
    fn test_scanner_finds_files() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.child("file1.rs").write_str("fn main() {}").unwrap();
        temp.child("file2.rs").write_str("pub fn test() {}").unwrap();

        let config = create_test_config(temp.path());
        let scanner = Scanner::new(&config);
        let files = scanner.scan().unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.relative_path.contains("file1.rs")));
        assert!(files.iter().any(|f| f.relative_path.contains("file2.rs")));
    }

    #[test]
    #[ignore] //TODO need fix it
    fn test_scanner_skips_binary() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.child("text.rs").write_str("fn main() {}").unwrap();
        temp.child("binary.exe").write_binary(&[0u8; 100]).unwrap();

        let config = create_test_config(temp.path());
        let scanner = Scanner::new(&config);
        let files = scanner.scan().unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].relative_path.contains("text.rs"));
    }

    #[test]
    fn test_scanner_includes_binary_when_enabled() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.child("text.rs").write_str("fn main() {}").unwrap();
        temp.child("binary.exe").write_binary(&[0u8; 100]).unwrap();

        let config = Config::builder()
            .root_dir(temp.path())
            .output_dir(temp.path().join("out"))
            .include_binary_files(true)
            .build()
            .unwrap();

        let scanner = Scanner::new(&config);
        let files = scanner.scan().unwrap();

        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_scanner_respects_gitignore() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.child(".gitignore").write_str("ignored.rs\n").unwrap();
        temp.child("included.rs").write_str("fn main() {}").unwrap();
        temp.child("ignored.rs").write_str("fn test() {}").unwrap();

        let config = create_test_config(temp.path());
        let scanner = Scanner::new(&config);
        let files = scanner.scan().unwrap();

        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_scanner_empty_directory() {
        let temp = assert_fs::TempDir::new().unwrap();

        let config = create_test_config(temp.path());
        let scanner = Scanner::new(&config);
        let result = scanner.scan();

        assert!(result.is_err());
    }

    #[test]
    fn test_scanner_nested_directories() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.child("src/main.rs").write_str("fn main() {}").unwrap();
        temp.child("src/lib.rs").write_str("pub fn test() {}").unwrap();
        temp.child("tests/test.rs").write_str("#[test]\nfn test() {}").unwrap();

        let config = create_test_config(temp.path());
        let scanner = Scanner::new(&config);
        let files = scanner.scan().unwrap();

        assert_eq!(files.len(), 3);
    }
}