//! PDF extractor service orchestrator
//! Corresponds to Python: pdf_extractor.py

use crate::cache::ExtractionCache;
use crate::config::ServerConfig;
use crate::dto::{
    AdapterInfo, CacheStats, ExtractOptions, KeywordSearchResult, StructuredExtractionResult,
    TextExtractionResult,
};
use crate::engine::{LopdfEngine, PdfEngine, PdfExtractEngine, PdfiumEngine};
use crate::error::{PdfModuleError, PdfResult};
use crate::keyword::KeywordExtractor;
use crate::metrics::metrics_def;
use crate::validator::FileValidator;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

// ============ Smart Router ============

/// Smart router for automatic engine selection based on document characteristics
pub struct SmartRouter {
    /// Threshold for small documents (pages)
    small_doc_threshold: u32,
    /// Threshold for small files (MB)
    small_file_size_mb: u64,
    /// Route cache to avoid repeated PDF loading
    route_cache: std::sync::Mutex<
        std::collections::HashMap<std::path::PathBuf, (String, std::time::Instant)>,
    >,
    /// Route cache TTL in seconds
    route_cache_ttl: u64,
}

impl SmartRouter {
    pub fn new() -> Self {
        Self {
            small_doc_threshold: 5,
            small_file_size_mb: 1, // Files < 1MB are considered small
            route_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
            route_cache_ttl: 300, // 5 minutes cache TTL
        }
    }

    /// Route to the best engine based on document characteristics
    /// Returns the engine name to use
    pub fn route(&self, file_path: &Path) -> Option<String> {
        // 1. Check route cache first
        {
            let cache = self.route_cache.lock().unwrap();
            if let Some((cached_engine, timestamp)) = cache.get(file_path) {
                if timestamp.elapsed().as_secs() < self.route_cache_ttl {
                    tracing::debug!("Route cache hit for {:?}: {}", file_path, cached_engine);
                    return Some(cached_engine.clone());
                }
            }
        }

        // 2. Quick file size check (avoid loading PDF for small files)
        if let Ok(metadata) = std::fs::metadata(file_path) {
            let file_size_mb = metadata.len() / (1024 * 1024);
            if file_size_mb <= self.small_file_size_mb {
                let engine = "pdf-extract".to_string();
                self.cache_route(file_path, &engine);
                return Some(engine);
            }
        }

        // 3. Load PDF and analyze
        let doc = lopdf::Document::load(file_path).ok()?;
        let page_count = doc.get_pages().len() as u32;

        // Feature 1: Small documents with simple content -> pdf-extract (fastest)
        let engine = if page_count <= self.small_doc_threshold {
            if !Self::detect_complex_layout(&doc) {
                "pdf-extract".to_string()
            } else {
                "lopdf".to_string()
            }
        }
        // Feature 2: Special encoding -> pdfium (most compatible)
        else if Self::detect_special_encoding(&doc) {
            "pdfium".to_string()
        }
        // Default: lopdf (layout-aware)
        else {
            "lopdf".to_string()
        };

        // 4. Cache the route result
        self.cache_route(file_path, &engine);

        Some(engine)
    }

    /// Cache the route result
    fn cache_route(&self, file_path: &Path, engine: &str) {
        let mut cache = self.route_cache.lock().unwrap();
        cache.insert(
            file_path.to_path_buf(),
            (engine.to_string(), std::time::Instant::now()),
        );

        // Clean up expired entries (simple cleanup)
        if cache.len() > 100 {
            let now = std::time::Instant::now();
            cache.retain(|_, (_, timestamp)| {
                now.duration_since(*timestamp).as_secs() < self.route_cache_ttl
            });
        }
    }

    /// Detect if document has complex layout (images, forms, etc.)
    fn detect_complex_layout(doc: &lopdf::Document) -> bool {
        // Check first few pages for XObject (images, forms)
        for (_, page_obj_id) in doc.get_pages().iter().take(3) {
            if let Ok(lopdf::Object::Dictionary(ref dict)) = doc.get_object(*page_obj_id) {
                if let Ok(lopdf::Object::Dictionary(ref res_dict)) = dict.get(b"Resources") {
                    // Check for XObject (images, forms)
                    if res_dict.get(b"XObject").is_ok() {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Detect if document uses special encoding (CIDFont, Type3, etc.)
    fn detect_special_encoding(doc: &lopdf::Document) -> bool {
        // Check for CIDFont or Type3 fonts
        for (_, page_obj_id) in doc.get_pages().iter().take(3) {
            if let Ok(lopdf::Object::Dictionary(ref dict)) = doc.get_object(*page_obj_id) {
                if let Ok(lopdf::Object::Dictionary(ref res_dict)) = dict.get(b"Resources") {
                    if let Ok(lopdf::Object::Dictionary(ref font_dict)) = res_dict.get(b"Font") {
                        for (_, font_obj) in font_dict.iter() {
                            if let Ok((_, lopdf::Object::Dictionary(ref f))) = doc.dereference(font_obj) {
                                // Check for CIDFont or Type3
                                if let Ok(lopdf::Object::Name(ref name)) = f.get(b"Subtype") {
                                    if name == b"CIDFontType0"
                                        || name == b"CIDFontType2"
                                        || name == b"Type3"
                                    {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }
}

impl Default for SmartRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ============ Circuit Breaker ============

use std::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};

/// Circuit breaker state (using atomic for lock-free access)
const CIRCUIT_CLOSED: u8 = 0;
const CIRCUIT_OPEN: u8 = 1;
const CIRCUIT_HALF_OPEN: u8 = 2;

/// Engine state for circuit breaker (lock-free)
struct EngineBreaker {
    consecutive_failures: AtomicU32,
    last_failure_time: AtomicU64, // Unix timestamp in seconds
    state: AtomicU8,
}

impl EngineBreaker {
    fn new() -> Self {
        Self {
            consecutive_failures: AtomicU32::new(0),
            last_failure_time: AtomicU64::new(0),
            state: AtomicU8::new(CIRCUIT_CLOSED),
        }
    }
}

/// Circuit breaker for engine failure protection (lock-free implementation)
pub struct CircuitBreaker {
    /// Failure threshold before opening circuit
    failure_threshold: u32,
    /// Cooldown period before attempting recovery
    cooldown: Duration,
    /// Per-engine breakers
    breakers: std::sync::Mutex<HashMap<String, EngineBreaker>>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, cooldown: Duration) -> Self {
        Self {
            failure_threshold,
            cooldown,
            breakers: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Register an engine
    pub fn register_engine(&mut self, engine_name: &str) {
        let mut breakers = self.breakers.lock().unwrap();
        breakers.insert(engine_name.to_string(), EngineBreaker::new());
    }

    /// Record a successful operation
    pub fn record_success(&self, engine: &str) {
        let breakers = self.breakers.lock().unwrap();
        if let Some(breaker) = breakers.get(engine) {
            breaker.consecutive_failures.store(0, Ordering::Relaxed);
            breaker.state.store(CIRCUIT_CLOSED, Ordering::Relaxed);
            metrics_def::circuit_breaker_state(engine, "closed");
        }
    }

    /// Record a failed operation
    pub fn record_failure(&self, engine: &str) {
        let breakers = self.breakers.lock().unwrap();
        if let Some(breaker) = breakers.get(engine) {
            let failures = breaker.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;

            // Update last failure time (current Unix timestamp)
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            breaker.last_failure_time.store(now, Ordering::Relaxed);

            if failures >= self.failure_threshold {
                breaker.state.store(CIRCUIT_OPEN, Ordering::Relaxed);
                metrics_def::circuit_breaker_state(engine, "open");
            }
        }
    }

    /// Check if engine is available (not in open state) - lock-free check
    pub fn is_available(&self, engine: &str) -> bool {
        let breakers = self.breakers.lock().unwrap();
        if let Some(breaker) = breakers.get(engine) {
            let state = breaker.state.load(Ordering::Relaxed);
            match state {
                CIRCUIT_CLOSED => true,
                CIRCUIT_OPEN => {
                    // Check if cooldown has passed
                    let last_failure = breaker.last_failure_time.load(Ordering::Relaxed);
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    if now - last_failure >= self.cooldown.as_secs() {
                        // Transition to half-open
                        breaker.state.store(CIRCUIT_HALF_OPEN, Ordering::Relaxed);
                        metrics_def::circuit_breaker_state(engine, "half_open");
                        true
                    } else {
                        false
                    }
                }
                CIRCUIT_HALF_OPEN => true,
                _ => true,
            }
        } else {
            true
        }
    }
}

// ============ PDF Extractor Service ============

/// PDF extractor service orchestrator
/// Corresponds to Python: PDFExtractor
pub struct PdfExtractorService {
    engines: HashMap<String, Arc<dyn PdfEngine>>,
    default_engine: String,
    fallback_engine: String,
    validator: FileValidator,
    cache: Option<ExtractionCache>,
    keyword_extractor: KeywordExtractor,
    router: SmartRouter,
    circuit_breaker: CircuitBreaker,
    enable_smart_routing: bool,
}

impl PdfExtractorService {
    /// Create a new extractor service from configuration
    pub fn new(config: &ServerConfig) -> PdfResult<Self> {
        let mut engines: HashMap<String, Arc<dyn PdfEngine>> = HashMap::new();

        // Register LopdfEngine with aliases
        let lopdf = Arc::new(LopdfEngine::new());
        engines.insert("lopdf".into(), lopdf.clone());
        engines.insert("pymupdf".into(), lopdf.clone());
        engines.insert("fitz".into(), lopdf);

        // Register PdfExtractEngine with alias
        let pdf_extract = Arc::new(PdfExtractEngine::new());
        engines.insert("pdf-extract".into(), pdf_extract.clone());
        engines.insert("pdfplumber".into(), pdf_extract);

        // Register PdfiumEngine
        let pdfium = Arc::new(PdfiumEngine::new()?);
        engines.insert("pdfium".into(), pdfium);

        // Initialize cache if enabled
        let cache = if config.cache.enabled {
            Some(ExtractionCache::new(
                config.cache.max_size,
                config.cache.ttl_seconds,
            ))
        } else {
            None
        };

        // Initialize circuit breaker
        let mut circuit_breaker = CircuitBreaker::new(5, Duration::from_secs(60));
        circuit_breaker.register_engine("lopdf");
        circuit_breaker.register_engine("pdf-extract");
        circuit_breaker.register_engine("pdfium");

        Ok(Self {
            engines,
            default_engine: "lopdf".to_string(), // Use default adapter
            fallback_engine: "pdfium".into(),
            validator: FileValidator::new(config.security.max_file_size_mb as u32),
            cache,
            keyword_extractor: KeywordExtractor::new(),
            router: SmartRouter::new(),
            circuit_breaker,
            enable_smart_routing: true,
        })
    }

    /// Get an engine by name (supports aliases)
    pub fn get_engine(&self, name: &str) -> PdfResult<Arc<dyn PdfEngine>> {
        self.engines.get(name).cloned().ok_or_else(|| {
            PdfModuleError::AdapterNotFound(format!(
                "Unknown engine '{}'. Available: {:?}",
                name,
                self.engines.keys().collect::<Vec<_>>()
            ))
        })
    }

    /// Select engine using smart routing or specified name
    fn select_engine(&self, file_path: &Path, engine_name: Option<&str>) -> String {
        match engine_name {
            Some(name) => name.to_string(),
            None if self.enable_smart_routing => self
                .router
                .route(file_path)
                .unwrap_or_else(|| self.default_engine.clone()),
            None => self.default_engine.clone(),
        }
    }

    /// Extract plain text from PDF
    /// Corresponds to Python: PDFExtractor.extract_text()
    pub async fn extract_text(
        &self,
        file_path: &Path,
        engine_name: Option<&str>,
    ) -> PdfResult<TextExtractionResult> {
        let engine_name = self.select_engine(file_path, engine_name);
        let start = Instant::now();

        // Validate file
        let file_info = self.validator.validate(file_path)?;

        // Check cache
        if let Some(ref cache) = self.cache {
            if let Some(cached) = cache.get(file_path, &engine_name)? {
                tracing::debug!("Cache hit for {:?}", file_path);
                return Ok(cached);
            }
        }

        // Check circuit breaker
        let use_fallback = !self.circuit_breaker.is_available(&engine_name);
        let actual_engine = if use_fallback {
            tracing::warn!(
                engine = %engine_name,
                "Circuit breaker open, using fallback engine {}",
                self.fallback_engine
            );
            &self.fallback_engine
        } else {
            &engine_name
        };

        // Get engine
        let engine = self.get_engine(actual_engine)?;

        // Extract with fallback
        let result = match engine.extract_text(file_path).await {
            Ok(r) => {
                self.circuit_breaker.record_success(actual_engine);
                metrics_def::extraction_total(actual_engine, "success");
                r
            }
            Err(e) => {
                tracing::warn!(
                    engine = %actual_engine,
                    error = %e,
                    "Primary engine failed, attempting fallback to {}",
                    self.fallback_engine
                );
                self.circuit_breaker.record_failure(actual_engine);
                metrics_def::extraction_total(actual_engine, "failure");

                // Fallback to pdfium if not already using it
                if actual_engine != &self.fallback_engine {
                    let fallback = self.get_engine(&self.fallback_engine)?;
                    fallback.extract_text(file_path).await?
                } else {
                    return Err(e);
                }
            }
        };

        // Record metrics
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        metrics_def::extraction_duration_ms(actual_engine, elapsed);
        metrics_def::file_size_mb(file_info.file_size_mb);
        metrics_def::route_distribution(actual_engine);

        // Write to cache
        if let Some(ref cache) = self.cache {
            cache.set(file_path, &engine_name, &result)?;
        }

        Ok(result)
    }

    /// Extract structured data from PDF
    /// Corresponds to Python: PDFExtractor.extract_structured()
    pub async fn extract_structured(
        &self,
        file_path: &Path,
        engine_name: Option<&str>,
        options: &ExtractOptions,
    ) -> PdfResult<StructuredExtractionResult> {
        let engine_name = self.select_engine(file_path, engine_name);
        let start = Instant::now();

        // Validate file
        let file_info = self.validator.validate(file_path)?;

        // Check circuit breaker
        let use_fallback = !self.circuit_breaker.is_available(&engine_name);
        let actual_engine = if use_fallback {
            &self.fallback_engine
        } else {
            &engine_name
        };

        // Get engine
        let engine = self.get_engine(actual_engine)?;

        // Extract with fallback
        let result = match engine.extract_structured(file_path, options).await {
            Ok(r) => {
                self.circuit_breaker.record_success(actual_engine);
                metrics_def::extraction_total(actual_engine, "success");
                r
            }
            Err(e) => {
                tracing::warn!(
                    engine = %actual_engine,
                    error = %e,
                    "Primary engine failed for structured extraction, fallback to {}",
                    self.fallback_engine
                );
                self.circuit_breaker.record_failure(actual_engine);
                metrics_def::extraction_total(actual_engine, "failure");

                if actual_engine != &self.fallback_engine {
                    let fallback = self.get_engine(&self.fallback_engine)?;
                    fallback.extract_structured(file_path, options).await?
                } else {
                    return Err(e);
                }
            }
        };

        // Record metrics
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        metrics_def::extraction_duration_ms(actual_engine, elapsed);
        metrics_def::file_size_mb(file_info.file_size_mb);
        metrics_def::route_distribution(actual_engine);

        Ok(result)
    }

    /// Get page count
    /// Corresponds to Python: PDFExtractor.get_page_count()
    pub async fn get_page_count(&self, file_path: &Path) -> PdfResult<u32> {
        self.validator.validate(file_path)?;
        let engine = self.get_engine(&self.default_engine)?;
        engine.get_page_count(file_path).await
    }

    /// Search keywords in PDF
    pub async fn search_keywords(
        &self,
        file_path: &Path,
        keywords: &[String],
        context_length: usize,
    ) -> PdfResult<KeywordSearchResult> {
        self.search_keywords_with_options(file_path, keywords, context_length, false)
            .await
    }

    /// Search keywords in PDF with full options
    pub async fn search_keywords_with_options(
        &self,
        file_path: &Path,
        keywords: &[String],
        context_length: usize,
        case_sensitive: bool,
    ) -> PdfResult<KeywordSearchResult> {
        let start = Instant::now();

        // Extract structured data first
        let structured = self
            .extract_structured(file_path, None, &ExtractOptions::default())
            .await?;

        // Search in pages with case sensitivity option
        let extractor = KeywordExtractor::with_case_sensitive(case_sensitive);
        let result = extractor.search_keywords(&structured.pages, keywords, context_length);

        // Record metrics
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        metrics_def::keyword_search_duration_ms(elapsed);

        Ok(result)
    }

    /// Extract keywords by frequency
    pub async fn extract_keywords(
        &self,
        file_path: &Path,
        min_length: usize,
        max_length: usize,
        top_n: usize,
    ) -> PdfResult<Vec<(String, usize)>> {
        let start = Instant::now();

        // Extract text first
        let text_result = self.extract_text(file_path, None).await?;

        // Extract keywords
        let result = self.keyword_extractor.extract_by_frequency(
            &text_result.extracted_text,
            min_length,
            max_length,
            top_n,
        );

        // Record metrics
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        metrics_def::keyword_search_duration_ms(elapsed);

        Ok(result)
    }

    /// List available engines
    /// Corresponds to Python: PDFExtractor.list_adapters()
    pub fn list_engines(&self) -> Vec<AdapterInfo> {
        // Only return primary engines, not aliases
        let primary_engines = ["lopdf", "pdf-extract", "pdfium"];
        primary_engines
            .iter()
            .filter_map(|name| {
                self.engines.get(*name).map(|engine| AdapterInfo {
                    id: engine.id().to_string(),
                    name: engine.name().to_string(),
                    description: engine.description().to_string(),
                })
            })
            .collect()
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        if let Some(ref cache) = self.cache {
            let stats = cache.stats();
            CacheStats {
                size: stats["size"].as_u64().unwrap_or(0),
                max_size: stats["max_size"].as_u64().unwrap_or(0),
                hits: stats["hits"].as_u64().unwrap_or(0),
                misses: stats["misses"].as_u64().unwrap_or(0),
                hit_rate: stats["hit_rate"].as_f64().unwrap_or(0.0),
            }
        } else {
            CacheStats {
                size: 0,
                max_size: 0,
                hits: 0,
                misses: 0,
                hit_rate: 0.0,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let config = ServerConfig::default();
        let service = PdfExtractorService::new(&config).unwrap();

        let engines = service.list_engines();
        assert!(engines.len() >= 3); // At least lopdf, pdf-extract, pdfium
    }

    #[test]
    fn test_engine_aliases() {
        let config = ServerConfig::default();
        let service = PdfExtractorService::new(&config).unwrap();

        // Test that aliases work
        assert!(service.get_engine("pymupdf").is_ok());
        assert!(service.get_engine("fitz").is_ok());
        assert!(service.get_engine("pdfplumber").is_ok());
        assert!(service.get_engine("nonexistent").is_err());
    }

    #[test]
    fn test_circuit_breaker() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(10));
        cb.register_engine("test");

        // Initially available
        assert!(cb.is_available("test"));

        // Record failures
        cb.record_failure("test");
        assert!(cb.is_available("test"));
        cb.record_failure("test");
        assert!(cb.is_available("test"));
        cb.record_failure("test");

        // Now should be open
        assert!(!cb.is_available("test"));

        // Record success
        cb.record_success("test");
        assert!(cb.is_available("test"));
    }

    #[test]
    fn test_smart_router() {
        let router = SmartRouter::new();
        // Without a real PDF file, route returns None
        // This test just verifies the router can be created
        let _ = router;
    }
}
