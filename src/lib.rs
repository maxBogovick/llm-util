//! # llm-utl
//!
//! A high-performance library for converting code repositories into LLM prompts.
//!
//! ## Features
//!
//! - Parallel file scanning with `.gitignore` support
//! - Smart chunking with configurable token limits
//! - Multiple output formats (Markdown, XML, JSON)
//! - Atomic file operations with automatic backups
//! - Advanced tokenization strategies
//!
//! ## Quick Start
//!
//! ```no_run
//! use llm_utl::{Config, Pipeline, OutputFormat};
//!
//! # fn main() -> anyhow::Result<()> {
//! let config = Config::builder()
//!     .root_dir("./src")
//!     .output_dir("./prompts")
//!     .format(OutputFormat::Markdown)
//!     .max_tokens(100_000)
//!     .build()?;
//!
//! Pipeline::new(config)?.run()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! The library follows a pipeline architecture:
//! 1. **Scanner**: Discovers files respecting `.gitignore`
//! 2. **Tokenizer**: Estimates token counts
//! 3. **Splitter**: Divides content into optimal chunks
//! 4. **Writer**: Renders and persists output files

#![warn(
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    clippy::all,
    clippy::pedantic,
    clippy::nursery
)]
#![allow(clippy::module_name_repetitions)]

mod config;
mod error;
mod file;
mod filter;
mod pipeline;
mod scanner;
mod splitter;
mod template;
mod token;
mod writer;

pub mod preset;

pub use config::{Config, ConfigBuilder, OutputFormat};
pub use error::{Error, Result};
pub use file::FileData;
pub use filter::{CodeFilter, FileFilterConfig, FilterConfig};
pub use pipeline::{Pipeline, PipelineStats};
pub use preset::{LLMPreset, PresetKind};
pub use splitter::Chunk;
pub use token::{TokenEstimator, TokenizerKind};

/// Runs the complete conversion pipeline with the given configuration.
///
/// This is the main entry point for the library.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration is invalid
/// - Root directory doesn't exist or is inaccessible
/// - No processable files are found
/// - Output directory cannot be created
/// - File operations fail
///
/// # Examples
///
/// ```no_run
/// use llm_utl::{Config, run};
///
/// # fn main() -> anyhow::Result<()> {
/// let config = Config::builder()
///     .root_dir(".")
///     .build()?;
///
/// run(config)?;
/// # Ok(())
/// # }
/// ```
pub fn run(config: Config) -> Result<PipelineStats> {
    Pipeline::new(config)?.run()
}