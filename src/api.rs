//! # API Start API
//!
//! High-level, ergonomic API for common use cases. Start here if you want to get
//! results fast without configuration overhead.
//!
//! ## Examples
//!
//! ```no_run
//!
//! // Simplest usage - scan current directory
//! use llm_utl::api::{Format, Preset, Scan};
//!
//! Scan::current_dir().run()?;
//!
//! // Scan specific directory
//! Scan::dir("./src").run()?;
//!
//! // Use a preset for common tasks
//! Scan::dir("./src")
//!     .preset(Preset::CodeReview)
//!     .run()?;
//!
//! // Custom configuration
//! Scan::dir("./project")
//!     .output("./prompts")
//!     .max_tokens(200_000)
//!     .format(Format::Json)
//!     .keep_tests()
//!     .keep_comments()
//!     .run()?;
//! # Ok::<(), llm_utl::Error>(())
//! ```

use crate::{Config, FileFilterConfig, FilterConfig, OutputFormat, Pipeline, PipelineStats, PresetKind, Result, TokenizerKind};
use std::path::{Path, PathBuf};

// ============================================================================
// Core API
// ============================================================================

/// Entry point for the API Start API.
///
/// Use this to build and execute scans with a fluent, type-safe interface.
///
/// # Examples
///
/// ```no_run
/// use llm_utl::api::*;
///
/// // Basic usage
/// Scan::current_dir().run()?;
///
/// // With configuration
/// Scan::dir("./src")
///     .max_tokens(150_000)
///     .preset(Preset::CodeReview)
///     .run()?;
/// # Ok::<(), llm_utl::Error>(())
/// ```
#[derive(Debug, Clone)]
#[must_use = "call .run() to execute the scan"]
pub struct Scan {
    dir: PathBuf,
    output: PathBuf,
    format: OutputFormat,
    max_tokens: usize,
    overlap: usize,
    preset: Option<PresetKind>,
    filters: FilterOptions,
    allow_files: Vec<String>,
    excludes: Vec<String>,
    exclude_files: Vec<String>,
    template_path: Option<PathBuf>,
    custom_format_name: Option<String>,
    custom_extension: Option<String>,
    custom_data: std::collections::HashMap<String, serde_json::Value>,
}

/// Filtering options for code processing.
#[derive(Debug, Clone)]
struct FilterOptions {
    tests: FilterMode,
    comments: FilterMode,
    doc_comments: FilterMode,
    debug_prints: FilterMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterMode {
    Remove,
    Keep,
}

impl Default for Scan {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("."),
            output: PathBuf::from("./out"),
            format: OutputFormat::Markdown,
            max_tokens: 100_000,
            overlap: 1_000,
            preset: None,
            filters: FilterOptions::default(),
            excludes: default_excludes(),
            exclude_files: vec![],
            allow_files: vec![],
            template_path: None,
            custom_format_name: None,
            custom_extension: None,
            custom_data: std::collections::HashMap::new(),
        }
    }
}

impl Default for FilterOptions {
    fn default() -> Self {
        Self {
            tests: FilterMode::Remove,
            comments: FilterMode::Remove,
            doc_comments: FilterMode::Remove,
            debug_prints: FilterMode::Remove,
        }
    }
}

impl Scan {
    /// Start a scan of the current directory.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_utl::api::*;
    ///
    /// let stats = Scan::current_dir().run()?;
    /// println!("Processed {} files", stats.total_files);
    /// # Ok::<(), llm_utl::Error>(())
    /// ```
    pub fn current_dir() -> Self {
        Self::default()
    }

    /// Start a scan of the specified directory.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_utl::api::*;
    ///
    /// Scan::dir("./src").run()?;
    /// Scan::dir("./my-project").run()?;
    /// # Ok::<(), llm_utl::Error>(())
    /// ```
    pub fn dir(path: impl Into<PathBuf>) -> Self {
        Self {
            dir: path.into(),
            ..Self::default()
        }
    }

    /// Set the output directory for generated files.
    ///
    /// Default: `./out`
    pub fn output(mut self, path: impl Into<PathBuf>) -> Self {
        self.output = path.into();
        self
    }

    /// Set the output format.
    ///
    /// Default: `Format::Markdown`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_utl::api::*;
    ///
    /// Scan::dir("./src")
    ///     .format(Format::Json)
    ///     .run()?;
    /// # Ok::<(), llm_utl::Error>(())
    /// ```
    pub fn format(mut self, format: Format) -> Self {
        self.format = format.into();
        self
    }

    /// Set maximum tokens per output file.
    ///
    /// Default: `100_000`
    pub fn max_tokens(mut self, tokens: usize) -> Self {
        self.max_tokens = tokens;
        self
    }

    /// Set overlap between chunks in tokens.
    ///
    /// Default: `1_000`
    pub fn overlap(mut self, tokens: usize) -> Self {
        self.overlap = tokens;
        self
    }

    /// Use a preset configuration for common tasks.
    ///
    /// Presets override filter settings with optimized defaults for specific use cases.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_utl::api::*;
    ///
    /// // Optimized for code review
    /// Scan::dir("./src")
    ///     .preset(Preset::CodeReview)
    ///     .run()?;
    ///
    /// // Optimized for documentation
    /// Scan::dir("./project")
    ///     .preset(Preset::Documentation)
    ///     .run()?;
    /// # Ok::<(), llm_utl::Error>(())
    /// ```
    pub fn preset(mut self, preset: Preset) -> Self {
        self.preset = Some(preset.into());
        self
    }

    /// Use a custom external Tera template.
    ///
    /// The template will override the built-in template for the selected format.
    /// For custom formats, combine with `.custom_format()`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_utl::api::*;
    ///
    /// // Override built-in markdown template
    /// Scan::dir("./src")
    ///     .format(Format::Markdown)
    ///     .template("./my-markdown.tera")
    ///     .run()?;
    ///
    /// // Use completely custom format
    /// Scan::dir("./src")
    ///     .custom_format("my_format", "txt")
    ///     .template("./custom.tera")
    ///     .run()?;
    /// # Ok::<(), llm_utl::Error>(())
    /// ```
    pub fn template(mut self, path: impl Into<PathBuf>) -> Self {
        self.template_path = Some(path.into());
        self
    }

    /// Define a custom output format with name and extension.
    ///
    /// Automatically sets format to `Format::Custom`. Requires a template
    /// to be specified via `.template()`.
    ///
    /// # Arguments
    ///
    /// * `name` - Internal template name
    /// * `extension` - File extension for output files (without leading dot)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_utl::api::*;
    ///
    /// Scan::dir("./src")
    ///     .custom_format("my_format", "txt")
    ///     .template("./custom.tera")
    ///     .run()?;
    /// # Ok::<(), llm_utl::Error>(())
    /// ```
    pub fn custom_format(mut self, name: impl Into<String>, extension: impl Into<String>) -> Self {
        self.format = OutputFormat::Custom;
        self.custom_format_name = Some(name.into());
        self.custom_extension = Some(extension.into());
        self
    }

    /// Add custom data to pass to templates.
    ///
    /// The data will be available in templates under `ctx.custom.<key>`.
    /// Can be called multiple times to add more data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_utl::api::*;
    /// use serde_json::json;
    ///
    /// Scan::dir("./src")
    ///     .template("./custom.tera")
    ///     .template_data("version", json!("1.0.0"))
    ///     .template_data("author", json!("John Doe"))
    ///     .template_data("project", json!("My Project"))
    ///     .run()?;
    /// # Ok::<(), llm_utl::Error>(())
    /// ```
    ///
    /// In template:
    /// ```tera
    /// Project: {{ ctx.custom.project }}
    /// Version: {{ ctx.custom.version }}
    /// Author: {{ ctx.custom.author }}
    /// ```
    pub fn template_data(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom_data.insert(key.into(), value);
        self
    }

    /// Include test files in the output.
    ///
    /// By default, tests are removed.
    pub fn keep_tests(mut self) -> Self {
        self.filters.tests = FilterMode::Keep;
        self
    }

    /// Remove test files from the output (default behavior).
    pub fn remove_tests(mut self) -> Self {
        self.filters.tests = FilterMode::Remove;
        self
    }

    /// Include comments in the output.
    ///
    /// By default, comments are removed.
    pub fn keep_comments(mut self) -> Self {
        self.filters.comments = FilterMode::Keep;
        self
    }

    /// Remove comments from the output (default behavior).
    pub fn remove_comments(mut self) -> Self {
        self.filters.comments = FilterMode::Remove;
        self
    }

    /// Include documentation comments in the output.
    ///
    /// By default, doc comments are removed.
    pub fn keep_doc_comments(mut self) -> Self {
        self.filters.doc_comments = FilterMode::Keep;
        self
    }

    /// Remove documentation comments from the output (default behavior).
    pub fn remove_doc_comments(mut self) -> Self {
        self.filters.doc_comments = FilterMode::Remove;
        self
    }

    /// Include debug print statements in the output.
    ///
    /// By default, debug prints are removed.
    pub fn keep_debug_prints(mut self) -> Self {
        self.filters.debug_prints = FilterMode::Keep;
        self
    }

    /// Remove debug print statements from the output (default behavior).
    pub fn remove_debug_prints(mut self) -> Self {
        self.filters.debug_prints = FilterMode::Remove;
        self
    }

    /// Add directories to exclude from scanning.
    ///
    /// Supports glob patterns (e.g., `**/node_modules`, `target/**`).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_utl::api::*;
    ///
    /// Scan::dir("./project")
    ///     .exclude(["**/node_modules", "**/dist"])
    ///     .run()?;
    /// # Ok::<(), llm_utl::Error>(())
    /// ```
    pub fn exclude<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.excludes.extend(patterns.into_iter().map(Into::into));
        self
    }

    /// Add files to exclude from scanning.
    ///
    /// Supports glob patterns (e.g., `**/*.rs`, `**.md`).
    /// ```
    pub fn exclude_files<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.excludes.extend(patterns.into_iter().map(Into::into));
        self
    }

    /// Add files to allow for scanning.
    ///
    /// Supports glob patterns (e.g., `**/*.rs`, `**.md`).
    /// ```
    pub fn allow_only<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allow_files.extend(patterns.into_iter().map(Into::into));
        self
    }

    /// Execute the scan and return statistics.
    ///
    /// This is a terminal operation that consumes the builder.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The directory doesn't exist
    /// - No processable files are found
    /// - Configuration is invalid
    /// - I/O errors occur during processing
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_utl::api::*;
    ///
    /// let stats = Scan::dir("./src").run()?;
    ///
    /// println!("Processed {} files in {:.2}s",
    ///     stats.total_files,
    ///     stats.duration.as_secs_f64()
    /// );
    /// # Ok::<(), llm_utl::Error>(())
    /// ```
    pub fn run(self) -> Result<PipelineStats> {
        let config = self.build_config()?;
        Pipeline::new(config)?.run()
    }

    fn build_config(self) -> Result<Config> {
        let mut builder = Config::builder()
            .root_dir(self.dir)
            .output_dir(self.output)
            .format(self.format)
            .max_tokens(self.max_tokens)
            .overlap_tokens(self.overlap)
            .tokenizer(TokenizerKind::Enhanced)
            .filter_config(FilterConfig {
                remove_tests: matches!(self.filters.tests, FilterMode::Remove),
                remove_doc_comments: matches!(self.filters.doc_comments, FilterMode::Remove),
                remove_comments: matches!(self.filters.comments, FilterMode::Remove),
                remove_blank_lines: true,
                preserve_headers: true,
                remove_debug_prints: matches!(self.filters.debug_prints, FilterMode::Remove),
            })
            .file_filter_config(FileFilterConfig::default()
                .allow_only(self.allow_files)
                .exclude_files(self.exclude_files)
                .exclude_directories(self.excludes));

        if let Some(preset) = self.preset {
            builder = builder.preset(preset);
        }

        // Add template configuration
        if let Some(template_path) = self.template_path {
            builder = builder.template_path(template_path);
        }

        if let Some(format_name) = self.custom_format_name {
            builder = builder.custom_format_name(format_name);
        }

        if let Some(extension) = self.custom_extension {
            builder = builder.custom_extension(extension);
        }

        if !self.custom_data.is_empty() {
            builder = builder.custom_data(self.custom_data);
        }

        builder.build()
    }
}

// ============================================================================
// Type-safe enums for common options
// ============================================================================

/// Output format for generated files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Markdown format (default)
    Markdown,
    /// XML format
    Xml,
    /// JSON format
    Json,
    /// Custom format (use with `.custom_format()`)
    Custom,
}

impl From<Format> for OutputFormat {
    fn from(format: Format) -> Self {
        match format {
            Format::Markdown => Self::Markdown,
            Format::Xml => Self::Xml,
            Format::Json => Self::Json,
            Format::Custom => Self::Custom,
        }
    }
}

/// Preset configurations for common use cases.
///
/// Each preset optimizes settings for a specific task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    /// Optimized for code review: removes tests, comments, and debug prints
    CodeReview,
    /// Optimized for documentation: keeps all comments and docs
    Documentation,
    /// Optimized for refactoring: clean view of structure
    Refactoring,
    /// Optimized for bug analysis: focuses on logic
    BugAnalysis,
    /// Optimized for security audit: includes everything
    SecurityAudit,
    /// Optimized for test generation: keeps tests as examples
    TestGeneration,
    /// Optimized for architecture review: high-level view
    ArchitectureReview,
    /// Optimized for performance analysis: focuses on algorithms
    PerformanceAnalysis,
    /// Optimized for migration planning: comprehensive view
    MigrationPlan,
    /// Optimized for API design: focuses on public interfaces
    ApiDesign,
}

impl From<Preset> for PresetKind {
    fn from(preset: Preset) -> Self {
        match preset {
            Preset::CodeReview => Self::CodeReview,
            Preset::Documentation => Self::Documentation,
            Preset::Refactoring => Self::Refactoring,
            Preset::BugAnalysis => Self::BugAnalysis,
            Preset::SecurityAudit => Self::SecurityAudit,
            Preset::TestGeneration => Self::TestGeneration,
            Preset::ArchitectureReview => Self::ArchitectureReview,
            Preset::PerformanceAnalysis => Self::PerformanceAnalysis,
            Preset::MigrationPlan => Self::MigrationPlan,
            Preset::ApiDesign => Self::ApiDesign,
        }
    }
}

// ============================================================================
// Preset shortcuts for common tasks
// ============================================================================

impl Scan {
    /// API preset: Code review configuration.
    ///
    /// Equivalent to `.preset(Preset::CodeReview)`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_utl::api::*;
    ///
    /// Scan::dir("./src")
    ///     .code_review()
    ///     .run()?;
    /// # Ok::<(), llm_utl::Error>(())
    /// ```
    pub fn code_review(self) -> Self {
        self.preset(Preset::CodeReview)
    }

    /// API preset: Documentation configuration.
    ///
    /// Equivalent to `.preset(Preset::Documentation)`.
    pub fn documentation(self) -> Self {
        self.preset(Preset::Documentation)
    }

    /// API preset: Security audit configuration.
    ///
    /// Equivalent to `.preset(Preset::SecurityAudit)`.
    pub fn security_audit(self) -> Self {
        self.preset(Preset::SecurityAudit)
    }

    /// API preset: Bug analysis configuration.
    ///
    /// Equivalent to `.preset(Preset::BugAnalysis)`.
    pub fn bug_analysis(self) -> Self {
        self.preset(Preset::BugAnalysis)
    }

    /// API preset: Refactoring configuration.
    ///
    /// Equivalent to `.preset(Preset::Refactoring)`.
    pub fn refactoring(self) -> Self {
        self.preset(Preset::Refactoring)
    }

    /// API preset: Test generation configuration.
    ///
    /// Equivalent to `.preset(Preset::TestGeneration)`.
    pub fn test_generation(self) -> Self {
        self.preset(Preset::TestGeneration)
    }
}

// ============================================================================
// Convenience functions
// ============================================================================

/// Scan the current directory with default settings.
///
/// This is the simplest way to use the library.
///
/// # Examples
///
/// ```no_run
/// use llm_utl::api::*;
///
/// let stats = scan()?;
/// println!("Created {} files", stats.files_written);
/// # Ok::<(), llm_utl::Error>(())
/// ```
pub fn scan() -> Result<PipelineStats> {
    Scan::current_dir().run()
}

/// Scan a specific directory with default settings.
///
/// # Examples
///
/// ```no_run
/// use llm_utl::api::*;
///
/// let stats = scan_dir("./src")?;
/// # Ok::<(), llm_utl::Error>(())
/// ```
pub fn scan_dir(path: impl AsRef<Path>) -> Result<PipelineStats> {
    Scan::dir(path.as_ref()).run()
}

// ============================================================================
// Utilities
// ============================================================================

fn default_excludes() -> Vec<String> {
    vec![
        "**/node_modules".to_string(),
        "**/target".to_string(),
        "**/out".to_string(),
        "**/dist".to_string(),
        "**/build".to_string(),
        "**/.git".to_string(),
        "**/templates".to_string(),
        "**/.idea".to_string(),
        "**/.vscode".to_string(),
        "**/vendor".to_string(),
    ]
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use serde_json::json;
    use super::*;

    #[test]
    fn scan_builder_has_sensible_defaults() {
        let scan = Scan::current_dir();
        assert_eq!(scan.dir, PathBuf::from("."));
        assert_eq!(scan.output, PathBuf::from("./out"));
        assert_eq!(scan.max_tokens, 100_000);
    }

    #[test]
    fn scan_builder_is_fluent() {
        let scan = Scan::dir("./test")
            .output("./custom-out")
            .max_tokens(200_000)
            .format(Format::Json)
            .keep_tests()
            .keep_comments();

        assert_eq!(scan.dir, PathBuf::from("./test"));
        assert_eq!(scan.output, PathBuf::from("./custom-out"));
        assert_eq!(scan.max_tokens, 200_000);
        assert_eq!(scan.format, OutputFormat::Json);
        assert_eq!(scan.filters.tests, FilterMode::Keep);
        assert_eq!(scan.filters.comments, FilterMode::Keep);
    }

    #[test]
    fn preset_shortcuts_work() {
        let scan = Scan::dir("./src").code_review();
        assert_eq!(scan.preset, Some(PresetKind::CodeReview));

        let scan = Scan::dir("./src").documentation();
        assert_eq!(scan.preset, Some(PresetKind::Documentation));
    }

    #[test]
    fn exclude_patterns_are_additive() {
        let scan = Scan::dir("./src")
            .exclude(["**/test1"])
            .exclude(["**/test2", "**/test3"]);

        assert!(scan.excludes.contains(&"**/test1".to_string()));
        assert!(scan.excludes.contains(&"**/test2".to_string()));
        assert!(scan.excludes.contains(&"**/test3".to_string()));
    }
}