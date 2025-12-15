# Contributing to llm-utl

Thank you for your interest in contributing to llm-utl! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and constructive in all interactions. We're here to build something useful together.

## How to Contribute

### Reporting Bugs

If you find a bug, please create an issue with:
- A clear, descriptive title
- Steps to reproduce the problem
- Expected behavior
- Actual behavior
- Your environment (OS, Rust version, etc.)
- Any relevant code samples or error messages

### Suggesting Enhancements

We welcome feature requests! Please create an issue describing:
- The problem you're trying to solve
- Your proposed solution
- Any alternative solutions you've considered
- Additional context or examples

### Pull Requests

1. **Fork the repository** and create your branch from `master`
2. **Write clear commit messages** describing what changed and why
3. **Add tests** for any new functionality
4. **Update documentation** if you're changing public APIs
5. **Run the test suite** with `cargo test`
6. **Run clippy** with `cargo clippy` to catch common mistakes
7. **Format your code** with `cargo fmt`
8. **Submit your PR** with a clear description of the changes

## Development Setup

```bash
# Clone your fork
git clone https://github.com/your-username/llm-utl.git
cd llm-utl

# Build the project
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- --help

# Run examples
cargo run --example basic
```

## Project Structure

```
llm-utl/
├── src/
│   ├── main.rs         # CLI entry point
│   ├── lib.rs          # Library entry point
│   ├── config.rs       # Configuration builder
│   ├── pipeline.rs     # Main pipeline orchestrator
│   ├── scanner.rs      # File discovery
│   ├── filter.rs       # Code filtering
│   ├── splitter.rs     # Chunking algorithm
│   ├── token.rs        # Token estimation
│   ├── writer.rs       # Output rendering
│   ├── file.rs         # File data structures
│   ├── error.rs        # Error types
│   ├── template.rs     # Template handling
│   └── preset.rs       # Configuration presets
├── templates/          # Tera templates
├── examples/           # Usage examples
└── tests/              # Integration tests
```

## Coding Guidelines

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common mistakes
- Prefer explicit types in public APIs
- Document all public items with doc comments

### Documentation

- All public functions, structs, and traits must have doc comments
- Include examples in doc comments where helpful
- Explain the "why" not just the "what"
- Update README.md if adding significant features

### Testing

- Write unit tests for new functionality
- Add integration tests for end-to-end scenarios
- Use property-based testing (proptest) for algorithms
- Ensure tests pass before submitting PR

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // Your test here
    }
}
```

### Error Handling

- Use `thiserror` for library errors
- Use `anyhow` for application errors
- Provide helpful error messages
- Include context with `.context()` when appropriate

### Performance

- Avoid unnecessary allocations
- Use `Arc` for shared data in parallel code
- Stream large files instead of loading into memory
- Profile before optimizing

## Adding Language Support

To add support for a new programming language:

1. Add a new filter in `src/filter.rs`:
   ```rust
   struct NewLanguageFilter<'a> {
       config: &'a FilterConfig,
   }

   impl<'a> LanguageFilter for NewLanguageFilter<'a> {
       // Implement trait methods
   }
   ```

2. Add the language to `CodeFilter::filter()` match statement
3. Add tests for the new filter
4. Update documentation

## Adding Output Formats

To add a new output format:

1. Add variant to `OutputFormat` enum in `config.rs`
2. Create template in `templates/` directory
3. Update `Writer` to handle the new format
4. Add tests
5. Update documentation and examples

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` with changes
3. Create git tag: `git tag -a v0.x.0 -m "Release v0.x.0"`
4. Push tag: `git push origin v0.x.0`
5. Publish to crates.io: `cargo publish`

## Questions?

Feel free to open an issue with your question or reach out to the maintainers.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.