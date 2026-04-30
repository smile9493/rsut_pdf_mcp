//! WASM PDF extraction engine with Arena allocator.
//!
//! Provides efficient memory management for PDF processing in WASM environments
//! using bump-allocated arenas that can be reset per-frame, avoiding memory leaks
//! and reducing fragmentation in the WASM linear memory.
//!
//! # Features
//!
//! - **Arena allocator**: Per-frame batch memory release via `bumpalo`
//! - **Zero-copy interop**: `WasmSlice` for efficient JS-WASM boundary data passing
//! - **Structured errors**: Typed error variants for WASM-friendly error handling

pub mod arena;
pub mod error;
pub mod slice;

pub use arena::WasmPdfEngine;
pub use error::WasmError;
pub use slice::{OwnedSlice, WasmSlice};
