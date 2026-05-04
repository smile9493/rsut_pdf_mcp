//! # Cognitive Index Layer
//!
//! Provides fast discovery and association across knowledge entries.
//!
//! - **FulltextIndex**: Tantivy-based full-text search with CJK support
//! - **GraphIndex**: petgraph-based link graph for neighbor discovery, orphan detection, and concept maps
//! - **CjkTokenizer**: Character n-gram tokenizer for Chinese text

pub mod fulltext;
pub mod graph;
pub mod tokenizer;

pub use fulltext::FulltextIndex;
pub use graph::GraphIndex;
pub use tokenizer::register_cjk_tokenizer;
