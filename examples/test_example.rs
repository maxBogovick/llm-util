use llm_utl::{Config, FilterConfig, FileFilterConfig, OutputFormat, Pipeline};

fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("Testing llm-utl with example configuration...\n");

    let config = Config::builder()
        .root_dir("./src")
        .output_dir("./test_out")
        .format(OutputFormat::Markdown)
        .max_tokens(300_000)
        .overlap_tokens(2_000)
        .filter_config(FilterConfig {
            remove_tests: true,
            remove_doc_comments: true,
            remove_comments: true,
            remove_blank_lines: true,
            preserve_headers: true,
            remove_debug_prints: true,
        })
        .file_filter_config(
            FileFilterConfig::default()
                .allow_only(vec!["**/*.rs".to_string()])
        )
        .build()?;

    println!("Configuration:");
    println!("  Root dir: {:?}", config.root_dir);
    println!("  Output dir: {:?}", config.output_dir);
    println!("  Max tokens: {}", config.max_tokens);
    println!();

    // Run the conversion pipeline
    let stats = Pipeline::new(config)?.run()?;

    // Print results
    stats.print_summary();
    println!(
        "Processed {} files into {} chunks",
        stats.total_files, stats.total_chunks
    );

    Ok(())
}