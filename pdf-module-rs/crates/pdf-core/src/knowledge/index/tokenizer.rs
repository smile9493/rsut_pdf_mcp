//! CJK-aware tokenizer for Tantivy full-text search.
//!
//! Wraps Tantivy's built-in `NgramTokenizer` for CJK character n-gram tokenization.
//! Produces unigrams and bigrams for Chinese text matching.
//!
//! ## Future Upgrade
//!
//! To integrate jieba-rs for better segmentation:
//! 1. Add `jieba-tantivy = "0.3"` to Cargo.toml
//! 2. Replace `register_cjk_tokenizer` with jieba's tokenizer registration
//! 3. No schema changes needed

use tantivy::tokenizer::{NgramTokenizer, TextAnalyzer};

/// Create and register a CJK n-gram tokenizer with a Tantivy index.
///
/// Registers as `"cjk_ngram"` — use this name in field options.
/// Produces unigrams (min_size=1) and bigrams (max_size=2) for CJK text.
pub fn register_cjk_tokenizer(index: &tantivy::Index) {
    let analyzer = TextAnalyzer::builder(
        NgramTokenizer::new(1, 2, false)
            .expect("NgramTokenizer with valid params (1, 2, false) should never fail")
    )
    .build();
    index.tokenizers().register("cjk_ngram", analyzer);
}
