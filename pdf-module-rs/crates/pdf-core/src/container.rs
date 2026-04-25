//! Service container for dependency injection.
//!
//! Provides a centralized registry for PDF engines, storage backends,
//! and tool handlers. Replaces the ad-hoc HashMap wiring previously
//! embedded in `PdfExtractorService`.

use std::collections::HashMap;
use std::sync::Arc;
use crate::engine::PdfEngine;
use crate::error::{PdfModuleError, PdfResult};

/// Macro to register engines with aliases in a concise way.
///
/// # Example
///
/// ```ignore
/// register_engines!(container,
///     LopdfEngine::new()? => ["lopdf", "pymupdf", "fitz"],
///     PdfExtractEngine::new() => ["pdf-extract", "pdfplumber"],
///     PdfiumEngine::new()? => ["pdfium"],
/// );
/// ```
#[macro_export]
macro_rules! register_engines {
    ($container:expr, $( $engine:expr => [ $($alias:expr),+ ] ),+ $(,)? ) => {
        $(
            {
                let engine = std::sync::Arc::new($engine);
                $(
                    $container.register_engine($alias, engine.clone());
                )+
            }
        )+
    };
}

/// Service container for dependency injection.
///
/// Centralizes engine registration, storage backend, and tool handler
/// management so that `PdfExtractorService` and other services receive
/// their dependencies through a single, testable object.
pub struct ServiceContainer {
    /// PDF engine registry (name -> engine instance, supports aliases)
    engines: HashMap<String, Arc<dyn PdfEngine>>,
    /// Default engine name
    default_engine: String,
    /// Fallback engine name
    fallback_engine: String,
}

impl ServiceContainer {
    /// Create an empty container.
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
            default_engine: "lopdf".to_string(),
            fallback_engine: "pdfium".to_string(),
        }
    }

    /// Register a single engine under a given name/alias.
    pub fn register_engine(&mut self, name: &str, engine: Arc<dyn PdfEngine>) {
        self.engines.insert(name.to_string(), engine);
    }

    /// Register an engine under multiple aliases.
    pub fn register_engine_with_aliases(
        &mut self,
        engine: Arc<dyn PdfEngine>,
        aliases: &[&str],
    ) {
        for alias in aliases {
            self.engines.insert(alias.to_string(), engine.clone());
        }
    }

    /// Look up an engine by name or alias.
    pub fn get_engine(&self, name: &str) -> Option<Arc<dyn PdfEngine>> {
        self.engines.get(name).cloned()
    }

    /// Look up an engine, returning an error if not found.
    pub fn get_engine_or_err(&self, name: &str) -> PdfResult<Arc<dyn PdfEngine>> {
        self.engines.get(name).cloned().ok_or_else(|| {
            PdfModuleError::AdapterNotFound(format!(
                "Unknown engine '{}'. Available: {:?}",
                name,
                self.engines.keys().collect::<Vec<_>>()
            ))
        })
    }

    /// Get the default engine.
    pub fn get_default_engine(&self) -> Option<Arc<dyn PdfEngine>> {
        self.get_engine(&self.default_engine)
    }

    /// Get the fallback engine.
    pub fn get_fallback_engine(&self) -> Option<Arc<dyn PdfEngine>> {
        self.get_engine(&self.fallback_engine)
    }

    /// Set the default engine name.
    pub fn set_default_engine(&mut self, engine_id: impl Into<String>) {
        self.default_engine = engine_id.into();
    }

    /// Set the fallback engine name.
    pub fn set_fallback_engine(&mut self, engine_id: impl Into<String>) {
        self.fallback_engine = engine_id.into();
    }

    /// Return the default engine name.
    pub fn default_engine_name(&self) -> &str {
        &self.default_engine
    }

    /// Return the fallback engine name.
    pub fn fallback_engine_name(&self) -> &str {
        &self.fallback_engine
    }

    /// List all registered engine names.
    pub fn list_engines(&self) -> Vec<String> {
        self.engines.keys().cloned().collect()
    }

    /// Number of registered engine entries (including aliases).
    pub fn engine_count(&self) -> usize {
        self.engines.len()
    }
}

impl Default for ServiceContainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{LopdfEngine, PdfExtractEngine};

    #[test]
    fn test_service_container_register_and_get() {
        let mut container = ServiceContainer::new();

        let lopdf: Arc<dyn PdfEngine> = Arc::new(LopdfEngine::new());
        container.register_engine_with_aliases(lopdf, &["lopdf", "pymupdf", "fitz"]);

        assert!(container.get_engine("lopdf").is_some());
        assert!(container.get_engine("pymupdf").is_some());
        assert!(container.get_engine("fitz").is_some());
        assert!(container.get_engine("nonexistent").is_none());
    }

    #[test]
    fn test_service_container_defaults() {
        let mut container = ServiceContainer::new();
        assert_eq!(container.default_engine_name(), "lopdf");
        assert_eq!(container.fallback_engine_name(), "pdfium");

        container.set_default_engine("pdf-extract");
        assert_eq!(container.default_engine_name(), "pdf-extract");
    }

    #[test]
    fn test_register_engines_macro() {
        let mut container = ServiceContainer::new();

        register_engines!(container,
            LopdfEngine::new() => ["lopdf", "pymupdf", "fitz"],
            PdfExtractEngine::new() => ["pdf-extract", "pdfplumber"],
        );

        assert!(container.get_engine("lopdf").is_some());
        assert!(container.get_engine("pymupdf").is_some());
        assert!(container.get_engine("fitz").is_some());
        assert!(container.get_engine("pdf-extract").is_some());
        assert!(container.get_engine("pdfplumber").is_some());
        assert_eq!(container.engine_count(), 5);
    }
}
