//! Dry run example
//!
//! This example shows how to preview what would be generated
//! without actually writing any files.

use llm_utl::{Config, OutputFormat};

fn main() -> anyhow::Result<()> {
    // Configure with dry-run mode enabled
    let config = Config::builder()
        .root_dir("./src")
        .output_dir("./prompts")
        .format(OutputFormat::Markdown)
        .max_tokens(75_000)
        .dry_run(true)  // Enable dry-run mode
        .build()?;

    println!("Running in DRY RUN mode - no files will be written\n");

    // Run the pipeline
    let stats = llm_utl::run(config)?;

    // Print what would have been generated
    println!("\nWould have generated:");
    println!("  {} output files", stats.total_chunks);
    println!("  Total tokens: {}", stats.total_tokens);
    println!("  Avg tokens per chunk: {}", stats.avg_tokens_per_chunk);
    println!();
    println!("Chunk distribution:");
    println!("  Min chunk size: {} tokens", stats.min_chunk_tokens);
    println!("  Avg chunk size: {} tokens", stats.avg_tokens_per_chunk);
    println!("  Max chunk size: {} tokens", stats.max_chunk_tokens);

    Ok(())
}