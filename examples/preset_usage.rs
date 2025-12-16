//! Example showing how to use LLM presets for specialized output.
//!
//! This example demonstrates using the code-review preset to generate
//! prompts optimized for code review tasks.

use llm_utl::{Config, PresetKind, FilterConfig};
use anyhow::Result;

fn main() -> Result<()> {
    // Example 1: Code Review with preset
    code_review_example()?;

    // Example 2: Documentation Generation
    documentation_example()?;

    // Example 3: Security Audit
    security_audit_example()?;

    // Example 4: List all available presets
    list_presets();

    Ok(())
}

fn code_review_example() -> Result<()> {
    println!("=== Code Review Example ===\n");

    let config = Config::builder()
        .root_dir("./src")
        .output_dir("./out/code-review")
        .preset(PresetKind::CodeReview)
        .max_tokens(150_000) // Preset suggests 150k tokens
        .filter_config(FilterConfig::default()) // Remove tests and debug prints
        .build()?;

    println!("Config created for code review");
    println!("- Preset: {:?}", config.preset);
    println!("- Max tokens: {}", config.max_tokens);
    println!("- Output: {}\n", config.output_dir.display());

    // In a real scenario, you would run the pipeline:
    // Pipeline::new(config)?.run()?;

    Ok(())
}

fn documentation_example() -> Result<()> {
    println!("=== Documentation Generation Example ===\n");

    let config = Config::builder()
        .root_dir("./src")
        .output_dir("./out/documentation")
        .preset(PresetKind::Documentation)
        .max_tokens(100_000)
        .filter_config(FilterConfig::preserve_docs()) // Keep doc comments
        .build()?;

    println!("Config created for documentation");
    println!("- Preset: {:?}", config.preset);
    println!("- Keeps doc comments: {:?}", config.filter_config.remove_doc_comments);
    println!("- Output: {}\n", config.output_dir.display());

    Ok(())
}

fn security_audit_example() -> Result<()> {
    println!("=== Security Audit Example ===\n");

    let config = Config::builder()
        .root_dir("./src")
        .output_dir("./out/security-audit")
        .preset(PresetKind::SecurityAudit)
        .max_tokens(120_000)
        .filter_config(FilterConfig::production()) // Production-ready code
        .build()?;

    println!("Config created for security audit");
    println!("- Preset: {:?}", config.preset);
    println!("- Max tokens: {}", config.max_tokens);
    println!("- Output: {}\n", config.output_dir.display());

    Ok(())
}

fn list_presets() {
    println!("=== Available Presets ===\n");

    for preset_kind in PresetKind::all() {
        println!("- {} (id: {})",
            format!("{:?}", preset_kind),
            preset_kind.id()
        );
    }

    println!("\n=== Preset Details ===\n");

    // Show details for code review preset
    use llm_utl::LLMPreset;
    let code_review = LLMPreset::for_kind(PresetKind::CodeReview);

    println!("Code Review Preset:");
    println!("  Name: {}", code_review.name);
    println!("  Description: {}", code_review.description);
    println!("  Suggested Model: {}", code_review.suggested_model);
    println!("  Max Tokens Hint: {}", code_review.max_tokens_hint);
    println!("  Temperature Hint: {}", code_review.temperature_hint);
    println!();

    // Show details for documentation preset
    let documentation = LLMPreset::for_kind(PresetKind::Documentation);

    println!("Documentation Preset:");
    println!("  Name: {}", documentation.name);
    println!("  Description: {}", documentation.description);
    println!("  Suggested Model: {}", documentation.suggested_model);
    println!("  Max Tokens Hint: {}", documentation.max_tokens_hint);
    println!("  Temperature Hint: {}", documentation.temperature_hint);
}