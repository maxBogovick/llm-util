//! Complete example of using presets with the full pipeline.
//!
//! This example demonstrates how to:
//! 1. Configure different presets
//! 2. Run the pipeline with preset-specific settings
//! 3. Generate specialized outputs for different use cases

use llm_utl::{Config, Pipeline, PresetKind, FilterConfig, OutputFormat};
use anyhow::Result;

fn main() -> Result<()> {
    println!("LLM-UTL Preset Pipeline Examples\n");
    println!("=================================\n");

    // Example 1: Generate code review prompts
    generate_code_review_prompts()?;

    // Example 2: Generate refactoring suggestions
    generate_refactoring_prompts()?;

    // Example 3: Generate test cases
    generate_test_generation_prompts()?;

    Ok(())
}

/// Generate prompts optimized for code review
fn generate_code_review_prompts() -> Result<()> {
    println!("📝 Generating Code Review Prompts...\n");

    let config = Config::builder()
        .root_dir("./src")
        .output_dir("./out/code-review")
        .preset(PresetKind::CodeReview)
        .format(OutputFormat::Markdown)
        .max_tokens(150_000)
        .filter_config(FilterConfig {
            remove_tests: true,
            remove_doc_comments: false, // Keep docs for review
            remove_comments: false,     // Keep comments for context
            remove_blank_lines: true,
            preserve_headers: true,
            remove_debug_prints: true,
        })
        .build()?;

    println!("Configuration:");
    println!("  Preset: Code Review");
    println!("  Format: Markdown");
    println!("  Max Tokens: 150,000");
    println!("  Keeps: Doc comments, regular comments");
    println!("  Removes: Tests, debug prints\n");

    // Run the pipeline
    let stats = Pipeline::new(config)?.run()?;

    println!("Results:");
    stats.print_summary();
    println!();

    Ok(())
}

/// Generate prompts for refactoring analysis
fn generate_refactoring_prompts() -> Result<()> {
    println!("🔧 Generating Refactoring Analysis Prompts...\n");

    let config = Config::builder()
        .root_dir("./src")
        .output_dir("./out/refactoring")
        .preset(PresetKind::Refactoring)
        .format(OutputFormat::Markdown)
        .max_tokens(120_000)
        .filter_config(FilterConfig::minimal()) // Only code, no noise
        .build()?;

    println!("Configuration:");
    println!("  Preset: Refactoring");
    println!("  Format: Markdown");
    println!("  Max Tokens: 120,000");
    println!("  Minimal mode: Only essential code\n");

    let stats = Pipeline::new(config)?.run()?;

    println!("Results:");
    stats.print_summary();
    println!();

    Ok(())
}

/// Generate prompts for test generation
fn generate_test_generation_prompts() -> Result<()> {
    println!("🧪 Generating Test Generation Prompts...\n");

    let config = Config::builder()
        .root_dir("./src")
        .output_dir("./out/test-generation")
        .preset(PresetKind::TestGeneration)
        .format(OutputFormat::Json) // JSON format for easier parsing
        .max_tokens(150_000)
        .filter_config(FilterConfig {
            remove_tests: true, // Remove existing tests
            remove_doc_comments: false,
            remove_comments: false,
            remove_blank_lines: true,
            preserve_headers: true,
            remove_debug_prints: true,
        })
        .build()?;

    println!("Configuration:");
    println!("  Preset: Test Generation");
    println!("  Format: JSON");
    println!("  Max Tokens: 150,000");
    println!("  Removes: Existing tests");
    println!("  Keeps: Documentation for context\n");

    let stats = Pipeline::new(config)?.run()?;

    println!("Results:");
    stats.print_summary();
    println!();

    Ok(())
}

/// Example of using all presets with custom configurations
#[allow(dead_code)]
fn generate_all_preset_types() -> Result<()> {
    let presets_to_generate = vec![
        (PresetKind::CodeReview, "code-review"),
        (PresetKind::Documentation, "documentation"),
        (PresetKind::Refactoring, "refactoring"),
        (PresetKind::BugAnalysis, "bug-analysis"),
        (PresetKind::SecurityAudit, "security-audit"),
        (PresetKind::TestGeneration, "test-generation"),
        (PresetKind::ArchitectureReview, "architecture-review"),
        (PresetKind::PerformanceAnalysis, "performance-analysis"),
        (PresetKind::MigrationPlan, "migration-plan"),
        (PresetKind::ApiDesign, "api-design"),
    ];

    for (preset_kind, dir_name) in presets_to_generate {
        println!("Generating {} prompts...", dir_name);

        let config = Config::builder()
            .root_dir("./src")
            .output_dir(format!("./out/{}", dir_name))
            .preset(preset_kind)
            .format(OutputFormat::Markdown)
            .build()?;

        let stats = Pipeline::new(config)?.run()?;
        println!("✓ Generated {} chunks\n", stats.total_chunks);
    }

    Ok(())
}