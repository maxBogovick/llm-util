use crate::error::{Error, Result};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

static BINARY_EXTENSIONS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "exe", "dll", "so", "dylib", "a", "o", "obj", "png", "jpg", "jpeg", "gif", "bmp", "ico",
        "webp", "mp3", "mp4", "avi", "mkv", "mov", "wav", "flac", "pdf", "doc", "docx", "xls",
        "xlsx", "ppt", "pptx", "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "wasm", "pyc",
        "class",
    ]
    .into_iter()
    .collect()
});

/// Represents a file with its content and metadata.
#[derive(Debug, Clone)]
pub struct FileData {
    /// Absolute path to the file
    pub absolute_path: PathBuf,

    /// Relative path from the root directory
    pub relative_path: String,

    /// File content (text or binary)
    pub content: FileContent,

    /// Estimated token count
    pub token_count: usize,
}

/// File content type (text or binary).
#[derive(Debug, Clone)]
pub enum FileContent {
    /// Text content with UTF-8 string
    Text(String),

    /// Binary content with file size
    Binary {
        /// Size of the binary file in bytes
        size: u64,
    },
}

impl FileData {
    /// Creates a new text file data.
    #[must_use]
    pub fn new_text(
        absolute_path: PathBuf,
        relative_path: String,
        content: String,
        token_count: usize,
    ) -> Self {
        Self {
            absolute_path,
            relative_path,
            content: FileContent::Text(content),
            token_count,
        }
    }

    /// Creates a new binary file data.
    #[must_use]
    pub fn new_binary(absolute_path: PathBuf, relative_path: String, size: u64) -> Self {
        Self {
            absolute_path,
            relative_path,
            content: FileContent::Binary { size },
            token_count: 0,
        }
    }

    /// Returns true if this is a text file.
    #[must_use]
    pub const fn is_text(&self) -> bool {
        matches!(self.content, FileContent::Text(_))
    }

    /// Returns true if this is a binary file.
    #[must_use]
    pub const fn is_binary(&self) -> bool {
        matches!(self.content, FileContent::Binary { .. })
    }

    /// Returns the text content if this is a text file.
    #[must_use]
    pub fn content_str(&self) -> Option<&str> {
        match &self.content {
            FileContent::Text(s) => Some(s),
            FileContent::Binary { .. } => None,
        }
    }

    /// Returns the size in bytes.
    #[must_use]
    pub fn size_bytes(&self) -> u64 {
        match &self.content {
            FileContent::Text(s) => s.len() as u64,
            FileContent::Binary { size } => *size,
        }
    }

    /// Returns the number of lines (for text files only).
    #[must_use]
    pub fn line_count(&self) -> Option<usize> {
        self.content_str().map(|s| s.lines().count())
    }
}

/// Determines if a file is likely binary by analyzing its content.
///
/// # Algorithm
///
/// 1. Reads the first 8KB of the file
/// 2. Checks for null bytes (binary indicator)
/// 3. Calculates the ratio of ASCII characters
/// 4. Files with null bytes or low ASCII ratio are considered binary
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read.
pub(crate) fn is_likely_binary(path: &Path) -> Result<bool> {
    const BUFFER_SIZE: usize = 8192;
    const ASCII_THRESHOLD: f64 = 0.85;

    let file = File::open(path).map_err(|e| Error::io(path, e))?;
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    let mut buffer = [0u8; BUFFER_SIZE];

    let bytes_read = reader.read(&mut buffer).map_err(|e| Error::io(path, e))?;

    if bytes_read == 0 {
        return Ok(false);
    }

    let sample = &buffer[..bytes_read];

    // Быстрая проверка на null bytes с помощью memchr
    if memchr::memchr(0, sample).is_some() {
        return Ok(true);
    }

    // Подсчет ASCII символов
    let ascii_count = sample.iter().filter(|&&b| b < 128).count();
    let ascii_ratio = ascii_count as f64 / bytes_read as f64;

    Ok(ascii_ratio < ASCII_THRESHOLD)
}

/// Checks if a file extension suggests a text file.
#[must_use]
pub(crate) fn has_text_extension(path: &Path) -> bool {
    static TEXT_EXTENSIONS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
        [
            "rs", "toml", "md", "txt", "json", "yaml", "yml", "js", "ts", "jsx", "tsx", "py", "go",
            "java", "c", "cpp", "h", "hpp", "cs", "rb", "php", "html", "css", "scss", "sass",
            "xml", "svg", "sh", "bash", "zsh", "fish", "vim", "lua",
        ]
        .into_iter()
        .collect()
    });
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| TEXT_EXTENSIONS.contains(ext))
        .unwrap_or(false)
}

/// Checks if a file extension suggests a binary file.
#[must_use]
pub(crate) fn has_binary_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| BINARY_EXTENSIONS.contains(ext))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_file_data_text() {
        let data = FileData::new_text(
            PathBuf::from("test.rs"),
            "test.rs".to_string(),
            "fn main() {}".to_string(),
            3,
        );

        assert!(data.is_text());
        assert!(!data.is_binary());
        assert_eq!(data.content_str(), Some("fn main() {}"));
        assert_eq!(data.token_count, 3);
    }

    #[test]
    fn test_file_data_binary() {
        let data = FileData::new_binary(PathBuf::from("test.exe"), "test.exe".to_string(), 1024);

        assert!(data.is_binary());
        assert!(!data.is_text());
        assert_eq!(data.content_str(), None);
        assert_eq!(data.size_bytes(), 1024);
    }

    #[test]
    fn test_is_likely_binary_text_file() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("test.txt");
        file.write_str("Hello, world!").unwrap();

        assert!(!is_likely_binary(file.path()).unwrap());
    }

    #[test]
    fn test_is_likely_binary_binary_file() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("test.bin");

        let mut f = File::create(file.path()).unwrap();
        f.write_all(&[0u8; 100]).unwrap(); // Null bytes

        assert!(is_likely_binary(file.path()).unwrap());
    }

    #[test]
    fn test_is_likely_binary_empty_file() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("empty.txt");
        file.touch().unwrap();

        assert!(!is_likely_binary(file.path()).unwrap());
    }

    #[test]
    fn test_has_text_extension() {
        assert!(has_text_extension(Path::new("test.rs")));
        assert!(has_text_extension(Path::new("config.toml")));
        assert!(has_text_extension(Path::new("README.md")));
        assert!(!has_text_extension(Path::new("binary.exe")));
        assert!(!has_text_extension(Path::new("no_extension")));
    }

    #[test]
    fn test_has_binary_extension() {
        assert!(has_binary_extension(Path::new("app.exe")));
        assert!(has_binary_extension(Path::new("image.png")));
        assert!(has_binary_extension(Path::new("archive.zip")));
        assert!(!has_binary_extension(Path::new("code.rs")));
    }

    #[test]
    fn test_line_count() {
        let data = FileData::new_text(
            PathBuf::from("test.rs"),
            "test.rs".to_string(),
            "line1\nline2\nline3".to_string(),
            5,
        );

        assert_eq!(data.line_count(), Some(3));
    }

    #[test]
    fn test_line_count_binary() {
        let data = FileData::new_binary(PathBuf::from("test.exe"), "test.exe".to_string(), 1024);

        assert_eq!(data.line_count(), None);
    }
}
