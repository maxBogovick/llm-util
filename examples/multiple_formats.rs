//! Multiple output formats example
//!
//! This example demonstrates generating output in all available formats.

use llm_utl::{Config, OutputFormat};

fn main() -> anyhow::Result<()> {
    let formats = [
        ("Markdown", OutputFormat::Markdown),
        ("XML", OutputFormat::Xml),
        ("JSON", OutputFormat::Json),
    ];

    for (name, format) in formats {
        println!("Generating {} format...", name);

        let config = Config::builder()
            .root_dir("./src")
            .output_dir(format!("./output_{}", name.to_lowercase()))
            .format(format)
            .max_tokens(100_000)
            .build()?;

        let stats = llm_utl::run(config)?;

        println!("  ✓ Created {} chunks in {:.2}s",
            stats.total_chunks,
            stats.duration.as_secs_f64()
        );
        println!();
    }

    println!("All formats generated successfully!");

    Ok(())
}