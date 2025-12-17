# llm-utl

[![Crates.io](https://img.shields.io/crates/v/llm-utl.svg)](https://crates.io/crates/llm-utl)
[![Documentation](https://docs.rs/llm-utl/badge.svg)](https://docs.rs/llm-utl)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Transform code repositories into LLM-friendly prompts with intelligent chunking and filtering. Convert your codebase into optimally-chunked, formatted prompts ready for use with Large Language Models like Claude, GPT-4, or other AI assistants.

## Features

- 🚀 **Zero-config** - Works out of the box with sensible defaults
- 🎯 **Type-safe API** - Fluent, compile-time checked interface with presets
- 📦 **Smart Chunking** - Automatically splits large codebases into optimal token-sized chunks with overlap
- 🔧 **Presets** - Optimized configurations for common tasks (code review, documentation, security audit)
- 🧹 **Code Filtering** - Removes tests, comments, debug prints, and other noise from code
- 🎨 **Multiple Formats** - Output to Markdown, XML, or JSON
- ⚡ **Fast** - Parallel file scanning with multi-threaded processing (~1000 files/second)
- 🔍 **Gitignore Support** - Respects `.gitignore` files automatically
- 🌍 **Multi-Language** - Built-in filters for Rust, Python, JavaScript/TypeScript, Go, Java, C/C++
- 🛡️ **Robust** - Comprehensive error handling with atomic file writes

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
llm-utl

# Specify input and output directories
llm-utl --dir ./src --out ./prompts

# Configure token limits and format
llm-utl --max-tokens 50000 --format xml

# Dry run to preview what would be generated
llm-utl --dry-run
```

All options:
```bash
llm-utl [OPTIONS]

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

#### Simple API (Recommended)

The `Scan` API provides a fluent, type-safe interface:

```rust
use llm_utl::Scan;

// Simplest usage - scan current directory
llm_utl::scan()?;

// Scan specific directory
Scan::dir("./src").run()?;

// Use a preset for common tasks
Scan::dir("./src")
    .code_review()
    .run()?;

// Custom configuration
Scan::dir("./project")
    .output("./prompts")
    .max_tokens(200_000)
    .format(Format::Json)
    .keep_tests()
    .run()?;
```

#### Using Presets

Presets provide optimized configurations for specific tasks:

```rust
use llm_utl::Scan;

// Code review - removes tests, comments, debug prints
Scan::dir("./src")
    .code_review()
    .run()?;

// Documentation - keeps all comments and docs
Scan::dir("./project")
    .documentation()
    .run()?;

// Security audit - includes everything
Scan::dir("./src")
    .security_audit()
    .run()?;

// Bug analysis - focuses on logic
Scan::dir("./src")
    .bug_analysis()
    .run()?;
```

#### Advanced API

For complex scenarios, use the full `Pipeline` API:

```rust
use llm_utl::{Config, Pipeline, OutputFormat};

fn main() -> anyhow::Result<()> {
    let config = Config::builder()
        .root_dir("./src")
        .output_dir("./prompts")
        .format(OutputFormat::Markdown)
        .max_tokens(100_000)
        .overlap_tokens(1_000)
        .build()?;

    let stats = Pipeline::new(config)?.run()?;

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
            // Or whitelist specific files (use glob patterns with **/):
            // .allow_only(vec!["**/*.rs".to_string(), "**/*.toml".to_string()])
    )
    .build()?;
```

**Important**: When using `.allow_only()`, use glob patterns like `**/*.rs` instead of `*.rs` to match files in all subdirectories. The pattern `*.rs` only matches files in the root directory.

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

## Working with Statistics

The `PipelineStats` struct provides detailed information about the scanning process:

```rust
let stats = Scan::dir("./src").run()?;

// File counts
println!("Total files: {}", stats.total_files);
println!("Text files: {}", stats.text_files);
println!("Binary files: {}", stats.binary_files);

// Chunks
println!("Total chunks: {}", stats.total_chunks);
println!("Avg chunk size: {} tokens", stats.avg_tokens_per_chunk);
println!("Max chunk size: {} tokens", stats.max_chunk_tokens);

// Performance
println!("Duration: {:.2}s", stats.duration.as_secs_f64());
println!("Throughput: {:.0} tokens/sec",
    stats.throughput_tokens_per_sec()
);

// Output
println!("Output directory: {}", stats.output_directory);
println!("Files written: {}", stats.files_written);
```

## Design Philosophy

### Progressive Disclosure

Start simple, add complexity only when needed:

1. **Level 1**: `llm_utl::scan()` - Zero config, works immediately
2. **Level 2**: `Scan::dir("path").code_review()` - Use presets for common tasks
3. **Level 3**: `Scan::dir().keep_tests().exclude([...])` - Fine-grained control
4. **Level 4**: Full `Config` API - Maximum flexibility

### Type Safety

All options are compile-time checked:

```rust
// This won't compile - caught at compile time
Scan::dir("./src")
    .format("invalid");  // Error: expected Format enum

// Correct usage
Scan::dir("./src")
    .format(Format::Json);
```

### Sensible Defaults

Works well without configuration:
- Excludes common directories (`node_modules`, `target`, `.git`, etc.)
- Removes noise (tests, comments, debug prints)
- Uses efficient token limits (100,000 per chunk)
- Provides clear, actionable error messages

### Fluent Interface

Natural, readable API:

```rust
Scan::dir("./src")
    .code_review()
    .output("./review")
    .max_tokens(200_000)
    .keep_tests()
    .run()?;
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

## Real-World Examples

### Pre-commit Review

```rust
use llm_utl::Scan;

fn pre_commit_hook() -> llm_utl::Result<()> {
    println!("🔍 Analyzing changes...");

    let stats = Scan::dir("./src")
        .code_review()
        .output("./review")
        .run()?;

    println!("✓ Review ready in {}", stats.output_directory);
    Ok(())
}
```

### CI/CD Security Scan

```rust
use llm_utl::Scan;

fn ci_security_check() -> llm_utl::Result<()> {
    let stats = Scan::dir("./src")
        .security_audit()
        .output("./security-reports")
        .max_tokens(120_000)
        .run()?;

    if stats.total_files == 0 {
        eprintln!("❌ No files to scan");
        std::process::exit(1);
    }

    println!("✓ Scanned {} files", stats.total_files);
    Ok(())
}
```

### Documentation Generation

```rust
use llm_utl::Scan;

fn generate_docs() -> llm_utl::Result<()> {
    Scan::dir(".")
        .documentation()
        .output("./docs/ai-generated")
        .run()?;

    Ok(())
}
```

### Batch Processing

```rust
use llm_utl::Scan;

fn process_multiple_projects() -> llm_utl::Result<()> {
    for project in ["./frontend", "./backend", "./mobile"] {
        println!("Processing {project}...");

        match Scan::dir(project).run() {
            Ok(stats) => println!("  ✓ {} files", stats.total_files),
            Err(e) => eprintln!("  ✗ Error: {e}"),
        }
    }
    Ok(())
}
```

## More Examples

See the `https://github.com/maxBogovick/llm-util/tree/master/examples` directory for more usage examples.

## Development

```bash
# Clone the repository
git clone https://github.com/maxBogovick/llm-util.git
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

## Troubleshooting

### "No processable files found" Error

If you see this error:
```
Error: No processable files found in '.'.
```

**Common causes:**

1. **Wrong directory**: The tool is running in an empty directory or a directory without source files.
   ```bash
   # ❌ Wrong - running in home directory
   cd ~
   llm-utl

   # ✅ Correct - specify your project directory
   llm-utl --dir ./my-project
   ```

2. **All files are gitignored**: Your `.gitignore` excludes all files in the directory.
   ```bash
   # Check what files would be scanned
   llm-utl --dir ./my-project --dry-run -v
   ```

3. **No source files**: The directory contains only non-source files (images, binaries, etc.).
   ```bash
   # Make sure directory contains code files
   ls ./my-project/*.rs  # or *.py, *.js, etc.
   ```

**Quick fix:**
```bash
# Always specify the directory containing your source code
llm-utl --dir ./path/to/your/project --out ./prompts
```

### Permission Issues

If you encounter permission errors:
```bash
# Ensure you have read access to source directory
# and write access to output directory
chmod +r ./src
chmod +w ./out
```

### Large Files

If processing is slow with very large files:
```bash
# Increase token limit for large codebases
llm-utl --max-tokens 200000

# Or use simple tokenizer for better performance
llm-utl --tokenizer simple
```

## FAQ

### How do I scan only specific file types?

Use the `Scan` API with exclusion patterns or the full `Config` API with custom file filters:

```rust
use llm_utl::{Config, FileFilterConfig};

Config::builder()
    .root_dir("./src")
    .file_filter_config(
        FileFilterConfig::default()
            .allow_only(vec!["**/*.rs".to_string(), "**/*.toml".to_string()])
    )
    .build()?
    .run()?;
```

### How do I handle very large codebases?

Increase token limits and adjust overlap:

```rust
Scan::dir("./large-project")
    .max_tokens(500_000)
    .overlap(5_000)
    .run()?;
```

### Can I process multiple directories?

Yes, scan each separately or use a common parent:

```rust
for dir in ["./src", "./lib", "./bin"] {
    Scan::dir(dir)
        .output(&format!("./out/{}", dir.trim_start_matches("./")))
        .run()?;
}
```

### How do I preserve everything for analysis?

Use the security audit preset or configure manually:

```rust
// Using preset
Scan::dir("./src")
    .security_audit()
    .run()?;

// Manual configuration
Scan::dir("./src")
    .keep_tests()
    .keep_comments()
    .keep_doc_comments()
    .keep_debug_prints()
    .run()?;
```

### What are the available presets?

The library provides these presets:

- **code_review** - Removes tests, comments, debug prints for clean code review
- **documentation** - Preserves all documentation and comments
- **security_audit** - Includes everything for comprehensive security analysis
- **bug_analysis** - Focuses on logic by removing noise
- **refactoring** - Optimized for refactoring tasks
- **test_generation** - Configured for generating tests

## Platform Support

- ✓ Linux
- ✓ macOS
- ✓ Windows

All major platforms are supported and tested.

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