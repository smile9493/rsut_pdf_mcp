//! Keyword extractor with Chinese segmentation
//! Corresponds to Python: keyword_extractor.py

use crate::dto::{KeywordMatch, KeywordSearchResult, LineInfo, PageMetadata};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};

/// Pre-compiled regex for English word extraction
static ENGLISH_WORD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[a-zA-Z]{2,}").expect("Invalid regex pattern for English words"));

/// Global regex cache for keyword patterns (lock-free using DashMap)
/// Key: (pattern, case_sensitive), Value: compiled Regex
static REGEX_CACHE: Lazy<DashMap<String, Regex>> = Lazy::new(DashMap::new);

/// Global Jieba instance for Chinese segmentation
static JIEBA: Lazy<jieba_rs::Jieba> = Lazy::new(jieba_rs::Jieba::new);

/// Get or create a cached regex for a keyword (lock-free)
fn get_cached_regex(keyword: &str, case_sensitive: bool) -> Option<Regex> {
    let cache_key = if case_sensitive {
        format!("cs:{}", keyword)
    } else {
        format!("ci:{}", keyword)
    };

    // Try to get from cache first (lock-free read)
    if let Some(re) = REGEX_CACHE.get(&cache_key) {
        return Some(re.clone());
    }

    // Compile new regex
    let pattern = regex::escape(keyword);
    let re = if case_sensitive {
        Regex::new(&pattern).ok()
    } else {
        Regex::new(&format!("(?i){}", pattern)).ok()
    };

    // Store in cache if valid
    if let Some(ref regex) = re {
        REGEX_CACHE.insert(cache_key, regex.clone());
    }

    re
}

/// Keyword extractor with jieba-rs Chinese segmentation
/// Corresponds to Python: KeywordExtractor
pub struct KeywordExtractor {
    case_sensitive: bool,
}

impl KeywordExtractor {
    /// Create a new keyword extractor
    pub fn new() -> Self {
        Self {
            case_sensitive: false,
        }
    }

    /// Create with case sensitivity option
    pub fn with_case_sensitive(case_sensitive: bool) -> Self {
        Self { case_sensitive }
    }

    /// Search for keywords in pages
    /// Corresponds to Python: KeywordExtractor.search_keywords()
    pub fn search_keywords(
        &self,
        pages: &[PageMetadata],
        keywords: &[String],
        context_length: usize,
    ) -> KeywordSearchResult {
        let mut matches = Vec::new();
        let mut pages_with_matches = HashSet::new();

        for page in pages {
            for keyword in keywords {
                // Use cached regex for better performance
                let re = match get_cached_regex(keyword, self.case_sensitive) {
                    Some(re) => re,
                    None => {
                        tracing::warn!("Invalid regex pattern for keyword: {}", keyword);
                        continue;
                    }
                };

                for mat in re.find_iter(&page.text) {
                    let start = mat.start();
                    let end = mat.end();
                    let ctx_start = start.saturating_sub(context_length);
                    let ctx_end = (end + context_length).min(page.text.len());
                    let context = &page.text[ctx_start..ctx_end];

                    let bbox = Self::find_bbox_for_position(&page.lines, keyword, start);

                    matches.push(KeywordMatch {
                        keyword: keyword.clone(),
                        page_number: page.page_number,
                        text: context.to_string(),
                        bbox,
                        start_index: start,
                        end_index: end,
                        confidence: 1.0,
                    });
                    pages_with_matches.insert(page.page_number);
                }
            }
        }

        KeywordSearchResult {
            keywords: keywords.to_vec(),
            total_matches: matches.len(),
            pages_with_matches: pages_with_matches.into_iter().collect(),
            matches,
        }
    }

    /// Extract keywords by frequency
    /// Corresponds to Python: KeywordExtractor.extract_keywords_by_frequency()
    pub fn extract_by_frequency(
        &self,
        text: &str,
        min_length: usize,
        max_length: usize,
        top_n: usize,
    ) -> Vec<(String, usize)> {
        let mut word_counts: HashMap<String, usize> = HashMap::new();

        // Chinese segmentation using cached jieba instance
        let words = JIEBA.cut(text, true);
        for word in words {
            let len = word.chars().count();
            if len >= min_length && len <= max_length && !word.trim().is_empty() {
                *word_counts.entry(word.to_string()).or_insert(0) += 1;
            }
        }

        // English word extraction
        for mat in ENGLISH_WORD_REGEX.find_iter(text) {
            let word = mat.as_str();
            let len = word.chars().count();
            if len >= min_length && len <= max_length {
                *word_counts
                    .entry(word.to_string().to_lowercase())
                    .or_insert(0) += 1;
            }
        }

        // Sort by frequency and take top N
        let mut sorted: Vec<_> = word_counts.into_iter().collect();
        sorted.sort_by_key(|a| std::cmp::Reverse(a.1));
        sorted.into_iter().take(top_n).collect()
    }

    /// Highlight keywords in text
    /// Corresponds to Python: KeywordExtractor.highlight_keywords_in_text()
    pub fn highlight(&self, text: &str, keywords: &[String], prefix: &str, suffix: &str) -> String {
        let mut result = text.to_string();
        for keyword in keywords {
            // Use cached regex for better performance
            let re = match get_cached_regex(keyword, self.case_sensitive) {
                Some(re) => re,
                None => {
                    tracing::warn!("Invalid regex pattern for highlight: {}", keyword);
                    continue;
                }
            };
            result = re
                .replace_all(&result, &format!("{}{}{}", prefix, keyword, suffix))
                .to_string();
        }
        result
    }

    /// Find bbox for a keyword at a given position
    fn find_bbox_for_position(
        lines: &[LineInfo],
        text: &str,
        _position: usize,
    ) -> Option<(f64, f64, f64, f64)> {
        for line in lines {
            if line.text.contains(text)
                && line.bbox.len() == 4 {
                    return Some((line.bbox[0], line.bbox[1], line.bbox[2], line.bbox[3]));
                }
        }
        None
    }
}

impl Default for KeywordExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_keywords() {
        let pages = vec![PageMetadata {
            page_number: 1,
            text: "This is a test document with test content.".to_string(),
            bbox: None,
            lines: vec![],
        }];

        let extractor = KeywordExtractor::new();
        let result = extractor.search_keywords(&pages, &["test".to_string()], 10);

        assert_eq!(result.total_matches, 2);
        assert_eq!(result.pages_with_matches, vec![1]);
    }

    #[test]
    fn test_extract_by_frequency() {
        let text = "hello world hello rust world world";
        let extractor = KeywordExtractor::new();
        let result = extractor.extract_by_frequency(text, 2, 20, 10);

        // Verify result is not empty
        assert!(!result.is_empty());

        // Verify "world" is in the results
        // Note: jieba may segment differently, so we just check presence
        assert!(result.iter().any(|(word, _)| word == "world"));

        // Verify "hello" is in the results
        assert!(result.iter().any(|(word, _)| word == "hello"));

        // Verify "rust" is in the results
        assert!(result.iter().any(|(word, _)| word == "rust"));
    }

    #[test]
    fn test_highlight() {
        let text = "This is a test document.";
        let extractor = KeywordExtractor::new();
        let result = extractor.highlight(text, &["test".to_string()], "**", "**");

        assert_eq!(result, "This is a **test** document.");
    }

    #[test]
    fn test_chinese_segmentation() {
        let text = "这是一个测试文档测试测试";
        let extractor = KeywordExtractor::new();
        let result = extractor.extract_by_frequency(text, 2, 10, 10);

        assert!(!result.is_empty());
    }
}
