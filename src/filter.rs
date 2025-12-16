//! Code filtering and preprocessing module.
//!
//! Provides functionality to strip tests, comments, and documentation
//! from source code before generating prompts.

use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;

/// Configuration for file filtering with glob patterns.
///
/// Allows selective file and directory inclusion/exclusion during repository scanning.
#[derive(Debug, Clone, Default)]
pub struct FileFilterConfig {
    exclude_files: Vec<String>,
    exclude_all_files_except: Vec<String>,
    exclude_directories: Vec<String>,
}
impl FileFilterConfig {
    /// Создает новую пустую конфигурацию.
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавляет файлы в черный список.
    pub fn exclude_files(mut self, paths: Vec<String>) -> Self {
        self.exclude_files = paths;
        self
    }

    /// Добавляет директории в черный список.
    pub fn exclude_directories(mut self, paths: Vec<String>) -> Self {
        self.exclude_directories = paths;
        self
    }

    /// Устанавливает белый список файлов.
    pub fn allow_only(mut self, paths: Vec<String>) -> Self {
        self.exclude_all_files_except = paths;
        self
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FileFilter {
    exclude_files: GlobSet,
    include_files: Option<GlobSet>,
    exclude_directories: GlobSet,
}

impl FileFilter {
    /// Создает новый фильтр с заданной конфигурацией.
    pub(crate) fn new(config: FileFilterConfig) -> Self {
        let exclude_files = Self::build_globset(&config.exclude_files).unwrap();
        let exclude_directories = Self::build_globset(&config.exclude_directories).unwrap();

        let include_files = if config.exclude_all_files_except.is_empty() {
            None
        } else {
            Some(Self::build_globset(&config.exclude_all_files_except).unwrap())
        };

        Self {
            exclude_files,
            include_files,
            exclude_directories,
        }
    }

    fn build_globset(patterns: &[String]) -> Result<GlobSet, crate::error::Error> {
        let mut builder = GlobSetBuilder::new();

        for pattern in patterns {
            let glob = Glob::new(pattern).map_err(|e| {
                crate::error::Error::config(format!("Invalid glob pattern '{}': {}", pattern, e))
            })?;
            builder.add(glob);
        }

        builder.build().map_err(|e| {
            crate::error::Error::config(format!("Failed to build glob set: {}", e))
        })
    }

    pub(crate) fn should_process(&self, path: &Path) -> bool {
        // Проверка include patterns (если указаны)
        if let Some(ref include) = self.include_files {
            if !include.is_match(path) {
                return false;
            }
        }

        // Проверка exclude directories
        if self.exclude_directories.is_match(path) {
            return false;
        }

        // Проверка на вхождение в исключенные директории
        for ancestor in path.ancestors().skip(1) {
            if self.exclude_directories.is_match(ancestor) {
                return false;
            }
        }

        // Проверка exclude files
        if self.exclude_files.is_match(path) {
            return false;
        }

        true
    }
}
/// Configuration for code filtering operations.
#[derive(Debug, Clone)]
pub struct FilterConfig {
    /// Remove test code (e.g., #[test], #[cfg(test)])
    pub remove_tests: bool,

    /// Remove documentation comments (///, /** */)
    pub remove_doc_comments: bool,

    /// Remove regular comments (//, /* */)
    pub remove_comments: bool,

    /// Remove blank lines after filtering
    pub remove_blank_lines: bool,

    /// Preserve copyright/license headers
    pub preserve_headers: bool,

    /// Remove debug print statements (println!, dbg!, etc.)
    pub remove_debug_prints: bool,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            remove_tests: true,
            remove_doc_comments: false,
            remove_comments: false,
            remove_blank_lines: true,
            preserve_headers: true,
            remove_debug_prints: false,
        }
    }
}

impl FilterConfig {
    /// Creates a configuration that removes everything except code.
    #[must_use]
    pub fn minimal() -> Self {
        Self {
            remove_tests: true,
            remove_doc_comments: true,
            remove_comments: true,
            remove_blank_lines: true,
            preserve_headers: false,
            remove_debug_prints: true,
        }
    }

    /// Creates a configuration that keeps documentation.
    #[must_use]
    pub fn preserve_docs() -> Self {
        Self {
            remove_tests: true,
            remove_doc_comments: false,
            remove_comments: true,
            remove_blank_lines: true,
            preserve_headers: true,
            remove_debug_prints: false,
        }
    }

    /// Creates a configuration for production-ready code.
    #[must_use]
    pub fn production() -> Self {
        Self {
            remove_tests: true,
            remove_doc_comments: false,
            remove_comments: false,
            remove_blank_lines: true,
            preserve_headers: true,
            remove_debug_prints: true,
        }
    }
}

/// Main code filter that dispatches to language-specific filters.
#[derive(Debug, Clone)]
pub struct CodeFilter {
    config: FilterConfig,
}

impl CodeFilter {
    /// Creates a new code filter with the given configuration.
    #[must_use]
    pub const fn new(config: FilterConfig) -> Self {
        Self { config }
    }

    /// Filters code content based on file extension and configuration.
    ///
    /// Returns filtered content or original if no filtering applies.
    #[must_use]
    pub fn filter(&self, content: &str, path: &Path) -> String {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension {
            "rs" => RustFilter::new(&self.config).filter(content),
            "py" => PythonFilter::new(&self.config).filter(content),
            "js" | "ts" | "jsx" | "tsx" => JavaScriptFilter::new(&self.config).filter(content),
            "go" => GoFilter::new(&self.config).filter(content),
            "java" | "kt" => JavaFilter::new(&self.config).filter(content),
            "c" | "cpp" | "cc" | "h" | "hpp" => CFilter::new(&self.config).filter(content),
            _ => content.to_string(),
        }
    }
}

/// Trait for language-specific code filters.
trait LanguageFilter {
    /// Returns the filter configuration.
    #[allow(dead_code)]
    fn config(&self) -> &FilterConfig;

    /// Filters the content according to language rules.
    fn filter(&self, content: &str) -> String;

    /// Checks if a line is a comment.
    fn is_comment_line(&self, line: &str) -> bool;

    /// Checks if a line is a doc comment.
    fn is_doc_comment(&self, line: &str) -> bool;

    /// Removes comments from a line while preserving strings.
    /// Removes comments from a line while preserving strings.
    fn strip_line_comment(&self, line: &str, _comment_start: &str) -> String {
        let mut in_string = false;
        let mut escape_next = false;
        let chars: Vec<char> = line.chars().collect();

        for i in 0..chars.len() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match chars[i] {
                '\\' if in_string => {
                    escape_next = true;
                }
                '"' => {
                    in_string = !in_string;
                }
                '/' if !in_string && i + 1 < chars.len() && chars[i + 1] == '/' => {
                    // Found comment outside of string
                    return line[..i].trim_end().to_string();
                }
                _ => {}
            }
        }

        line.to_string()
    }
}

/// Rust-specific code filter.
struct RustFilter<'a> {
    config: &'a FilterConfig,
}

impl<'a> RustFilter<'a> {
    const fn new(config: &'a FilterConfig) -> Self {
        Self { config }
    }

    /// Checks if we're entering a test module or function.
    fn is_test_start(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("#[test]")
            || trimmed.starts_with("#[cfg(test)]")
            || trimmed.starts_with("#[tokio::test]")
            || trimmed.starts_with("#[async_test]")
    }

    /// Checks if a line contains test-related attributes.
    fn has_test_attribute(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.contains("#[test")
            || trimmed.contains("#[cfg(test")
            || trimmed.contains("#[should_panic")
            || trimmed.contains("#[ignore")
    }

    /// Checks if a line contains a debug print macro.
    fn is_debug_print(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("println!")
            || trimmed.starts_with("eprintln!")
            || trimmed.starts_with("dbg!")
            || trimmed.starts_with("print!")
            || trimmed.starts_with("eprint!")
            || trimmed.contains("println!(")
            || trimmed.contains("eprintln!(")
            || trimmed.contains("dbg!(")
    }

    /// Removes debug print statements from a line.
    /// Returns (processed_line, is_multiline_print)
    fn strip_debug_prints(&self, line: &str) -> (String, bool) {
        if !self.config.remove_debug_prints {
            return (line.to_string(), false);
        }

        let trimmed = line.trim();

        // Check if line starts with a debug print macro
        if self.is_debug_print(trimmed) {
            // Count parentheses to see if it's a complete statement
            let open_count = line.matches('(').count();
            let close_count = line.matches(')').count();

            if open_count > close_count {
                // Multi-line print, need to skip subsequent lines
                return (String::new(), true);
            } else {
                // Single line print, skip it
                return (String::new(), false);
            }
        }

        (line.to_string(), false)
    }
}

impl<'a> LanguageFilter for RustFilter<'a> {
    fn config(&self) -> &FilterConfig {
        self.config
    }

    fn is_comment_line(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("//") && !trimmed.starts_with("///")
    }

    fn is_doc_comment(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("///") || trimmed.starts_with("//!")
    }

    fn filter(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut in_test_block = false;
        let mut in_block_comment = false;
        let mut in_doc_comment = false;
        let mut in_multiline_print = false;
        let mut brace_depth = 0;
        let mut test_block_depth = 0;

        for line in lines {
            let trimmed = line.trim();

            // Handle multi-line print statements
            if in_multiline_print {
                let close_count = line.matches(')').count();
                let open_count = line.matches('(').count();

                if close_count > open_count {
                    in_multiline_print = false;
                }
                continue;
            }

            // Handle block comments
            if trimmed.starts_with("/*") {
                in_block_comment = true;
                in_doc_comment = trimmed.starts_with("/**") || trimmed.starts_with("/*!");
            }

            if in_block_comment {
                if trimmed.ends_with("*/") {
                    in_block_comment = false;
                    in_doc_comment = false;
                }

                let should_skip = if in_doc_comment {
                    self.config.remove_doc_comments
                } else {
                    self.config.remove_comments
                };

                if !should_skip {
                    result.push(line.to_string());
                }
                continue;
            }

            // Skip doc comments
            if self.config.remove_doc_comments && self.is_doc_comment(line) {
                continue;
            }

            // Skip regular comments
            if self.config.remove_comments && self.is_comment_line(line) {
                continue;
            }

            // Handle test blocks
            if self.config.remove_tests {
                if self.is_test_start(line) || self.has_test_attribute(line) {
                    in_test_block = true;
                    test_block_depth = 0;
                    continue;
                }

                if in_test_block {
                    // Track braces to find end of test block
                    for ch in trimmed.chars() {
                        match ch {
                            '{' => brace_depth += 1,
                            '}' => {
                                brace_depth -= 1;
                                if brace_depth <= test_block_depth {
                                    in_test_block = false;
                                }
                            }
                            _ => {}
                        }
                    }
                    continue;
                }
            }

            // Remove debug prints
            let (processed_line, is_multiline) = self.strip_debug_prints(line);
            if is_multiline {
                in_multiline_print = true;
                continue;
            }

            // Remove inline comments if configured
            let mut final_line = processed_line;
            if self.config.remove_comments && !final_line.is_empty() {
                final_line = self.strip_line_comment(&final_line, "//");
            }

            // Skip blank lines if configured
            if self.config.remove_blank_lines && final_line.trim().is_empty() {
                continue;
            }

            result.push(final_line);
        }

        result.join("\n")
    }
}

/// Python-specific code filter.
struct PythonFilter<'a> {
    config: &'a FilterConfig,
}

impl<'a> PythonFilter<'a> {
    const fn new(config: &'a FilterConfig) -> Self {
        Self { config }
    }

    fn is_test_function(&self, line: &str) -> bool {
        let trimmed = line.trim();
        (trimmed.starts_with("def test_") || trimmed.starts_with("async def test_"))
            && trimmed.contains('(')
    }

    fn is_test_decorator(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("@pytest")
            || trimmed.starts_with("@unittest")
            || trimmed == "@test"
    }
}

impl<'a> LanguageFilter for PythonFilter<'a> {
    fn config(&self) -> &FilterConfig {
        self.config
    }

    fn is_comment_line(&self, line: &str) -> bool {
        line.trim().starts_with('#')
    }

    fn is_doc_comment(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''")
    }

    fn filter(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut in_docstring = false;
        let mut in_test_function = false;
        let _indent_level = 0;
        let mut test_indent = 0;

        for line in lines {
            let trimmed = line.trim();

            // Handle docstrings
            if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
                in_docstring = !in_docstring;
                if self.config.remove_doc_comments {
                    continue;
                }
            }

            if in_docstring {
                if self.config.remove_doc_comments {
                    continue;
                }
                result.push(line.to_string());
                continue;
            }

            // Skip comments
            if self.config.remove_comments && self.is_comment_line(line) {
                continue;
            }

            // Handle test functions
            if self.config.remove_tests {
                let current_indent = line.len() - line.trim_start().len();

                if self.is_test_decorator(line) {
                    in_test_function = true;
                    test_indent = current_indent;
                    continue;
                }

                if self.is_test_function(line) {
                    in_test_function = true;
                    test_indent = current_indent;
                    continue;
                }

                if in_test_function {
                    if !trimmed.is_empty() && current_indent <= test_indent {
                        in_test_function = false;
                    } else {
                        continue;
                    }
                }
            }

            // Skip blank lines if configured
            if self.config.remove_blank_lines && trimmed.is_empty() {
                continue;
            }

            result.push(line.to_string());
        }

        result.join("\n")
    }
}

/// JavaScript/TypeScript code filter.
struct JavaScriptFilter<'a> {
    config: &'a FilterConfig,
}

impl<'a> JavaScriptFilter<'a> {
    const fn new(config: &'a FilterConfig) -> Self {
        Self { config }
    }

    #[allow(dead_code)]
    fn is_test_block(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("describe(")
            || trimmed.starts_with("it(")
            || trimmed.starts_with("test(")
            || trimmed.starts_with("expect(")
    }
}

impl<'a> LanguageFilter for JavaScriptFilter<'a> {
    fn config(&self) -> &FilterConfig {
        self.config
    }

    fn is_comment_line(&self, line: &str) -> bool {
        line.trim().starts_with("//")
    }

    fn is_doc_comment(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("/**") || trimmed.starts_with("///")
    }

    fn filter(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut in_block_comment = false;
        let mut in_doc_comment = false;

        for line in lines {
            let trimmed = line.trim();

            // Handle block comments
            if trimmed.starts_with("/*") {
                in_block_comment = true;
                in_doc_comment = trimmed.starts_with("/**");
            }

            if in_block_comment {
                if trimmed.ends_with("*/") {
                    in_block_comment = false;
                    in_doc_comment = false;
                }

                let should_skip = if in_doc_comment {
                    self.config.remove_doc_comments
                } else {
                    self.config.remove_comments
                };

                if !should_skip {
                    result.push(line.to_string());
                }
                continue;
            }

            // Skip comments
            if self.config.remove_comments && self.is_comment_line(line) {
                continue;
            }

            if self.config.remove_doc_comments && self.is_doc_comment(line) {
                continue;
            }

            // Skip blank lines if configured
            if self.config.remove_blank_lines && trimmed.is_empty() {
                continue;
            }

            // Remove inline comments
            let mut processed_line = line.to_string();
            if self.config.remove_comments {
                processed_line = self.strip_line_comment(&processed_line, "//");
            }

            result.push(processed_line);
        }

        result.join("\n")
    }
}

/// Go-specific code filter.
struct GoFilter<'a> {
    config: &'a FilterConfig,
}

impl<'a> GoFilter<'a> {
    const fn new(config: &'a FilterConfig) -> Self {
        Self { config }
    }
}

impl<'a> LanguageFilter for GoFilter<'a> {
    fn config(&self) -> &FilterConfig {
        self.config
    }

    fn is_comment_line(&self, line: &str) -> bool {
        line.trim().starts_with("//")
    }

    fn is_doc_comment(&self, _line: &str) -> bool {
        false // Go doesn't have special doc comments
    }

    fn filter(&self, content: &str) -> String {
        JavaScriptFilter::new(self.config).filter(content)
    }
}

/// Java/Kotlin code filter.
struct JavaFilter<'a> {
    config: &'a FilterConfig,
}

impl<'a> JavaFilter<'a> {
    const fn new(config: &'a FilterConfig) -> Self {
        Self { config }
    }

    fn is_test_annotation(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("@Test")
            || trimmed.starts_with("@org.junit")
            || trimmed.starts_with("@BeforeEach")
            || trimmed.starts_with("@AfterEach")
    }
}

impl<'a> LanguageFilter for JavaFilter<'a> {
    fn config(&self) -> &FilterConfig {
        self.config
    }

    fn is_comment_line(&self, line: &str) -> bool {
        line.trim().starts_with("//")
    }

    fn is_doc_comment(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("/**")
    }

    fn filter(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut in_block_comment = false;
        let mut in_doc_comment = false;
        let mut skip_next_method = false;

        for line in lines {
            let trimmed = line.trim();

            // Check for test annotations
            if self.config.remove_tests && self.is_test_annotation(line) {
                skip_next_method = true;
                continue;
            }

            // Handle block comments
            if trimmed.starts_with("/*") {
                in_block_comment = true;
                in_doc_comment = trimmed.starts_with("/**");
            }

            if in_block_comment {
                if trimmed.ends_with("*/") {
                    in_block_comment = false;
                    in_doc_comment = false;
                }

                let should_skip = if in_doc_comment {
                    self.config.remove_doc_comments
                } else {
                    self.config.remove_comments
                };

                if !should_skip {
                    result.push(line.to_string());
                }
                continue;
            }

            // Skip test methods
            if skip_next_method {
                if trimmed.contains('{') {
                    // Found method start, now skip until closing brace
                    let brace_count = trimmed.matches('{').count() as i32
                        - trimmed.matches('}').count() as i32;

                    if brace_count == 0 {
                        skip_next_method = false;
                    }
                }
                continue;
            }

            // Skip comments
            if self.config.remove_comments && self.is_comment_line(line) {
                continue;
            }

            // Skip blank lines
            if self.config.remove_blank_lines && trimmed.is_empty() {
                continue;
            }

            result.push(line.to_string());
        }

        result.join("\n")
    }
}

/// C/C++ code filter.
struct CFilter<'a> {
    config: &'a FilterConfig,
}

impl<'a> CFilter<'a> {
    const fn new(config: &'a FilterConfig) -> Self {
        Self { config }
    }
}

impl<'a> LanguageFilter for CFilter<'a> {
    fn config(&self) -> &FilterConfig {
        self.config
    }

    fn is_comment_line(&self, line: &str) -> bool {
        line.trim().starts_with("//")
    }

    fn is_doc_comment(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("///") || trimmed.starts_with("/**")
    }

    fn filter(&self, content: &str) -> String {
        JavaScriptFilter::new(self.config).filter(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_filter_removes_tests() {
        let config = FilterConfig::default();
        let filter = CodeFilter::new(config);

        let code = r#"
fn production_code() {}

#[test]
fn test_something() {
    assert_eq!(1, 1);
}

fn more_production() {}
"#;

        let filtered = filter.filter(code, Path::new("test.rs"));
        assert!(!filtered.contains("#[test]"));
        assert!(!filtered.contains("test_something"));
        assert!(filtered.contains("production_code"));
        assert!(filtered.contains("more_production"));
    }

    #[test]
    fn test_rust_filter_removes_comments() {
        let mut config = FilterConfig::default();
        config.remove_comments = true;

        let filter = CodeFilter::new(config);

        let code = r#"
// This is a comment
fn code() {} // inline comment
"#;

        let filtered = filter.filter(code, Path::new("test.rs"));
        assert!(!filtered.contains("This is a comment"));
        assert!(filtered.contains("fn code()"));
    }

    #[test]
    fn test_python_filter_removes_tests() {
        let config = FilterConfig::default();
        let filter = CodeFilter::new(config);

        let code = r#"
def production_function():
    pass

def test_something():
    assert True

def another_production():
    pass
"#;

        let filtered = filter.filter(code, Path::new("test.py"));
        assert!(!filtered.contains("test_something"));
        assert!(filtered.contains("production_function"));
        assert!(filtered.contains("another_production"));
    }

    #[test]
    fn test_filter_preserves_strings_with_comment_markers() {
        let config = FilterConfig {
            remove_doc_comments: true,
            remove_comments: true,
            ..Default::default()
        };
        let filter = CodeFilter::new(config);

        let code = r#"let url = "https://example.com"; // real comment"#;
        let filtered = filter.filter(code, Path::new("test.rs"));

        assert!(filtered.contains("https://"));
        assert!(!filtered.contains("real comment"));
    }

    #[test]
    fn test_remove_println() {
        let config = FilterConfig {
            remove_debug_prints: true,
            ..Default::default()
        };
        let filter = CodeFilter::new(config);

        let code = r#"
fn main() {
    let x = 5;
    println!("x = {}", x);
    let y = 10;
}
"#;

        let filtered = filter.filter(code, Path::new("test.rs"));
        assert!(!filtered.contains("println!"));
        assert!(filtered.contains("let x = 5"));
        assert!(filtered.contains("let y = 10"));
    }

    #[test]
    fn test_remove_multiline_println() {
        let config = FilterConfig {
            remove_debug_prints: true,
            ..Default::default()
        };
        let filter = CodeFilter::new(config);

        let code = r#"
fn main() {
    let x = 5;
    println!(
        "x = {}",
        x
    );
    let y = 10;
}
"#;

        let filtered = filter.filter(code, Path::new("test.rs"));
        assert!(!filtered.contains("println!"));
        assert!(filtered.contains("let x = 5"));
        assert!(filtered.contains("let y = 10"));
    }

    #[test]
    fn test_remove_dbg() {
        let config = FilterConfig {
            remove_debug_prints: true,
            ..Default::default()
        };
        let filter = CodeFilter::new(config);

        let code = r#"
fn main() {
    let x = 5;
    dbg!(x);
    let y = 10;
}
"#;

        let filtered = filter.filter(code, Path::new("test.rs"));
        assert!(!filtered.contains("dbg!"));
        assert!(filtered.contains("let x = 5"));
    }
}