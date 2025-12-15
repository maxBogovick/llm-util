use anyhow::Context;
use clap::Parser;
use llm_utl::{Config, FilterConfig, OutputFormat, Pipeline, TokenizerKind, FileFilterConfig};
use std::path::PathBuf;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser, Debug)]
#[command(
    name = "llm-ult",
    version,
    author,
    about = "Convert code repositories into LLM prompts",
    long_about = None
)]
struct Cli {
    /// Root directory to scan
    #[arg(short, long, default_value = ".")]
    dir: PathBuf,

    /// Output directory for generated prompts
    #[arg(short, long, default_value = "out")]
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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Настройка трассировки
    setup_tracing(cli.verbose)?;

    // Построение конфигурации
    let config = Config::builder()
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
        )
        .build()
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