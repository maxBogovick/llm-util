use crate::error::{Error, Result};
use std::fs;
use std::path::Path;
use tera::Tera;

/// Maximum template file size (1MB)
const MAX_TEMPLATE_SIZE: u64 = 1024 * 1024;

/// Required template variables that should be accessible in templates
const REQUIRED_VARIABLES: &[&str] = &[
    "chunk_index",
    "total_chunks",
    "files",
];

/// Optional but commonly used variables
const OPTIONAL_VARIABLES: &[&str] = &[
    "chunk_files",
    "total_tokens",
    "metadata",
    "preset",
    "custom",
];

/// Validates external Tera templates
pub(crate) struct TemplateValidator;

impl TemplateValidator {
    /// Validates an external template file
    ///
    /// Performs the following checks:
    /// 1. File exists and is readable
    /// 2. File size is within limits
    /// 3. Template syntax is valid (can be compiled by Tera)
    /// 4. Template contains required variables
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File doesn't exist or can't be read
    /// - File is too large
    /// - Template has syntax errors
    /// - Template is missing required variables
    pub(crate) fn validate_template(path: &Path) -> Result<()> {
        // 1. Check file exists
        if !path.exists() {
            return Err(Error::io(
                path,
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Template file not found",
                ),
            ));
        }

        if !path.is_file() {
            return Err(Error::template_validation(
                path.to_string_lossy().to_string(),
                "Path is not a file",
            ));
        }

        // 2. Check file size
        let metadata = fs::metadata(path).map_err(|e| Error::io(path, e))?;
        if metadata.len() > MAX_TEMPLATE_SIZE {
            return Err(Error::template_validation(
                path.to_string_lossy().to_string(),
                format!(
                    "Template file too large: {} bytes (max: {} bytes)",
                    metadata.len(),
                    MAX_TEMPLATE_SIZE
                ),
            ));
        }

        // 3. Read template content
        let content = fs::read_to_string(path).map_err(|e| Error::io(path, e))?;

        // Check for empty template
        if content.trim().is_empty() {
            return Err(Error::template_validation(
                path.to_string_lossy().to_string(),
                "Template file is empty",
            ));
        }

        // 4. Validate Tera syntax by compiling
        let mut temp_tera = Tera::default();
        temp_tera
            .add_raw_template("validation", &content)
            .map_err(|e| {
                Error::template_validation(
                    path.to_string_lossy().to_string(),
                    format!("Template syntax error: {}", e),
                )
            })?;

        // 5. Check for required variables (heuristic-based)
        Self::check_required_variables(&content, path)?;

        // 6. Log warnings for optional variables
        Self::check_optional_variables(&content);

        Ok(())
    }

    /// Checks if template contains required variables
    ///
    /// Uses simple heuristic: searches for variable names in template content.
    /// This may produce false positives/negatives but is sufficient for most cases.
    fn check_required_variables(content: &str, path: &Path) -> Result<()> {
        let missing: Vec<&str> = REQUIRED_VARIABLES
            .iter()
            .filter(|var| {
                // Check if variable appears in template
                // Look for patterns like {{ ctx.var }}, {{ var }}, {% for x in var %}
                let patterns = [
                    format!("ctx.{}", var),
                    format!("{{{{{} ", var),     // {{ var
                    format!("{{{{ ctx.{}", var), // {{ ctx.var
                    format!("in {}", var),        // {% for x in var %}
                    format!("in ctx.{}", var),    // {% for x in ctx.var %}
                ];

                !patterns.iter().any(|pattern| content.contains(pattern))
            })
            .copied()
            .collect();

        if !missing.is_empty() {
            return Err(Error::template_validation(
                path.to_string_lossy().to_string(),
                format!(
                    "Template may be missing required variables: {}. \n\
                    Templates should access: chunk_index, total_chunks, files. \n\
                    See built-in templates for reference.",
                    missing.join(", ")
                ),
            ));
        }

        Ok(())
    }

    /// Checks for optional variables and logs debug information
    fn check_optional_variables(content: &str) {
        for var in OPTIONAL_VARIABLES {
            let patterns = [
                format!("ctx.{}", var),
                format!("{{{{{} ", var),
                format!("{{{{ ctx.{}", var),
            ];

            if !patterns.iter().any(|pattern| content.contains(pattern)) {
                tracing::debug!("Template does not use optional variable: {}", var);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn test_validate_valid_template() {
        let temp = assert_fs::TempDir::new().unwrap();
        let template_file = temp.child("test.tera");
        template_file
            .write_str(
                "Chunk {{ ctx.chunk_index }}/{{ ctx.total_chunks }}\n\
                {% for file in ctx.files %}{{ file.path }}{% endfor %}",
            )
            .unwrap();

        let result = TemplateValidator::validate_template(template_file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_nonexistent_file() {
        let result = TemplateValidator::validate_template(Path::new("/nonexistent/template.tera"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.is_io());
    }

    #[test]
    fn test_validate_empty_template() {
        let temp = assert_fs::TempDir::new().unwrap();
        let template_file = temp.child("empty.tera");
        template_file.write_str("   \n  \n  ").unwrap();

        let result = TemplateValidator::validate_template(template_file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_validate_syntax_error() {
        let temp = assert_fs::TempDir::new().unwrap();
        let template_file = temp.child("invalid.tera");
        template_file
            .write_str("{% if condition %}\nUnclosed if")
            .unwrap();

        let result = TemplateValidator::validate_template(template_file.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Template syntax error"));
    }

    #[test]
    fn test_validate_missing_required_vars() {
        let temp = assert_fs::TempDir::new().unwrap();
        let template_file = temp.child("incomplete.tera");
        template_file
            .write_str("Hello {{ ctx.chunk_index }}")
            .unwrap();

        let result = TemplateValidator::validate_template(template_file.path());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("missing required variables"));
    }

    #[test]
    fn test_validate_with_for_loop() {
        let temp = assert_fs::TempDir::new().unwrap();
        let template_file = temp.child("with_loop.tera");
        template_file
            .write_str(
                "Chunk {{ ctx.chunk_index }}/{{ ctx.total_chunks }}\n\
                {% for file in ctx.files %}\n\
                  File: {{ file.path }}\n\
                {% endfor %}",
            )
            .unwrap();

        let result = TemplateValidator::validate_template(template_file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_too_large() {
        let temp = assert_fs::TempDir::new().unwrap();
        let template_file = temp.child("large.tera");

        // Create a file larger than MAX_TEMPLATE_SIZE
        let large_content = "x".repeat((MAX_TEMPLATE_SIZE + 1) as usize);
        template_file.write_str(&large_content).unwrap();

        let result = TemplateValidator::validate_template(template_file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }
}
