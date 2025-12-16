# LLM Presets Guide

This document describes the available LLM presets and how to use them for specialized code analysis tasks.

## What are Presets?

Presets are pre-configured templates that optimize the output format and content for specific LLM tasks. Each preset includes:
- Optimized system prompt
- Task-specific user prompt template
- Recommended model and parameters
- Metadata and structure configuration

## Available Presets

### 1. Code Review (`PresetKind::CodeReview`)

Comprehensive code review focusing on:
- Code quality and maintainability
- Performance issues
- Security vulnerabilities
- Best practices
- Potential bugs and edge cases

**Suggested Configuration:**
```rust
use llm_utl::{Config, PresetKind, FilterConfig};

let config = Config::builder()
    .root_dir("./src")
    .preset(PresetKind::CodeReview)
    .max_tokens(150_000)
    .filter_config(FilterConfig {
        remove_tests: true,
        remove_doc_comments: false,  // Keep docs for review
        remove_comments: false,       // Keep comments for context
        remove_blank_lines: true,
        preserve_headers: true,
        remove_debug_prints: true,
    })
    .build()?;
```

### 2. Documentation Generation (`PresetKind::Documentation`)

Generate comprehensive project documentation including:
- Project overview
- Architecture explanation
- API documentation
- Usage examples

**Suggested Configuration:**
```rust
let config = Config::builder()
    .root_dir("./src")
    .preset(PresetKind::Documentation)
    .max_tokens(100_000)
    .filter_config(FilterConfig::preserve_docs())
    .build()?;
```

### 3. Refactoring (`PresetKind::Refactoring`)

Get refactoring recommendations for:
- Code duplication removal
- Design pattern applications
- Improved abstractions
- Better naming conventions

**Suggested Configuration:**
```rust
let config = Config::builder()
    .root_dir("./src")
    .preset(PresetKind::Refactoring)
    .max_tokens(120_000)
    .filter_config(FilterConfig::minimal())
    .build()?;
```

### 4. Bug Analysis (`PresetKind::BugAnalysis`)

Identify potential bugs:
- Null pointer/undefined access
- Race conditions
- Memory leaks
- Off-by-one errors
- Unhandled edge cases

**Suggested Configuration:**
```rust
let config = Config::builder()
    .root_dir("./src")
    .preset(PresetKind::BugAnalysis)
    .max_tokens(100_000)
    .build()?;
```

### 5. Security Audit (`PresetKind::SecurityAudit`)

Comprehensive security assessment for:
- SQL injection
- XSS vulnerabilities
- Authentication/authorization flaws
- Insecure data storage
- Cryptographic weaknesses

**Suggested Configuration:**
```rust
let config = Config::builder()
    .root_dir("./src")
    .preset(PresetKind::SecurityAudit)
    .max_tokens(120_000)
    .filter_config(FilterConfig::production())
    .build()?;
```

### 6. Test Generation (`PresetKind::TestGeneration`)

Generate comprehensive test suites:
- Unit tests for all functions
- Integration tests
- Edge case tests
- Mock/stub suggestions

**Suggested Configuration:**
```rust
let config = Config::builder()
    .root_dir("./src")
    .preset(PresetKind::TestGeneration)
    .max_tokens(150_000)
    .filter_config(FilterConfig {
        remove_tests: true,  // Remove existing tests
        ..Default::default()
    })
    .build()?;
```

### 7. Architecture Review (`PresetKind::ArchitectureReview`)

Evaluate system architecture:
- Component relationships
- Design patterns usage
- Scalability considerations
- Technology choices

**Suggested Configuration:**
```rust
let config = Config::builder()
    .root_dir("./src")
    .preset(PresetKind::ArchitectureReview)
    .max_tokens(100_000)
    .build()?;
```

### 8. Performance Analysis (`PresetKind::PerformanceAnalysis`)

Identify performance bottlenecks:
- Algorithm complexity (Big O)
- Memory usage patterns
- I/O operations
- Caching opportunities
- Parallelization potential

**Suggested Configuration:**
```rust
let config = Config::builder()
    .root_dir("./src")
    .preset(PresetKind::PerformanceAnalysis)
    .max_tokens(120_000)
    .build()?;
```

### 9. Migration Plan (`PresetKind::MigrationPlan`)

Create technology migration plans:
- Current state analysis
- Step-by-step migration path
- Risk assessment
- Testing approach

**Suggested Configuration:**
```rust
let config = Config::builder()
    .root_dir("./src")
    .preset(PresetKind::MigrationPlan)
    .max_tokens(100_000)
    .build()?;
```

### 10. API Design (`PresetKind::ApiDesign`)

Review and improve API design:
- RESTful principles
- Consistency
- Error handling
- Versioning strategy

**Suggested Configuration:**
```rust
let config = Config::builder()
    .root_dir("./src")
    .preset(PresetKind::ApiDesign)
    .max_tokens(100_000)
    .build()?;
```

## Preset Details

You can programmatically access preset details:

```rust
use llm_utl::{PresetKind, LLMPreset};

// Get preset details
let preset = LLMPreset::for_kind(PresetKind::CodeReview);

println!("Name: {}", preset.name);
println!("Description: {}", preset.description);
println!("Suggested Model: {}", preset.suggested_model);
println!("Max Tokens Hint: {}", preset.max_tokens_hint);
println!("Temperature Hint: {}", preset.temperature_hint);

// List all available presets
for preset_kind in PresetKind::all() {
    println!("- {} (id: {})",
        format!("{:?}", preset_kind),
        preset_kind.id()
    );
}
```

## Examples

See the `examples/` directory for complete working examples:

- `examples/preset_usage.rs` - Basic preset usage
- `examples/preset_pipeline.rs` - Full pipeline with presets
- `examples/simple_preset.rs` - Minimal example for quick start

Run examples:
```bash
cargo run --example simple_preset
cargo run --example preset_usage
cargo run --example preset_pipeline
```

## Best Practices

1. **Match preset to task**: Choose the preset that best matches your analysis goals
2. **Configure filters appropriately**: Use `FilterConfig` to control what code is included
3. **Adjust token limits**: Different presets suggest different token limits based on task complexity
4. **Use appropriate output format**: Some presets work better with specific formats (Markdown, JSON, XML)
5. **Review preset prompts**: Check `preset.system_prompt` and `preset.user_prompt_template` to understand what the LLM will analyze

## Customization

While presets provide sensible defaults, you can customize:
- Token limits
- Filter configurations
- Output formats
- File inclusion/exclusion patterns

Example of customized preset:
```rust
let config = Config::builder()
    .root_dir("./src")
    .preset(PresetKind::CodeReview)
    .max_tokens(200_000)  // Custom limit
    .format(OutputFormat::Json)  // Custom format
    .filter_config(FilterConfig {
        // Custom filter settings
        remove_tests: false,  // Include tests in review
        ..Default::default()
    })
    .file_filter_config(
        FileFilterConfig::default()
            .exclude_directories(vec!["**/vendor".to_string()])
    )
    .build()?;
```