use crate::{
    config::{Config, OutputFormat},
    error::{Error, Result},
    splitter::Chunk,
};
use serde::Serialize;
use std::collections::HashMap;
use tera::{Context, Tera, Value};

#[derive(Serialize)]
struct TemplateContext<'a> {
    chunk_index: usize,
    total_chunks: usize,
    chunk_files: usize,
    total_tokens: usize,
    files: Vec<FileView<'a>>,
    metadata: ContextMetadata,
}

#[derive(Serialize)]
struct FileView<'a> {
    path: &'a str,
    relative_path: &'a str,
    content: Option<&'a str>,
    is_binary: bool,
    token_count: usize,
    lines: Option<usize>,
}

#[derive(Serialize)]
struct ContextMetadata {
    generated_at: String,
    format: String,
}

/// Template engine for rendering chunks in different formats.
pub(crate) struct TemplateEngine {
    tera: Tera,
    format: OutputFormat,
}

impl TemplateEngine {
    /// Creates a new template engine from configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if template registration or filter setup fails.
    pub(crate) fn new(config: &Config) -> Result<Self> {
        let mut tera = Tera::default();

        // Register built-in templates
        Self::register_builtin_templates(&mut tera)?;

        // Register custom filters
        Self::register_filters(&mut tera);

        Ok(Self {
            tera,
            format: config.format,
        })
    }

    /// Registers built-in templates for each output format.
    fn register_builtin_templates(tera: &mut Tera) -> Result<()> {
        // Markdown template
        tera.add_raw_template(
            "markdown",
            include_str!("../templates/markdown.tera"),
        )
            .map_err(|e| Error::template("markdown", e))?;

        // XML template
        tera.add_raw_template("xml", include_str!("../templates/xml.tera"))
            .map_err(|e| Error::template("xml", e))?;

        // JSON template
        tera.add_raw_template("json", include_str!("../templates/json.tera"))
            .map_err(|e| Error::template("json", e))?;

        Ok(())
    }

    /// Registers custom Tera filters.
    fn register_filters(tera: &mut Tera) {
        // XML escaping filter
        tera.register_filter("xml_escape", Self::xml_escape_filter);

        // JSON encoding filter
        tera.register_filter("json_encode", Self::json_encode_filter);

        // Truncate lines filter (for limiting output size)
        tera.register_filter("truncate_lines", Self::truncate_lines_filter);

        // Language detection filter
        tera.register_filter("detect_language", Self::detect_language_filter);
    }

    /// XML escape filter implementation.
    fn xml_escape_filter(
        value: &Value,
        _args: &HashMap<String, Value>,
    ) -> tera::Result<Value> {
        if let Some(s) = value.as_str() {
            let escaped = s
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&apos;");
            Ok(Value::String(escaped))
        } else {
            Ok(value.clone())
        }
    }

    /// JSON encode filter implementation.
    fn json_encode_filter(
        value: &Value,
        args: &HashMap<String, Value>,
    ) -> tera::Result<Value> {
        let pretty = args
            .get("pretty")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let result = if pretty {
            serde_json::to_string_pretty(value)
        } else {
            serde_json::to_string(value)
        };

        match result {
            Ok(json) => Ok(Value::String(json)),
            Err(e) => Err(tera::Error::msg(format!(
                "Failed to encode JSON: {}",
                e
            ))),
        }
    }

    /// Truncate lines filter implementation.
    fn truncate_lines_filter(
        value: &Value,
        args: &HashMap<String, Value>,
    ) -> tera::Result<Value> {
        let max_lines = args
            .get("max")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000) as usize;

        if let Some(s) = value.as_str() {
            let lines: Vec<&str> = s.lines().collect();
            if lines.len() > max_lines {
                let truncated = lines[..max_lines].join("\n");
                Ok(Value::String(format!(
                    "{}\n... ({} more lines omitted)",
                    truncated,
                    lines.len() - max_lines
                )))
            } else {
                Ok(value.clone())
            }
        } else {
            Ok(value.clone())
        }
    }

    /// Detects programming language from file extension.
    fn detect_language_filter(
        value: &Value,
        _args: &HashMap<String, Value>,
    ) -> tera::Result<Value> {
        if let Some(path) = value.as_str() {
            let language = if let Some(ext) = path.rsplit('.').next() {
                match ext {
                    "rs" => "rust",
                    "py" => "python",
                    "js" => "javascript",
                    "ts" => "typescript",
                    "jsx" => "jsx",
                    "tsx" => "tsx",
                    "go" => "go",
                    "java" => "java",
                    "c" => "c",
                    "h" => "c",
                    "cpp" | "cc" | "cxx" => "cpp",
                    "hpp" | "hh" | "hxx" => "cpp",
                    "cs" => "csharp",
                    "rb" => "ruby",
                    "php" => "php",
                    "swift" => "swift",
                    "kt" => "kotlin",
                    "scala" => "scala",
                    "sh" | "bash" => "bash",
                    "zsh" => "zsh",
                    "fish" => "fish",
                    "ps1" => "powershell",
                    "html" | "htm" => "html",
                    "css" => "css",
                    "scss" => "scss",
                    "sass" => "sass",
                    "xml" => "xml",
                    "json" => "json",
                    "yaml" | "yml" => "yaml",
                    "toml" => "toml",
                    "ini" => "ini",
                    "md" | "markdown" => "markdown",
                    "sql" => "sql",
                    "graphql" | "gql" => "graphql",
                    "proto" => "protobuf",
                    "dockerfile" => "dockerfile",
                    "makefile" => "makefile",
                    _ => "",
                }
            } else {
                ""
            };
            Ok(Value::String(language.to_string()))
        } else {
            Ok(Value::String(String::new()))
        }
    }

    /// Renders a chunk using the configured template.
    ///
    /// # Errors
    ///
    /// Returns an error if template rendering fails.
    pub(crate) fn render(&self, chunk: &Chunk, total_chunks: usize) -> Result<String> {
        let template_name = self.format.template_name();

        let files: Vec<FileView> = chunk
            .files
            .iter()
            .map(|f| {
                let content_str = f.content_str();
                let lines = content_str.map(|s| s.lines().count());

                FileView {
                    path: f.absolute_path.to_str().unwrap_or(""),
                    relative_path: &f.relative_path,
                    content: content_str,
                    is_binary: f.is_binary(),
                    token_count: f.token_count,
                    lines,
                }
            })
            .collect();

        let context = TemplateContext {
            chunk_index: chunk.index + 1,
            total_chunks,
            chunk_files: chunk.files.len(),
            total_tokens: chunk.total_tokens,
            files,
            metadata: ContextMetadata {
                generated_at: chrono::Local::now()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                format: format!("{:?}", self.format),
            },
        };

        let mut tera_context = Context::new();
        tera_context.insert("ctx", &context);

        self.tera
            .render(template_name, &tera_context)
            .map_err(|e| Error::template(template_name, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::FileData;
    use std::path::PathBuf;

    fn create_test_config(format: OutputFormat) -> Config {
        use assert_fs::TempDir;
        let temp = TempDir::new().unwrap();

        Config::builder()
            .root_dir(temp.path())
            .output_dir(temp.path().join("out"))
            .format(format)
            .build()
            .unwrap()
    }

    fn create_test_chunk() -> Chunk {
        Chunk::new(
            0,
            vec![
                FileData::new_text(
                    PathBuf::from("test.rs"),
                    "test.rs".to_string(),
                    "fn main() {\n    println!(\"Hello\");\n}".to_string(),
                    10,
                ),
                FileData::new_binary(
                    PathBuf::from("binary.exe"),
                    "binary.exe".to_string(),
                    1024,
                ),
            ],
            10,
        )
    }

    #[test]
    fn test_template_engine_creation() {
        let config = create_test_config(OutputFormat::Markdown);
        let engine = TemplateEngine::new(&config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_render_markdown() {
        let config = create_test_config(OutputFormat::Markdown);
        let engine = TemplateEngine::new(&config).unwrap();
        let chunk = create_test_chunk();

        let result = engine.render(&chunk, 1);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("test.rs"));
        assert!(rendered.contains("fn main()"));
        assert!(rendered.contains("Binary file"));
    }

    #[test]
    fn test_render_json() {
        let config = create_test_config(OutputFormat::Json);
        let engine = TemplateEngine::new(&config).unwrap();
        let chunk = create_test_chunk();

        let result = engine.render(&chunk, 1);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(parsed["chunk_index"], 1);
        assert_eq!(parsed["total_chunks"], 1);
    }

    #[test]
    fn test_render_xml() {
        let config = create_test_config(OutputFormat::Xml);
        let engine = TemplateEngine::new(&config).unwrap();
        let chunk = create_test_chunk();

        let result = engine.render(&chunk, 1);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("<?xml"));
        assert!(rendered.contains("<repository_context>"));
        assert!(rendered.contains("test.rs"));
    }

    #[test]
    fn test_xml_escape_filter() {
        let value = Value::String("<test & \"quotes\">".to_string());
        let result = TemplateEngine::xml_escape_filter(&value, &HashMap::new()).unwrap();

        let escaped = result.as_str().unwrap();
        assert_eq!(escaped, "&lt;test &amp; &quot;quotes&quot;&gt;");
    }

    #[test]
    fn test_json_encode_filter() {
        let value = Value::String("Hello \"World\"".to_string());
        let result = TemplateEngine::json_encode_filter(&value, &HashMap::new()).unwrap();

        let encoded = result.as_str().unwrap();
        assert!(encoded.contains("\\\""));
    }

    #[test]
    fn test_detect_language_filter() {
        let test_cases = vec![
            ("test.rs", "rust"),
            ("script.py", "python"),
            ("app.js", "javascript"),
            ("style.css", "css"),
            ("index.html", "html"),
            ("config.toml", "toml"),
            ("unknown.xyz", ""),
        ];

        for (path, expected_lang) in test_cases {
            let value = Value::String(path.to_string());
            let result = TemplateEngine::detect_language_filter(&value, &HashMap::new()).unwrap();

            assert_eq!(result.as_str().unwrap(), expected_lang);
        }
    }

    #[test]
    fn test_truncate_lines_filter() {
        let content = (0..100).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
        let value = Value::String(content);

        let mut args = HashMap::new();
        args.insert("max".to_string(), Value::Number(10.into()));

        let result = TemplateEngine::truncate_lines_filter(&value, &args).unwrap();

        let truncated = result.as_str().unwrap();
        assert!(truncated.contains("90 more lines omitted"));
    }
}