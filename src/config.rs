use crate::error::{Error, Result};
use crate::filter::{FileFilterConfig, FilterConfig};
use crate::token::TokenizerKind;
use std::path::PathBuf;

const DEFAULT_MAX_TOKENS: usize = 100_000;
const DEFAULT_OVERLAP_TOKENS: usize = 1_000;
const DEFAULT_CHUNK_SAFETY_MARGIN: usize = 2_000;
const DEFAULT_OUTPUT_PATTERN: &str = "prompt_{index:03}.{ext}";

/// Output format for generated prompts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Markdown format with code blocks
    Markdown,
    /// XML format with structured tags
    Xml,
    /// JSON format with metadata
    Json,
}

impl OutputFormat {
    /// Returns the file extension for this format.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Xml => "xml",
            Self::Json => "json",
        }
    }

    /// Returns the template name for this format.
    #[must_use]
    pub const fn template_name(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Xml => "xml",
            Self::Json => "json",
        }
    }
}

/// Configuration for the llm-utl pipeline.
///
/// Use [`Config::builder()`] to construct a new configuration.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Config {
    /// Root directory to scan for files
    pub root_dir: PathBuf,

    /// Output directory for generated prompts
    pub output_dir: PathBuf,

    /// Output filename pattern (supports {index}, {index:03}, {ext})
    pub output_pattern: String,

    /// Output format
    pub format: OutputFormat,

    /// Maximum tokens per chunk
    pub max_tokens: usize,

    /// Overlap tokens between chunks for context continuity
    pub overlap_tokens: usize,

    /// Safety margin to prevent exceeding limits
    pub chunk_safety_margin: usize,

    /// Tokenizer implementation to use
    pub tokenizer: TokenizerKind,

    /// Whether to prefer splitting at line boundaries
    pub prefer_line_boundaries: bool,

    /// Code filtering configuration
    pub filter_config: FilterConfig,

    /// Code filtering configuration
    pub file_filter_config: FileFilterConfig,

    /// Dry run mode (no file writes)
    pub dry_run: bool,

    /// Include binary files in output
    pub include_binary_files: bool,

    /// Create backups of existing files
    pub backup_existing: bool,
}

impl Config {
    /// Creates a new configuration builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_utl::Config;
    ///
    /// let config = Config::builder()
    ///     .root_dir("./src")
    ///     .max_tokens(50_000)
    ///     .build()
    ///     .expect("valid configuration");
    /// ```
    #[must_use]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Validates the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Root directory doesn't exist
    /// - Token limits are invalid
    /// - Output pattern is invalid
    pub fn validate(&self) -> Result<()> {
        // Validate root directory
        if !self.root_dir.exists() {
            return Err(Error::config(format!(
                "Root directory does not exist: {}",
                self.root_dir.display()
            )));
        }

        if !self.root_dir.is_dir() {
            return Err(Error::config(format!(
                "Root path is not a directory: {}",
                self.root_dir.display()
            )));
        }

        // Validate token limits
        if self.max_tokens == 0 {
            return Err(Error::config("max_tokens must be greater than 0"));
        }

        if self.overlap_tokens >= self.max_tokens {
            return Err(Error::config(format!(
                "overlap_tokens ({}) must be less than max_tokens ({})",
                self.overlap_tokens, self.max_tokens
            )));
        }

        if self.chunk_safety_margin >= self.max_tokens {
            return Err(Error::config(format!(
                "chunk_safety_margin ({}) must be less than max_tokens ({})",
                self.chunk_safety_margin, self.max_tokens
            )));
        }

        // Validate output pattern
        if !self.output_pattern.contains("{index") {
            return Err(Error::invalid_pattern(
                &self.output_pattern,
                "Pattern must contain {index} or {index:03} placeholder",
            ));
        }

        if !self.output_pattern.contains("{ext}") {
            return Err(Error::invalid_pattern(
                &self.output_pattern,
                "Pattern must contain {ext} placeholder",
            ));
        }

        Ok(())
    }

    /// Returns the effective chunk size after applying safety margin.
    #[must_use]
    pub const fn effective_chunk_size(&self) -> usize {
        self.max_tokens.saturating_sub(self.chunk_safety_margin)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("."),
            output_dir: PathBuf::from("out"),
            output_pattern: DEFAULT_OUTPUT_PATTERN.to_string(),
            format: OutputFormat::Markdown,
            max_tokens: DEFAULT_MAX_TOKENS,
            overlap_tokens: DEFAULT_OVERLAP_TOKENS,
            chunk_safety_margin: DEFAULT_CHUNK_SAFETY_MARGIN,
            tokenizer: TokenizerKind::Simple,
            prefer_line_boundaries: true,
            filter_config: FilterConfig::default(),
            file_filter_config: FileFilterConfig::default(),
            dry_run: false,
            include_binary_files: false,
            backup_existing: true,
        }
    }
}

/// Builder for creating a [`Config`].
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    root_dir: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    output_pattern: Option<String>,
    format: Option<OutputFormat>,
    max_tokens: Option<usize>,
    overlap_tokens: Option<usize>,
    chunk_safety_margin: Option<usize>,
    tokenizer: Option<TokenizerKind>,
    prefer_line_boundaries: Option<bool>,
    filter_config: Option<FilterConfig>,
    file_filter_config: Option<FileFilterConfig>,
    dry_run: bool,
    include_binary_files: bool,
    backup_existing: Option<bool>,
}

impl ConfigBuilder {
    /// Sets the root directory to scan.
    #[must_use]
    pub fn root_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.root_dir = Some(path.into());
        self
    }

    /// Sets the output directory for generated files.
    #[must_use]
    pub fn output_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.output_dir = Some(path.into());
        self
    }

    /// Sets the output filename pattern.
    ///
    /// Pattern must contain `{index}` and `{ext}` placeholders.
    #[must_use]
    pub fn output_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.output_pattern = Some(pattern.into());
        self
    }

    /// Sets the output format.
    #[must_use]
    pub fn format(mut self, format: OutputFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Sets the maximum tokens per chunk.
    #[must_use]
    pub fn max_tokens(mut self, tokens: usize) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    /// Sets the overlap tokens between chunks.
    #[must_use]
    pub fn overlap_tokens(mut self, tokens: usize) -> Self {
        self.overlap_tokens = Some(tokens);
        self
    }

    /// Sets the chunk safety margin.
    #[must_use]
    pub fn chunk_safety_margin(mut self, margin: usize) -> Self {
        self.chunk_safety_margin = Some(margin);
        self
    }

    /// Sets the tokenizer implementation.
    #[must_use]
    pub fn tokenizer(mut self, kind: TokenizerKind) -> Self {
        self.tokenizer = Some(kind);
        self
    }

    /// Enables or disables line boundary preference.
    #[must_use]
    pub fn prefer_line_boundaries(mut self, enabled: bool) -> Self {
        self.prefer_line_boundaries = Some(enabled);
        self
    }

    /// Enables dry run mode (no file writes).
    #[must_use]
    pub fn dry_run(mut self, enabled: bool) -> Self {
        self.dry_run = enabled;
        self
    }

    /// Enables or disables binary file inclusion.
    #[must_use]
    pub fn include_binary_files(mut self, enabled: bool) -> Self {
        self.include_binary_files = enabled;
        self
    }

    /// Enables or disables backup creation.
    #[must_use]
    pub fn backup_existing(mut self, enabled: bool) -> Self {
        self.backup_existing = Some(enabled);
        self
    }

    /// Sets the code filtering configuration.
    #[must_use]
    pub fn filter_config(mut self, config: FilterConfig) -> Self {
        self.filter_config = Some(config);
        self
    }

    /// Sets the code filtering configuration.
    #[must_use]
    pub fn file_filter_config(mut self, config: FileFilterConfig) -> Self {
        self.file_filter_config = Some(config);
        self
    }

    /// Builds the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    pub fn build(self) -> Result<Config> {
        let config = Config {
            root_dir: self.root_dir.unwrap_or_else(|| PathBuf::from(".")),
            output_dir: self.output_dir.unwrap_or_else(|| PathBuf::from("out")),
            output_pattern: self
                .output_pattern
                .unwrap_or_else(|| DEFAULT_OUTPUT_PATTERN.to_string()),
            format: self.format.unwrap_or(OutputFormat::Markdown),
            max_tokens: self.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
            overlap_tokens: self.overlap_tokens.unwrap_or(DEFAULT_OVERLAP_TOKENS),
            chunk_safety_margin: self
                .chunk_safety_margin
                .unwrap_or(DEFAULT_CHUNK_SAFETY_MARGIN),
            tokenizer: self.tokenizer.unwrap_or(TokenizerKind::Simple),
            prefer_line_boundaries: self.prefer_line_boundaries.unwrap_or(true),
            filter_config: self.filter_config.unwrap_or_default(),
            file_filter_config: self.file_filter_config.unwrap_or_default(),
            dry_run: self.dry_run,
            include_binary_files: self.include_binary_files,
            backup_existing: self.backup_existing.unwrap_or(true),
        };

        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let temp = assert_fs::TempDir::new().unwrap();
        let config = Config::builder()
            .root_dir(temp.path())
            .build()
            .unwrap();

        assert_eq!(config.max_tokens, DEFAULT_MAX_TOKENS);
        assert_eq!(config.format, OutputFormat::Markdown);
    }

    #[test]
    fn test_invalid_root_dir() {
        let result = Config::builder()
            .root_dir("/nonexistent/path/that/should/not/exist")
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_token_limits() {
        let temp = assert_fs::TempDir::new().unwrap();

        let result = Config::builder()
            .root_dir(temp.path())
            .max_tokens(1000)
            .overlap_tokens(1000)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_pattern() {
        let temp = assert_fs::TempDir::new().unwrap();

        let result = Config::builder()
            .root_dir(temp.path())
            .output_pattern("invalid_pattern")
            .build();

        assert!(result.is_err());
    }
}