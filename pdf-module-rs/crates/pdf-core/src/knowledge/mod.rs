//! # Knowledge Engine
//!
//! AI-native knowledge compilation and reasoning engine.
//! Implements the Karpathy compiler pattern: PDFs → structured Markdown → indexed knowledge.
//!
//! ## Architecture
//!
//! - **KnowledgeEntry**: Standardized front matter for all wiki entries
//! - **HashCache**: Merkle-tree-based incremental change detection
//! - **KnowledgeEngine**: Orchestrates compile, index, and quality operations
//! - **FulltextIndex**: Tantivy-based full-text search
//! - **GraphIndex**: petgraph-based link graph

pub mod entry;
pub mod hash_cache;
pub mod engine;
pub mod quality;
pub mod index;

pub use entry::{KnowledgeEntry, EntryLevel, CompileStatus};
pub use hash_cache::HashCache;
pub use engine::KnowledgeEngine;
pub use quality::QualityReport;
pub use index::{FulltextIndex, GraphIndex};
