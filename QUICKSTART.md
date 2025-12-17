# llm-utl

[![Crates.io](https://img.shields.io/crates/v/llm-utl.svg)](https://crates.io/crates/llm-utl)
[![Documentation](https://docs.rs/llm-utl/badge.svg)](https://docs.rs/llm-utl)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Transform code repositories into LLM-friendly prompts with intelligent chunking and filtering.

## Features

- 🚀 **Zero-config**: Works out of the box with sensible defaults
- 🎯 **Type-safe API**: Fluent, compile-time checked interface
- 📦 **Smart chunking**: Automatic token-aware file splitting
- 🔧 **Presets**: Optimized configurations for common tasks
- 🎨 **Multiple formats**: Markdown, XML, and JSON output
- ⚡ **Fast**: Parallel processing with efficient I/O
- 🛡️ **Robust**: Comprehensive error handling

## Quick Start

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

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
llm-utl = "0.1"
```

## Usage

### Basic Scanning

```rust
use llm_utl::{Scan, Result};

fn main() -> Result<()> {
    // Scan with defaults
    let stats = Scan::dir("./src").run()?;
    
    println!("Processed {} files in {:.2}s",
        stats.total_files,
        stats.duration.as_secs_f64()
    );
    
    Ok(())
}
```

### Using Presets

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

### Custom Configuration

```rust
use llm_utl::{Format, Scan};

Scan::dir("./src")
    .output("./prompts")
    .format(Format::Json)
    .max_tokens(150_000)
    .overlap(2_000)
    .keep_tests()
    .keep_comments()
    .exclude(["**/vendor", "**/generated"])
    .run()?;
```

## API Overview

### Starting a Scan

```rust
// Current directory
Scan::current_dir()

// Specific directory
Scan::dir("./path")

// Convenience functions
llm_utl::scan()              // Current dir
llm_utl::scan_dir("./path")  // Specific dir
```

### Configuration Methods

| Method | Description | Default |
|--------|-------------|---------|
| `.output(path)` | Set output directory | `./out` |
| `.format(fmt)` | Set output format | `Markdown` |
| `.max_tokens(n)` | Max tokens per file | `100_000` |
| `.overlap(n)` | Overlap between chunks | `1_000` |
| `.preset(p)` | Use a preset config | None |

### Filtering Options

| Method | Description |
|--------|-------------|
| `.keep_tests()` | Include test files |
| `.remove_tests()` | Exclude test files (default) |
| `.keep_comments()` | Include comments |
| `.remove_comments()` | Exclude comments (default) |
| `.keep_doc_comments()` | Include documentation |
| `.remove_doc_comments()` | Exclude docs (default) |
| `.keep_debug_prints()` | Include debug statements |
| `.remove_debug_prints()` | Exclude debug (default) |

### Preset Methods

Shortcuts for common configurations:

```rust
.code_review()        // Code review preset
.documentation()      // Documentation preset
.security_audit()     // Security audit preset
.bug_analysis()       // Bug analysis preset
.refactoring()        // Refactoring preset
.test_generation()    // Test generation preset
```

### Exclusion Patterns

```rust
.exclude(["**/node_modules", "**/target"])
.exclude(["**/dist/**", "**/*.test.js"])
```

Patterns support glob syntax:
- `**` matches any number of directories
- `*` matches any characters except `/`
- `?` matches a single character

## Output Formats

### Markdown (Default)

Clean, readable format suitable for direct LLM input:

```rust
Scan::dir("./src")
    .format(Format::Markdown)
    .run()?;
```

### JSON

Structured format for programmatic processing:

```rust
Scan::dir("./src")
    .format(Format::Json)
    .run()?;
```

### XML

Hierarchical format with metadata:

```rust
Scan::dir("./src")
    .format(Format::Xml)
    .run()?;
```

## Presets in Detail

### Code Review

Optimized for reviewing code structure and logic:
- ✓ Removes tests
- ✓ Removes comments
- ✓ Removes debug prints
- ✓ Clean, focused output

```rust
Scan::dir("./src").code_review().run()?;
```

### Documentation

Preserves all documentation for analysis:
- ✓ Keeps doc comments
- ✓ Keeps inline comments
- ✗ Removes tests

```rust
Scan::dir("./project").documentation().run()?;
```

### Security Audit

Comprehensive view for security analysis:
- ✓ Includes tests
- ✓ Includes all comments
- ✓ Includes debug prints
- ✓ Maximum context

```rust
Scan::dir("./src").security_audit().run()?;
```

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

## Working with Statistics

The `PipelineStats` struct provides detailed information:

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

## Error Handling

The library uses a comprehensive error type:

```rust
use llm_utl::Scan;

match Scan::dir("./project").run() {
    Ok(stats) => {
        println!("Success: {} files", stats.total_files);
    }
    Err(e) => {
        eprintln!("Error: {e}");
        
        // Type-safe error inspection
        if e.is_config() {
            eprintln!("Configuration error");
        } else if e.is_io() {
            eprintln!("I/O error");
        }
    }
}
```

## Advanced Usage

For complex scenarios, use the full `Config` API:

```rust
use llm_utl::{Config, FilterConfig, FileFilterConfig};

let config = Config::builder()
    .root_dir("./src")
    .output_dir("./out")
    .max_tokens(150_000)
    .filter_config(FilterConfig {
        remove_tests: true,
        remove_comments: true,
        remove_doc_comments: false,
        remove_blank_lines: true,
        preserve_headers: true,
        remove_debug_prints: true,
    })
    .file_filter_config(
        FileFilterConfig::default()
            .exclude_directories(vec!["**/vendor".to_string()])
    )
    .build()?;

llm_utl::run(config)?;
```

## Design Philosophy

### Progressive Disclosure

Start simple, add complexity only when needed:

1. **Level 1**: `llm_utl::scan()`
2. **Level 2**: `Scan::dir("path").preset(Preset::CodeReview)`
3. **Level 3**: `Scan::dir().keep_tests().exclude([...])`
4. **Level 4**: Full `Config` API

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
- Excludes common directories (`node_modules`, `target`, etc.)
- Removes noise (tests, comments, debug prints)
- Uses efficient token limits
- Provides clear error messages

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

## Performance

- **Parallel scanning**: Uses all CPU cores
- **Streaming I/O**: Handles large files efficiently
- **Smart chunking**: Minimizes redundancy
- **Fast tokenization**: Optimized token estimation

Typical performance:
- ~1000 files/second for scanning
- ~100,000 tokens/second for processing
- Minimal memory footprint

## Platform Support

- ✓ Linux
- ✓ macOS  
- ✓ Windows

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md).

## Related Projects

- [tokei](https://github.com/XAMPPRocky/tokei) - Code statistics
- [loc](https://github.com/cgag/loc) - Lines of code counter
- [scc](https://github.com/boyter/scc) - Sloc Cloc and Code

## FAQ

### How do I scan only specific file types?

Use the full `Config` API with custom file filters:

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
Scan::dir("./src")
    .security_audit()
    .run()?;

// Or manually:
Scan::dir("./src")
    .keep_tests()
    .keep_comments()
    .keep_doc_comments()
    .keep_debug_prints()
    .run()?;
```
