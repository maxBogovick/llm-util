//! Example with custom configuration
//!
//! This example demonstrates how to customize the conversion process
//! with different settings for tokens, output format, and tokenizer.

use llm_utl::{Config, OutputFormat, TokenizerKind};

fn main() -> anyhow::Result<()> {
    // Create a custom configuration
    let config = Config::builder()
        .root_dir("./src")
        .output_dir("./prompts")
        .output_pattern("chunk_{index:04}.{ext}")  // Custom pattern
        .format(OutputFormat::Json)                 // Use JSON format
        .max_tokens(50_000)                         // Smaller chunks
        .overlap_tokens(500)                        // Less overlap
        .tokenizer(TokenizerKind::Enhanced)         // Enhanced tokenizer
        .prefer_line_boundaries(true)               // Split at line boundaries
        .build()?;

    println!("Configuration:");
    println!("  Root directory: {}", config.root_dir.display());
    println!("  Output directory: {}", config.output_dir.display());
    println!("  Max tokens per chunk: {}", config.max_tokens);
    println!("  Overlap tokens: {}", config.overlap_tokens);
    println!("  Format: {:?}", config.format);
    println!();

    // Run the pipeline
    let stats = llm_utl::run(config)?;

    // Print detailed statistics
    stats.print_summary();

    println!("\nPerformance Metrics:");
    println!("  Files/sec: {:.2}", stats.throughput_files_per_sec());
    println!("  Tokens/sec: {:.2}", stats.throughput_tokens_per_sec());

    Ok(())
}