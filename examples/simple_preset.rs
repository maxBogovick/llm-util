//! Simple example of using presets - perfect for README documentation.

use llm_utl::{Config, Pipeline, PresetKind};

fn main() -> anyhow::Result<()> {
    // Create a configuration with a code review preset
    let config = Config::builder()
        .root_dir("./src")
        .output_dir("./prompts")
        .preset(PresetKind::CodeReview) // Use code review preset
        .max_tokens(150_000)
        .build()?;

    // Run the pipeline
    let stats = Pipeline::new(config)?.run()?;

    // Print results
    println!("✓ Generated {} prompt files", stats.total_chunks);
    println!("✓ Processed {} source files", stats.total_files);
    println!("✓ Total tokens: ~{}", stats.total_tokens);

    Ok(())
}