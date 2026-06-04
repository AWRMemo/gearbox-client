use crate::ai::service::AIService;
use crate::types::{ConnectionCandidate, EnrichmentOutput, Highlight};
use std::collections::HashSet;
use std::sync::LazyLock;

/// Maximum number of characters to process from a single highlight input.
/// Prevents OOM on low-memory devices (4GB phones per PRD §11).
pub const MAX_HIGHLIGHT_CHARS: usize = 8192;

/// Maximum characters for a fallback summary when no sentence boundary is found.
const SUMMARY_TRUNCATION_LIMIT: usize = 150;

/// Static set of ~100 English stop words used by the keyword extraction fallback.
static STOP_WORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "a",
        "about",
        "above",
        "after",
        "again",
        "against",
        "all",
        "am",
        "an",
        "and",
        "any",
        "are",
        "as",
        "at",
        "be",
        "because",
        "been",
        "before",
        "being",
        "below",
        "between",
        "both",
        "but",
        "by",
        "can",
        "could",
        "did",
        "do",
        "does",
        "doing",
        "don",
        "down",
        "during",
        "each",
        "few",
        "for",
        "from",
        "further",
        "had",
        "has",
        "have",
        "having",
        "he",
        "her",
        "here",
        "hers",
        "herself",
        "him",
        "himself",
        "his",
        "how",
        "i",
        "if",
        "in",
        "into",
        "is",
        "it",
        "its",
        "itself",
        "just",
        "me",
        "more",
        "most",
        "my",
        "myself",
        "no",
        "nor",
        "not",
        "now",
        "of",
        "on",
        "once",
        "only",
        "or",
        "other",
        "our",
        "ours",
        "ourselves",
        "out",
        "over",
        "own",
        "per",
        "s",
        "same",
        "she",
        "should",
        "so",
        "some",
        "such",
        "than",
        "that",
        "the",
        "their",
        "theirs",
        "them",
        "themselves",
        "then",
        "there",
        "these",
        "they",
        "this",
        "those",
        "through",
        "to",
        "too",
        "under",
        "until",
        "up",
        "very",
        "was",
        "we",
        "were",
        "what",
        "when",
        "where",
        "which",
        "while",
        "who",
        "whom",
        "why",
        "will",
        "with",
        "would",
        "you",
        "your",
        "yours",
        "yourself",
        "yourselves",
    ])
});

pub struct FallbackService;

impl FallbackService {
    fn extract_first_sentence(text: &str) -> String {
        let text = text.trim();
        if text.is_empty() {
            return "[no text]".to_string();
        }
        let mut best_end = None;
        for (i, ch) in text.char_indices() {
            if matches!(ch, '.' | '!' | '?') {
                if let Some(next) = text[i + ch.len_utf8()..].chars().next() {
                    if next == ' ' || next == '\n' || next == '\r' {
                        let end = i + ch.len_utf8();
                        best_end = Some(end);
                        break;
                    }
                }
            }
        }
        if let Some(end) = best_end {
            let sentence = text[..end].trim();
            if !sentence.is_empty() {
                return sentence.to_string();
            }
        }
        if text.chars().count() <= SUMMARY_TRUNCATION_LIMIT {
            text.to_string()
        } else {
            let truncated: String = text.chars().take(SUMMARY_TRUNCATION_LIMIT).collect();
            format!("{truncated}…")
        }
    }

    fn extract_tags(text: &str) -> Vec<String> {
        let text = text.trim();
        if text.is_empty() {
            return vec![];
        }

        let mut words: Vec<String> = Vec::new();
        let mut buf = String::new();

        for c in text.chars() {
            if c.is_alphanumeric() {
                for lowered in c.to_lowercase() {
                    buf.push(lowered);
                }
            } else if c == '\'' || c == '-' {
                buf.push(c);
            } else {
                flush_word(&mut buf, &mut words);
            }
        }
        flush_word(&mut buf, &mut words);

        words.retain(|w| !w.is_empty() && w.len() > 1 && !STOP_WORDS.contains(w.as_str()));

        if words.is_empty() {
            return vec![];
        }

        let mut freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for w in &words {
            *freq.entry(w.clone()).or_insert(0) += 1;
        }

        let mut freq_pairs: Vec<(String, usize)> = freq.into_iter().collect();
        freq_pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        let top_n = freq_pairs.len().min(5);
        freq_pairs[..top_n].iter().map(|(w, _)| w.clone()).collect()
    }
}

fn flush_word(buf: &mut String, words: &mut Vec<String>) {
    if buf.is_empty() {
        return;
    }
    let word = std::mem::take(buf);
    words.push(word);
}

impl AIService for FallbackService {
    fn enrich(
        &self,
        highlight: &Highlight,
        _candidates: &[ConnectionCandidate],
    ) -> Result<EnrichmentOutput, String> {
        let text = highlight.text.trim();

        if text.is_empty() {
            return Ok(EnrichmentOutput {
                summary: "[no text]".to_string(),
                tags: vec![],
                connection_suggestion: None,
            });
        }

        let truncated: &str = if text.chars().count() > MAX_HIGHLIGHT_CHARS {
            let byte_end = text
                .char_indices()
                .nth(MAX_HIGHLIGHT_CHARS)
                .map(|(i, _)| i)
                .unwrap_or(text.len());
            &text[..byte_end]
        } else {
            text
        };

        let summary = Self::extract_first_sentence(truncated);
        let tags = Self::extract_tags(truncated);

        Ok(EnrichmentOutput {
            summary,
            tags,
            connection_suggestion: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Highlight;

    fn make_highlight(text: &str) -> Highlight {
        Highlight {
            id: "test-1".to_string(),
            text: text.to_string(),
            source_url: None,
            source_title: None,
            source_author: None,
        }
    }

    #[test]
    fn test_normal_paragraph() {
        let text = "The Gearbox Relay project uses on-device AI to enrich captured highlights. It runs entirely locally with zero cloud tokens. The pipeline tags, summarizes, and suggests connections.";
        let highlight = make_highlight(text);
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert!(!result.summary.is_empty(), "summary should not be empty");
        assert_eq!(
            result.summary,
            "The Gearbox Relay project uses on-device AI to enrich captured highlights."
        );
        assert!(
            result.tags.len() >= 3,
            "expected at least 3 tags, got {}",
            result.tags.len()
        );
        assert!(
            result.tags.len() <= 5,
            "expected at most 5 tags, got {}",
            result.tags.len()
        );
        for tag in &result.tags {
            assert_eq!(
                tag,
                &tag.to_lowercase(),
                "tag '{}' should be lowercase",
                tag
            );
        }
        assert!(result.connection_suggestion.is_none());
    }

    #[test]
    fn test_single_sentence_no_period() {
        let text = "Gearbox Relay captures text highlights";
        let highlight = make_highlight(text);
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert_eq!(result.summary, text);
        assert!(!result.tags.is_empty());
        assert!(result.connection_suggestion.is_none());
    }

    #[test]
    fn test_empty_string() {
        let highlight = make_highlight("");
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert_eq!(result.summary, "[no text]");
        assert!(result.tags.is_empty());
        assert!(result.connection_suggestion.is_none());
    }

    #[test]
    fn test_whitespace_only() {
        let highlight = make_highlight("   \n  \t  ");
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert_eq!(result.summary, "[no text]");
        assert!(result.tags.is_empty());
        assert!(result.connection_suggestion.is_none());
    }

    #[test]
    fn test_all_stop_words() {
        let text = "the and or but for with";
        let highlight = make_highlight(text);
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert!(!result.summary.is_empty());
        assert!(
            result.tags.is_empty(),
            "all-stop-word text should produce no tags, got {:?}",
            result.tags
        );
    }

    #[test]
    fn test_repeated_words() {
        let text = "dog dog dog cat cat mouse";
        let highlight = make_highlight(text);
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert!(result.tags.contains(&"dog".to_string()));
        assert!(result.tags.contains(&"cat".to_string()));
        assert!(result.tags.contains(&"mouse".to_string()));
        assert_eq!(result.tags.len(), 3);
    }

    #[test]
    fn test_long_text_truncation() {
        let text = "a".repeat(200);
        let highlight = make_highlight(&text);
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert_eq!(result.summary.chars().count(), 151);
        assert!(result.summary.ends_with('…'));
    }

    #[test]
    fn test_punctuation_tokenization() {
        let text = "Hello, world! This is a test. Another sentence.";
        let highlight = make_highlight(text);
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert_eq!(result.summary, "Hello, world!");
        assert!(result.tags.contains(&"hello".to_string()));
        assert!(result.tags.contains(&"world".to_string()));
        assert!(result.tags.contains(&"test".to_string()));
        assert!(result.tags.contains(&"another".to_string()));
        assert!(result.tags.contains(&"sentence".to_string()));
    }

    #[test]
    fn test_fewer_than_three_tags() {
        let text = "Quantum entanglement.";
        let highlight = make_highlight(text);
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert_eq!(result.tags.len(), 2);
        assert!(result.tags.contains(&"quantum".to_string()));
        assert!(result.tags.contains(&"entanglement".to_string()));
    }

    #[test]
    fn test_mixed_case_tags_are_lowercased() {
        let text = "Rust React Tauri Qwen Model.";
        let highlight = make_highlight(text);
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        for tag in &result.tags {
            assert_eq!(
                tag,
                &tag.to_lowercase(),
                "tag '{}' should be lowercase",
                tag
            );
        }
        assert_eq!(result.tags.len(), 5);
        assert!(result.tags.contains(&"rust".to_string()));
        assert!(result.tags.contains(&"react".to_string()));
        assert!(result.tags.contains(&"tauri".to_string()));
        assert!(result.tags.contains(&"qwen".to_string()));
        assert!(result.tags.contains(&"model".to_string()));
    }

    #[test]
    fn test_connection_suggestion_none() {
        let text = "Standalone highlight with no connections.";
        let existing = vec![];
        let service = FallbackService;
        let result = service.enrich(&make_highlight(text), &existing).unwrap();

        assert!(result.connection_suggestion.is_none());
    }

    #[test]
    fn test_max_chars_truncation() {
        let text = "A".repeat(10000);
        let highlight = make_highlight(&text);
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert!(result.summary.ends_with('…'));
        assert!(result.tags.len() <= 1);
    }

    #[test]
    fn test_input_under_limit_not_truncated() {
        let text = "Short highlight.";
        let highlight = make_highlight(text);
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert_eq!(result.summary, "Short highlight.");
    }

    #[test]
    fn test_exactly_150_chars_no_truncation() {
        let text = "a".repeat(150);
        let highlight = make_highlight(&text);
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert_eq!(result.summary.chars().count(), 150);
        assert!(!result.summary.ends_with('…'));
    }

    #[test]
    fn test_151_chars_truncated() {
        let text = "a".repeat(151);
        let highlight = make_highlight(&text);
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert_eq!(result.summary.chars().count(), 151);
        assert!(result.summary.ends_with('…'));
    }

    #[test]
    fn test_numeric_only_text() {
        let text = "123 456 789 101112";
        let highlight = make_highlight(text);
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert!(!result.tags.is_empty());
        for tag in &result.tags {
            assert!(
                tag.chars().all(|c| c.is_ascii_digit()),
                "tag '{}' should be numeric only",
                tag
            );
        }
    }

    #[test]
    fn test_unicode_emoji_boundary() {
        let text = "Rust 🔥 Tauri 🦀 Relay";
        let highlight = make_highlight(text);
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert!(result.tags.contains(&"rust".to_string()));
        assert!(result.tags.contains(&"tauri".to_string()));
        assert!(result.tags.contains(&"relay".to_string()));
        assert_eq!(result.tags.len(), 3);
    }

    #[test]
    fn test_single_char_words() {
        let text = "a b c dog cat";
        let highlight = make_highlight(text);
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert!(result.tags.contains(&"dog".to_string()));
        assert!(result.tags.contains(&"cat".to_string()));
        assert!(!result.tags.contains(&"a".to_string()));
        assert!(!result.tags.contains(&"b".to_string()));
        assert!(!result.tags.contains(&"c".to_string()));
    }

    #[test]
    fn test_leading_trailing_whitespace() {
        let text = "  \n  Leading highlight text.  \t  ";
        let trimmed = "Leading highlight text.";
        let highlight = make_highlight(text);
        let service = FallbackService;
        let result = service.enrich(&highlight, &[]).unwrap();

        assert_eq!(result.summary, trimmed);
        assert!(result.tags.contains(&"leading".to_string()));
        assert!(result.tags.contains(&"highlight".to_string()));
        assert!(result.tags.contains(&"text".to_string()));
    }
}
