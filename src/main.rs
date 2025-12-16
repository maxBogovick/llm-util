use anyhow::Context;
use clap::Parser;
use llm_utl::{Config, FileFilterConfig, FilterConfig, OutputFormat, Pipeline, PresetKind, TokenizerKind};
use std::path::PathBuf;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser, Debug)]
#[command(
    name = "llm-ult",
    version,
    author,
    about = "Convert code repositories into LLM prompts",
    long_about = "Convert code repositories into LLM-friendly prompts with intelligent chunking.\n\n\
    This tool scans a directory, processes source files, and generates optimized prompts \
    for use with Large Language Models. It respects .gitignore patterns and provides \
    various presets for different analysis tasks.\n\n\
    USAGE EXAMPLES:\n  \
      # Scan current directory\n  \
      llm-utl\n\n  \
      # Scan a specific project\n  \
      llm-utl --dir ./my-project --out ./prompts\n\n  \
      # Use a preset for code review\n  \
      llm-utl --dir ./src --preset code-review\n\n  \
      # Generate JSON output with custom token limit\n  \
      llm-utl --dir ./src --format json --max-tokens 150000"
)]
struct Cli {
    /// Root directory to scan for source files (must contain code files)
    #[arg(short, long, default_value = ".", value_name = "PATH")]
    dir: PathBuf,

    /// Output directory for generated prompts
    #[arg(short, long, default_value = "out", value_name = "PATH")]
    out: PathBuf,

    /// Output filename pattern
    #[arg(long, default_value = "prompt_{index:03}.{ext}")]
    pattern: String,

    /// Output format
    #[arg(short, long, value_enum, default_value = "markdown")]
    format: CliFormat,

    /// Max tokens per chunk
    #[arg(long, default_value_t = 100_000)]
    max_tokens: usize,

    /// Overlap tokens between chunks
    #[arg(long, default_value_t = 1_000)]
    overlap: usize,

    /// Tokenizer to use
    #[arg(long, value_enum, default_value = "enhanced")]
    tokenizer: CliTokenizer,

    /// LLM preset for specialized output
    #[arg(short, long, value_enum)]
    preset: Option<CliPreset>,

    /// Dry run (don't write files)
    #[arg(long)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum CliFormat {
    Markdown,
    Xml,
    Json,
}

impl From<CliFormat> for OutputFormat {
    fn from(f: CliFormat) -> Self {
        match f {
            CliFormat::Markdown => Self::Markdown,
            CliFormat::Xml => Self::Xml,
            CliFormat::Json => Self::Json,
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum CliTokenizer {
    Simple,
    Enhanced,
}

impl From<CliTokenizer> for TokenizerKind {
    fn from(t: CliTokenizer) -> Self {
        match t {
            CliTokenizer::Simple => Self::Simple,
            CliTokenizer::Enhanced => Self::Enhanced,
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum CliPreset {
    /// Comprehensive code review
    CodeReview,
    /// Documentation generation
    Documentation,
    /// Refactoring suggestions
    Refactoring,
    /// Bug analysis
    BugAnalysis,
    /// Security audit
    SecurityAudit,
    /// Test generation
    TestGeneration,
    /// Architecture review
    ArchitectureReview,
    /// Performance analysis
    PerformanceAnalysis,
    /// Migration planning
    MigrationPlan,
    /// API design review
    ApiDesign,
}

impl From<CliPreset> for PresetKind {
    fn from(p: CliPreset) -> Self {
        match p {
            CliPreset::CodeReview => Self::CodeReview,
            CliPreset::Documentation => Self::Documentation,
            CliPreset::Refactoring => Self::Refactoring,
            CliPreset::BugAnalysis => Self::BugAnalysis,
            CliPreset::SecurityAudit => Self::SecurityAudit,
            CliPreset::TestGeneration => Self::TestGeneration,
            CliPreset::ArchitectureReview => Self::ArchitectureReview,
            CliPreset::PerformanceAnalysis => Self::PerformanceAnalysis,
            CliPreset::MigrationPlan => Self::MigrationPlan,
            CliPreset::ApiDesign => Self::ApiDesign,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Настройка трассировки
    setup_tracing(cli.verbose)?;

    // Построение конфигурации
    let mut builder = Config::builder()
        .root_dir(cli.dir)
        .output_dir(cli.out)
        .output_pattern(cli.pattern)
        .format(cli.format.into())
        .max_tokens(cli.max_tokens)
        .overlap_tokens(cli.overlap)
        .tokenizer(cli.tokenizer.into())
        .dry_run(cli.dry_run)
        .filter_config(FilterConfig {
            remove_tests: true,
            remove_doc_comments: true,
            remove_comments: true,
            remove_blank_lines: true,
            preserve_headers: true,
            remove_debug_prints: true,
        })
        .file_filter_config(FileFilterConfig::default()
                                //.allow_only(vec!("*.toml".to_string()))
                                //.allow_only(vec!(PathBuf::from("pipeline.rs")))
            .exclude_directories(vec!("**/templates".to_string(), "**/out".to_string(), "**/target".to_string()))
        );

    // Добавление preset если указан
    if let Some(preset) = cli.preset {
        builder = builder.preset(preset.into());
    }

    let config = builder.build()
        .context("Failed to build configuration")?;

    // Запуск pipeline
    Pipeline::new(config)
        .context("Failed to create pipeline")?
        .run()
        .context("Pipeline execution failed")?;

    Ok(())
}

fn setup_tracing(verbosity: u8) -> anyhow::Result<()> {
    let filter = match verbosity {
        0 => EnvFilter::new("llm_utl=info"),
        1 => EnvFilter::new("llm_utl=debug"),
        _ => EnvFilter::new("llm_utl=trace"),
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false).with_thread_ids(false))
        .init();

    Ok(())
}