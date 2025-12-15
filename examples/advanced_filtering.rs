//! Advanced filtering example
//!
//! This example shows how to use file and code filtering features
//! to control what gets included in the output.

use llm_utl::{Config, FileFilterConfig, FilterConfig};

fn main() -> anyhow::Result<()> {
    // Configure code filtering
    let filter_config = FilterConfig {
        remove_tests: true,
        remove_doc_comments: false,    // Keep documentation
        remove_comments: true,          // Remove regular comments
        remove_blank_lines: true,
        preserve_headers: true,         // Keep copyright/license headers
        remove_debug_prints: true,      // Remove println!, dbg!, etc.
    };

    // Configure file filtering
    let file_filter = FileFilterConfig::default()
        .exclude_directories(vec![
            "**/target".to_string(),
            "**/node_modules".to_string(),
            "**/.git".to_string(),
            "**/out".to_string(),
        ])
        .exclude_files(vec![
            "*.lock".to_string(),
            "*.min.js".to_string(),
        ]);
        // Or use whitelist mode:
        // .allow_only(vec!["src/**/*.rs".to_string()]);

    // Build configuration
    let config = Config::builder()
        .root_dir(".")
        .output_dir("./filtered_output")
        .filter_config(filter_config)
        .file_filter_config(file_filter)
        .max_tokens(100_000)
        .build()?;

    println!("Running with advanced filtering...");
    println!("  Code filters:");
    println!("    - Removing tests: {}", config.filter_config.remove_tests);
    println!("    - Removing comments: {}", config.filter_config.remove_comments);
    println!("    - Keeping doc comments: {}", !config.filter_config.remove_doc_comments);
    println!("    - Removing debug prints: {}", config.filter_config.remove_debug_prints);
    println!();

    // Run the pipeline
    let stats = llm_utl::run(config)?;

    stats.print_summary();

    Ok(())
}