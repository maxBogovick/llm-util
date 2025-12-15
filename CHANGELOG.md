# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2024-12-15

### Added
- Initial release of llm-utl (llm-utl)
- Parallel file scanning with gitignore support
- Smart chunking algorithm with configurable token limits
- Multiple output formats: Markdown, XML, JSON
- Language-specific code filters for:
  - Rust
  - Python
  - JavaScript/TypeScript (including JSX/TSX)
  - Go
  - Java/Kotlin
  - C/C++
- Code filtering features:
  - Remove test code
  - Remove comments and doc comments
  - Remove debug print statements
  - Remove blank lines
  - Preserve license headers
- File filtering with glob patterns:
  - Blacklist specific files
  - Blacklist directories
  - Whitelist mode
- Two tokenizer implementations:
  - Simple: ~4 characters per token
  - Enhanced: Word count + character analysis
- Streaming mode for large files (>10MB)
- Atomic file writes with tempfile
- Comprehensive statistics and metrics
- Dry-run mode for previewing output
- Configurable logging with tracing
- CLI tool with rich options
- Library API with builder pattern configuration

### Features
- 🚀 High performance with parallel processing
- 🧹 Automatic code cleanup and noise reduction
- 📊 Detailed pipeline statistics
- 💾 Safe file operations
- 🔍 Respects .gitignore files
- 🎯 Smart chunk overlap for context preservation

[Unreleased]: https://github.com/yourusername/llm-utl/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/llm-utl/releases/tag/v0.1.0