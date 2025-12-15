//! Basic example of using llm-utl as a library
//!
//! This example shows the simplest way to convert a directory to prompts.

use llm_utl::{Config, Pipeline};

fn main() -> anyhow::Result<()> {
    // Create a simple configuration
    let config = Config::builder()
        .root_dir("./src")
        .output_dir("./output")
        .build()?;

    // Run the pipeline
    let stats = Pipeline::new(config)?.run()?;

    // Print summary
    stats.print_summary();

    println!("\n✓ Successfully converted {} files into {} chunks",
        stats.text_files,
        stats.total_chunks
    );

    println!("✓ Output written to: {}", stats.output_directory);

    Ok(())
}