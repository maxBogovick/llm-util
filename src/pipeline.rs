use crate::{
    config::Config,
    error::Result,
    file::FileData,
    scanner::Scanner,
    splitter::Splitter,
    writer::Writer,
};
use serde::Serialize;
use std::time::{Duration, Instant};
use tracing::{info, instrument, warn};

/// Statistics collected during pipeline execution.
#[derive(Debug, Clone, Serialize)]
pub struct PipelineStats {
    /// Total number of files scanned
    pub total_files: usize,

    /// Number of text files processed
    pub text_files: usize,

    /// Number of binary files found
    pub binary_files: usize,

    /// Total number of chunks created
    pub total_chunks: usize,

    /// Total tokens across all files
    pub total_tokens: usize,

    /// Average tokens per chunk
    pub avg_tokens_per_chunk: usize,

    /// Largest chunk size in tokens
    pub max_chunk_tokens: usize,

    /// Smallest chunk size in tokens
    pub min_chunk_tokens: usize,

    /// Total execution time
    pub duration: Duration,

    /// Time spent scanning
    pub scan_duration: Duration,

    /// Time spent splitting
    pub split_duration: Duration,

    /// Time spent writing
    pub write_duration: Duration,

    /// Output directory path
    pub output_directory: String,

    /// Number of files written
    pub files_written: usize,
}

impl PipelineStats {
    /// Creates statistics from pipeline execution data.
    #[must_use]
    pub fn new(
        total_files: usize,
        text_files: usize,
        binary_files: usize,
        chunks: &[crate::Chunk],
        duration: Duration,
        scan_duration: Duration,
        split_duration: Duration,
        write_duration: Duration,
        output_directory: String,
        files_written: usize,
    ) -> Self {
        let total_chunks = chunks.len();
        let total_tokens: usize = chunks.iter().map(|c| c.total_tokens).sum();

        let avg_tokens_per_chunk = if total_chunks > 0 {
            total_tokens / total_chunks
        } else {
            0
        };

        let max_chunk_tokens = chunks.iter().map(|c| c.total_tokens).max().unwrap_or(0);

        let min_chunk_tokens = chunks.iter().map(|c| c.total_tokens).min().unwrap_or(0);

        Self {
            total_files,
            text_files,
            binary_files,
            total_chunks,
            total_tokens,
            avg_tokens_per_chunk,
            max_chunk_tokens,
            min_chunk_tokens,
            duration,
            scan_duration,
            split_duration,
            write_duration,
            output_directory,
            files_written,
        }
    }

    /// Prints a human-readable summary to stdout.
    pub fn print_summary(&self) {
        println!("\n╔═══════════════════════════════════════════════════════╗");
        println!("║            Pipeline Execution Summary                 ║");
        println!("╠═══════════════════════════════════════════════════════╣");
        println!(
            "║ Files Scanned:        {:>8}                        ║",
            self.total_files
        );
        println!(
            "║   - Text files:       {:>8}                        ║",
            self.text_files
        );
        println!(
            "║   - Binary files:     {:>8}                        ║",
            self.binary_files
        );
        println!("║                                                       ║");
        println!(
            "║ Chunks Created:       {:>8}                        ║",
            self.total_chunks
        );
        println!(
            "║ Total Tokens:         {:>8}                        ║",
            self.total_tokens
        );
        println!(
            "║ Avg Tokens/Chunk:     {:>8}                        ║",
            self.avg_tokens_per_chunk
        );
        println!(
            "║ Min Chunk Size:       {:>8} tokens                 ║",
            self.min_chunk_tokens
        );
        println!(
            "║ Max Chunk Size:       {:>8} tokens                 ║",
            self.max_chunk_tokens
        );
        println!("║                                                       ║");
        println!(
            "║ Files Written:        {:>8}                        ║",
            self.files_written
        );
        println!("║ Output Directory:                                     ║");
        println!(
            "║   {}                                              ║",
            self.output_directory
        );
        println!("║                                                       ║");
        println!("║ Timing Breakdown:                                     ║");
        println!(
            "║   - Scanning:         {:>8.2}s                     ║",
            self.scan_duration.as_secs_f64()
        );
        println!(
            "║   - Splitting:        {:>8.2}s                     ║",
            self.split_duration.as_secs_f64()
        );
        println!(
            "║   - Writing:          {:>8.2}s                     ║",
            self.write_duration.as_secs_f64()
        );
        println!(
            "║   - Total:            {:>8.2}s                     ║",
            self.duration.as_secs_f64()
        );
        println!("╚═══════════════════════════════════════════════════════╝\n");
    }

    /// Returns the throughput in files per second.
    #[must_use]
    pub fn throughput_files_per_sec(&self) -> f64 {
        self.total_files as f64 / self.duration.as_secs_f64()
    }

    /// Returns the throughput in tokens per second.
    #[must_use]
    pub fn throughput_tokens_per_sec(&self) -> f64 {
        self.total_tokens as f64 / self.duration.as_secs_f64()
    }
}

/// Main pipeline orchestrator for converting repositories to prompts.
pub struct Pipeline {
    config: Config,
    scanner: Scanner,
    splitter: Splitter,
    writer: Writer,
}

impl Pipeline {
    /// Creates a new pipeline with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Configuration validation fails
    /// - Writer initialization fails
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;

        let scanner = Scanner::new(&config);
        let splitter = Splitter::new(&config);
        let writer = Writer::new(&config)?;

        Ok(Self {
            config,
            scanner,
            splitter,
            writer,
        })
    }

    /// Executes the complete pipeline and returns statistics.
    ///
    /// # Process
    ///
    /// 1. **Scan**: Discovers and reads files from the root directory
    /// 2. **Split**: Divides content into optimal chunks based on token limits
    /// 3. **Write**: Renders and persists chunks to output files
    ///
    /// # Errors
    ///
    /// Returns an error if any stage fails critically.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_utl::{Config, Pipeline};
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let config = Config::builder()
    ///     .root_dir("./src")
    ///     .build()?;
    ///
    /// let stats = Pipeline::new(config)?.run()?;
    /// stats.print_summary();
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(root_dir = %self.config.root_dir.display()))]
    pub fn run(self) -> Result<PipelineStats> {
        let start_time = Instant::now();

        info!("Starting pipeline execution");

        // Stage 1: Scanning
        info!("Stage 1/3: Scanning repository...");
        let scan_start = Instant::now();
        let files = self.scan()?;
        let scan_duration = scan_start.elapsed();

        let total_files = files.len();
        let text_files = files.iter().filter(|f| f.is_text()).count();
        let binary_files = files.iter().filter(|f| f.is_binary()).count();

        info!(
            "✓ Scanned {} files ({} text, {} binary) in {:.2}s",
            total_files,
            text_files,
            binary_files,
            scan_duration.as_secs_f64()
        );

        // Stage 2: Splitting
        info!("Stage 2/3: Splitting into chunks...");
        let split_start = Instant::now();
        let chunks = self.splitter.split(files)?;
        let split_duration = split_start.elapsed();

        info!(
            "✓ Created {} chunks in {:.2}s",
            chunks.len(),
            split_duration.as_secs_f64()
        );

        // Log chunk distribution
        self.log_chunk_distribution(&chunks);

        // Stage 3: Writing
        let write_start = Instant::now();
        let files_written = if self.config.dry_run {
            warn!("Dry run mode enabled - skipping file writes");
            self.print_dry_run_summary(&chunks);
            0
        } else {
            info!("Stage 3/3: Writing output files...");
            self.writer.write_chunks(&chunks)?;
            self.writer.write_summary(&chunks, start_time.elapsed())?;
            chunks.len() + 1 // +1 for summary.json
        };
        let write_duration = write_start.elapsed();

        if !self.config.dry_run {
            info!(
                "✓ Wrote {} files in {:.2}s",
                files_written,
                write_duration.as_secs_f64()
            );
        }

        let total_duration = start_time.elapsed();

        // Create statistics
        let stats = PipelineStats::new(
            total_files,
            text_files,
            binary_files,
            &chunks,
            total_duration,
            scan_duration,
            split_duration,
            write_duration,
            self.config.output_dir.display().to_string(),
            files_written,
        );

        info!(
            "✓ Pipeline completed successfully in {:.2}s",
            total_duration.as_secs_f64()
        );

        Ok(stats)
    }

    /// Executes the scanning stage.
    fn scan(&self) -> Result<Vec<FileData>> {
        self.scanner.scan()
    }

    /// Logs information about chunk distribution.
    fn log_chunk_distribution(&self, chunks: &[crate::Chunk]) {
        if chunks.is_empty() {
            return;
        }

        let total_tokens: usize = chunks.iter().map(|c| c.total_tokens).sum();
        let avg_tokens = total_tokens / chunks.len();
        let max_tokens = chunks.iter().map(|c| c.total_tokens).max().unwrap_or(0);
        let min_tokens = chunks.iter().map(|c| c.total_tokens).min().unwrap_or(0);

        info!(
            "  Chunk stats: avg={}, min={}, max={} tokens",
            avg_tokens, min_tokens, max_tokens
        );

        // Log chunks that are close to the limit
        let limit = self.config.max_tokens;
        let threshold = (limit as f64 * 0.9) as usize;
        let near_limit: Vec<_> = chunks
            .iter()
            .filter(|c| c.total_tokens > threshold)
            .collect();

        if !near_limit.is_empty() {
            warn!(
                "  {} chunk(s) are >90% of token limit (may need adjustment)",
                near_limit.len()
            );
        }
    }

    /// Prints a summary for dry run mode.
    fn print_dry_run_summary(&self, chunks: &[crate::Chunk]) {
        println!("\n╔═══════════════════════════════════════════════════════╗");
        println!("║                 Dry Run Summary                       ║");
        println!("╠═══════════════════════════════════════════════════════╣");
        println!(
            "║ Total chunks:         {:>8}                        ║",
            chunks.len()
        );
        println!(
            "║ Total files:          {:>8}                        ║",
            chunks.iter().map(|c| c.files.len()).sum::<usize>()
        );
        println!("║ Output directory:                                     ║");
        println!(
            "║   {}                                              ║",
            self.config.output_dir.display()
        );
        println!("║                                                       ║");
        println!("║ ⚠ No files were written (dry run mode)               ║");
        println!("╚═══════════════════════════════════════════════════════╝\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    fn create_test_config(root: &std::path::Path) -> Config {
        Config::builder()
            .root_dir(root)
            .output_dir(root.join("out"))
            .build()
            .unwrap()
    }

    #[test]
    fn test_pipeline_basic_execution() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.child("file1.rs").write_str("fn main() {}").unwrap();
        temp.child("file2.rs")
            .write_str("pub fn test() {}")
            .unwrap();

        let config = create_test_config(temp.path());
        let pipeline = Pipeline::new(config).unwrap();
        let stats = pipeline.run().unwrap();

        assert_eq!(stats.total_files, 2);
        assert!(stats.total_chunks > 0);
        assert!(stats.duration.as_secs_f64() > 0.0);
    }

    #[test]
    fn test_pipeline_dry_run() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.child("file.rs").write_str("fn main() {}").unwrap();

        let config = Config::builder()
            .root_dir(temp.path())
            .output_dir(temp.path().join("out"))
            .dry_run(true)
            .build()
            .unwrap();

        let pipeline = Pipeline::new(config).unwrap();
        let stats = pipeline.run().unwrap();

        assert_eq!(stats.files_written, 0);
        assert!(!temp.child("out").exists());
    }

    #[test]
    fn test_pipeline_stats_calculation() {
        use crate::{Chunk, FileData};
        use std::path::PathBuf;

        let chunks = vec![
            Chunk {
                index: 0,
                files: vec![FileData::new_text(
                    PathBuf::from("test.rs"),
                    "test.rs".to_string(),
                    "fn main() {}".to_string(),
                    100,
                )],
                total_tokens: 100,
            },
            Chunk {
                index: 1,
                files: vec![FileData::new_text(
                    PathBuf::from("test2.rs"),
                    "test2.rs".to_string(),
                    "pub fn test() {}".to_string(),
                    200,
                )],
                total_tokens: 200,
            },
        ];

        let stats = PipelineStats::new(
            2,
            2,
            0,
            &chunks,
            Duration::from_secs(1),
            Duration::from_millis(300),
            Duration::from_millis(200),
            Duration::from_millis(500),
            "/tmp/out".to_string(),
            3,
        );

        assert_eq!(stats.total_chunks, 2);
        assert_eq!(stats.total_tokens, 300);
        assert_eq!(stats.avg_tokens_per_chunk, 150);
        assert_eq!(stats.max_chunk_tokens, 200);
        assert_eq!(stats.min_chunk_tokens, 100);
    }

    #[test]
    fn test_pipeline_throughput() {
        use crate::Chunk;

        let stats = PipelineStats::new(
            100,
            100,
            0,
            &vec![Chunk {
                index: 0,
                files: vec![],
                total_tokens: 10000,
            }],
            Duration::from_secs(2),
            Duration::from_secs(1),
            Duration::from_secs(0),
            Duration::from_secs(1),
            "/tmp/out".to_string(),
            1,
        );

        assert_eq!(stats.throughput_files_per_sec(), 50.0);
        assert_eq!(stats.throughput_tokens_per_sec(), 5000.0);
    }
}
