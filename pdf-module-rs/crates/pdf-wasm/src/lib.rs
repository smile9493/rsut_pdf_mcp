//! WASM PDF extraction engine with Arena allocator.
//!
//! Provides efficient memory management for PDF processing in WASM environments
//! using bump-allocated arenas that can be reset per-frame, avoiding memory leaks
//! and reducing fragmentation in the WASM linear memory.
//!
//! # Features
//!
//! - **Arena allocator**: Per-frame batch memory release via `bumpalo`
//! - **Global allocator**: `talc` for reduced binary size (~10KB → ~1KB)
//! - **Zero-copy interop**: `WasmSlice` for efficient JS-WASM boundary data passing
//! - **Structured errors**: Typed error variants for WASM-friendly error handling
//! - **Tracing support**: Optional `tracing-wasm` for logging

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(clippy::all)]
#![warn(clippy::await_holding_lock)]
#![warn(clippy::await_holding_refcell_ref)]
#![warn(clippy::large_stack_frames)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::todo)]
#![warn(clippy::dbg_macro)]
#![cfg_attr(not(test), warn(clippy::unwrap_used))]

#[cfg(feature = "wasm")]
#[global_allocator]
static ALLOC: talc::TalckWasm = talc::TalckWasm::new();

pub mod arena;
pub mod error;
pub mod slice;

pub use arena::WasmPdfEngine;
pub use error::WasmError;
pub use slice::{OwnedSlice, WasmSlice};

/// Initialize WASM panic hook and tracing for better error messages.
///
/// This function should be called once when the WASM module is loaded.
/// It installs:
/// - A panic hook that logs panic messages to the JavaScript console
/// - A tracing subscriber that sends logs to the browser console
#[cfg(feature = "wasm")]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
}
