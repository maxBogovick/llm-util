# llm-utl (repo-to-prompt)

[![Crates.io](https://img.shields.io/crates/v/llm-utl.svg)](https://crates.io/crates/llm-utl)
[![Documentation](https://docs.rs/llm-utl/badge.svg)](https://docs.rs/llm-utl)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance Rust tool for converting code repositories into LLM-friendly prompts. Transform your codebase into optimally-chunked, formatted prompts ready for use with Large Language Models like Claude, GPT-4, or other AI assistants.

## Features

- 🚀 **Blazingly Fast** - Parallel file scanning with multi-threaded processing
- 🎯 **Smart Chunking** - Automatically splits large codebases into optimal token-sized chunks with overlap
- 🧹 **Code Filtering** - Removes tests, comments, debug prints, and other noise from code
- 📝 **Multiple Formats** - Output to Markdown, XML, or JSON
- 🔍 **Gitignore Support** - Respects `.gitignore` files automatically
- 🌍 **Multi-Language** - Built-in filters for Rust, Python, JavaScript/TypeScript, Go, Java, C/C++
- 💾 **Safe Operations** - Atomic file writes with automatic backups
- 📊 **Statistics** - Detailed metrics on processing and token usage

## Installation

### As a CLI Tool

```bash
cargo install llm-utl
```

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
llm-utl = "0.1.0"
```

## Quick Start

### Command Line Usage

Basic usage:
```bash
# Convert current directory to prompts
repo-to-prompt

# Specify input and output directories
repo-to-prompt --dir ./src --out ./prompts

# Configure token limits and format
repo-to-prompt --max-tokens 50000 --format xml

# Dry run to preview what would be generated
repo-to-prompt --dry-run
```

All options:
```bash
repo-to-prompt [OPTIONS]

Options:
  -d, --dir <DIR>              Root directory to scan [default: .]
  -o, --out <OUT>              Output directory [default: out]
      --pattern <PATTERN>      Output filename pattern [default: prompt_{index:03}.{ext}]
  -f, --format <FORMAT>        Output format [default: markdown] [possible values: markdown, xml, json]
      --max-tokens <TOKENS>    Max tokens per chunk [default: 100000]
      --overlap <TOKENS>       Overlap tokens between chunks [default: 1000]
      --tokenizer <TOKENIZER>  Tokenizer to use [default: enhanced] [possible values: simple, enhanced]
      --dry-run               Dry run (don't write files)
  -v, --verbose               Verbose output (use -vv for trace level)
  -h, --help                  Print help
  -V, --version               Print version
```

### Library Usage

```rust
use llm_utl::{Config, Pipeline, OutputFormat};

fn main() -> anyhow::Result<()> {
    // Configure the pipeline
    let config = Config::builder()
        .root_dir("./src")
        .output_dir("./prompts")
        .format(OutputFormat::Markdown)
        .max_tokens(100_000)
        .overlap_tokens(1_000)
        .build()?;

    // Run the conversion pipeline
    let stats = Pipeline::new(config)?.run()?;

    // Print results
    stats.print_summary();
    println!("Processed {} files into {} chunks",
        stats.total_files,
        stats.total_chunks
    );

    Ok(())
}
```

## Advanced Configuration

### Code Filtering

Control what gets removed from your code:

```rust
use llm_utl::{Config, FilterConfig};

let config = Config::builder()
    .root_dir(".")
    .filter_config(FilterConfig {
        remove_tests: true,
        remove_doc_comments: false,  // Keep documentation
        remove_comments: true,
        remove_blank_lines: true,
        preserve_headers: true,
        remove_debug_prints: true,   // Remove println!, dbg!, etc.
    })
    .build()?;
```

Or use presets:

```rust
use llm_utl::FilterConfig;

// Minimal - remove everything except code
let minimal = FilterConfig::minimal();

// Preserve docs - keep documentation comments
let with_docs = FilterConfig::preserve_docs();

// Production - ready for production review
let production = FilterConfig::production();
```

### File Filtering

Include or exclude specific files and directories:

```rust
use llm_utl::{Config, FileFilterConfig};

let config = Config::builder()
    .root_dir(".")
    .file_filter_config(
        FileFilterConfig::default()
            .exclude_directories(vec![
                "**/target".to_string(),
                "**/node_modules".to_string(),
                "**/.git".to_string(),
            ])
            .exclude_files(vec!["*.lock".to_string()])
            // Or whitelist specific files:
            // .allow_only(vec!["src/**/*.rs".to_string()])
    )
    .build()?;
```

### Custom Tokenizers

Choose between simple and enhanced tokenization:

```rust
use llm_utl::{Config, TokenizerKind};

let config = Config::builder()
    .root_dir(".")
    .tokenizer(TokenizerKind::Enhanced)  // More accurate
    // .tokenizer(TokenizerKind::Simple) // Faster, ~4 chars per token
    .build()?;
```

## Output Formats

### Markdown (Default)

```markdown
# Chunk 1/3 (45,234 tokens)

## File: src/main.rs (1,234 tokens)

```rust
fn main() {
    println!("Hello, world!");
}
```
```

### XML

```xml
<?xml version="1.0" encoding="UTF-8"?>
<chunk index="1" total="3">
  <file path="src/main.rs" tokens="1234">
    <![CDATA[
fn main() {
    println!("Hello, world!");
}
    ]]>
  </file>
</chunk>
```

### JSON

```json
{
  "chunk_index": 1,
  "total_chunks": 3,
  "total_tokens": 45234,
  "files": [
    {
      "path": "src/main.rs",
      "tokens": 1234,
      "content": "fn main() {\n    println!(\"Hello, world!\");\n}"
    }
  ]
}
```

## Use Cases

- 📖 **Code Review with AI** - Feed your codebase to Claude or GPT-4 for comprehensive reviews
- 🎓 **Learning** - Generate study materials from large codebases
- 📚 **Documentation** - Create AI-friendly documentation sources
- 🔍 **Analysis** - Prepare code for AI-powered analysis and insights
- 🤖 **Training Data** - Generate datasets for fine-tuning models

## How It Works

The tool follows a 4-stage pipeline:

1. **Scanner** - Discovers files in parallel, respecting `.gitignore`
2. **Filter** - Removes noise (tests, comments, debug statements) using language-specific filters
3. **Splitter** - Intelligently chunks content based on token limits with overlap for context
4. **Writer** - Renders chunks using Tera templates with atomic file operations

## Performance

- Parallel file scanning using all CPU cores
- Streaming mode for large files (>10MB)
- Zero-copy operations where possible
- Optimized for minimal allocations

Typical performance: **~1000 files/second** on modern hardware.

## Supported Languages

Built-in filtering support for:
- Rust
- Python
- JavaScript/TypeScript (including JSX/TSX)
- Go
- Java/Kotlin
- C/C++

Other languages are processed as plain text.

## Examples

See the `examples/` directory for more usage examples:

```bash
cargo run --example basic
cargo run --example custom_config
cargo run --example advanced_filtering
```

## Development

```bash
# Clone the repository
git clone https://github.com/yourusername/llm-utl.git
cd llm-utl

# Build
cargo build --release

# Run tests
cargo test

# Run with verbose logging
RUST_LOG=llm_utl=debug cargo run -- --dir ./src

# Format code
cargo fmt

# Lint
cargo clippy
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Built with these excellent crates:
- [ignore](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore) - Fast gitignore-aware file walking
- [tera](https://github.com/Keats/tera) - Powerful template engine
- [clap](https://github.com/clap-rs/clap) - CLI argument parsing
- [tracing](https://github.com/tokio-rs/tracing) - Structured logging

## See Also

- [API Documentation](https://docs.rs/llm-utl)
- [Changelog](CHANGELOG.md)
- [Contributing Guidelines](CONTRIBUTING.md)