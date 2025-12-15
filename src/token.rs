use std::sync::Arc;

const SIMPLE_CHARS_PER_TOKEN: usize = 4;
const ENHANCED_WORD_MULTIPLIER: f64 = 1.3;
const ENHANCED_SPECIAL_DIVISOR: usize = 10;

/// Type of tokenizer to use for estimation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenizerKind {
    /// Simple character-based tokenizer (~4 chars per token)
    Simple,
    /// Enhanced tokenizer with word and special character analysis
    Enhanced,
}

impl TokenizerKind {
    /// Creates a new tokenizer instance of this kind.
    #[must_use]
    pub fn create(self) -> Arc<dyn TokenEstimator> {
        match self {
            Self::Simple => Arc::new(SimpleTokenizer),
            Self::Enhanced => Arc::new(EnhancedTokenizer),
        }
    }
}

/// Trait for estimating token counts in text.
///
/// Implementations should be thread-safe and efficient.
pub trait TokenEstimator: Send + Sync {
    /// Estimates the number of tokens in the given text.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to analyze
    ///
    /// # Returns
    ///
    /// The estimated token count (always >= 0)
    fn estimate(&self, text: &str) -> usize;

    /// Estimates tokens for a batch of texts in parallel.
    ///
    /// Default implementation calls `estimate` for each text.
    fn estimate_batch(&self, texts: &[&str]) -> Vec<usize> {
        texts.iter().map(|t| self.estimate(t)).collect()
    }
}

/// Simple character-based tokenizer.
///
/// Uses a heuristic of approximately 4 characters per token,
/// which works reasonably well for source code.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SimpleTokenizer;

impl TokenEstimator for SimpleTokenizer {
    fn estimate(&self, text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        // Count characters and divide by 4, with minimum of 1
        let char_count = text.chars().count();
        char_count
            .saturating_add(SIMPLE_CHARS_PER_TOKEN - 1)
            .saturating_div(SIMPLE_CHARS_PER_TOKEN)
            .max(1)
    }
}

/// Enhanced tokenizer with multiple heuristics.
///
/// This tokenizer considers:
/// - Word count (weighted by 1.3)
/// - Character count (divided by 4)
/// - Special characters (penalty factor)
///
/// It provides better accuracy than [`SimpleTokenizer`] but is slightly slower.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EnhancedTokenizer;

impl TokenEstimator for EnhancedTokenizer {
    fn estimate(&self, text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        let words = count_words(text);
        let chars = text.chars().count();
        let special_chars = count_special_chars(text);

        // Calculate different estimates
        let word_estimate = (f64::from(words as u32) * ENHANCED_WORD_MULTIPLIER) as usize;
        let char_estimate = chars.saturating_div(SIMPLE_CHARS_PER_TOKEN);
        let special_penalty = special_chars.saturating_div(ENHANCED_SPECIAL_DIVISOR);

        // Average the estimates and add penalty
        let base_estimate = word_estimate
            .saturating_add(char_estimate)
            .saturating_div(2);

        base_estimate.saturating_add(special_penalty).max(1)
    }
}

/// Counts words in text (whitespace-separated).
#[inline]
fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Counts special (non-alphanumeric) characters.
#[inline]
fn count_special_chars(text: &str) -> usize {
    text.chars()
        .filter(|c| !c.is_alphanumeric() && !c.is_whitespace())
        .count()
}

/// Estimates tokens for a slice of text starting at a given position.
///
/// Useful for chunking operations.
#[inline]
pub(crate) fn estimate_slice(estimator: &dyn TokenEstimator, text: &str, start: usize, end: usize) -> usize {
    if start >= text.len() {
        return 0;
    }

    let end = end.min(text.len());
    let slice = &text[start..end];
    estimator.estimate(slice)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokenizer_empty() {
        let tokenizer = SimpleTokenizer;
        assert_eq!(tokenizer.estimate(""), 0);
    }

    #[test]
    fn test_simple_tokenizer_basic() {
        let tokenizer = SimpleTokenizer;
        assert_eq!(tokenizer.estimate("test"), 1); // 4 chars = 1 token
        assert_eq!(tokenizer.estimate("hello world"), 3); // 11 chars = 3 tokens
    }

    #[test]
    fn test_simple_tokenizer_long_text() {
        let tokenizer = SimpleTokenizer;
        let text = "a".repeat(1000);
        assert_eq!(tokenizer.estimate(&text), 250); // 1000/4 = 250
    }

    #[test]
    fn test_enhanced_tokenizer_empty() {
        let tokenizer = EnhancedTokenizer;
        assert_eq!(tokenizer.estimate(""), 0);
    }

    #[test]
    fn test_enhanced_tokenizer_basic() {
        let tokenizer = EnhancedTokenizer;
        let result = tokenizer.estimate("hello world");
        assert!(result > 0);
        assert!(result < 10); // Sanity check
    }

    #[test]
    fn test_enhanced_tokenizer_code() {
        let tokenizer = EnhancedTokenizer;
        let code = r#"
            fn main() {
                println!("Hello, world!");
            }
        "#;
        let result = tokenizer.estimate(code);
        assert!(result > 5);
        assert!(result < 30);
    }

    #[test]
    fn test_tokenizer_large_input() {
        let tokenizer = SimpleTokenizer;
        // Test with a reasonably large text (1MB instead of unrealistic usize::MAX/2)
        let huge_text = "a".repeat(1_000_000);
        let result = tokenizer.estimate(&huge_text);
        assert!(result > 0); // Should not panic
        assert_eq!(result, 250_000); // 1M chars / 4 = 250k tokens
    }

    #[test]
    fn test_estimate_batch() {
        let tokenizer = SimpleTokenizer;
        let texts = vec!["hello", "world", "test"];
        let results = tokenizer.estimate_batch(&texts);

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|&r| r > 0));
    }

    #[test]
    fn test_count_words() {
        assert_eq!(count_words(""), 0);
        assert_eq!(count_words("hello"), 1);
        assert_eq!(count_words("hello world"), 2);
        assert_eq!(count_words("  hello   world  "), 2);
    }

    #[test]
    fn test_count_special_chars() {
        assert_eq!(count_special_chars("hello"), 0);
        assert_eq!(count_special_chars("hello!"), 1);
        assert_eq!(count_special_chars("fn main() {}"), 4);
    }

    #[test]
    fn test_estimate_slice() {
        let tokenizer = SimpleTokenizer;
        let text = "hello world test";

        let result = estimate_slice(&tokenizer, text, 0, 5);
        assert!(result > 0);

        let result = estimate_slice(&tokenizer, text, 100, 200);
        assert_eq!(result, 0);
    }
}