//! PDF extractor service orchestrator
//! Corresponds to Python: pdf_extractor.py

use crate::cache::ExtractionCache;
use crate::config::ServerConfig;
use crate::container::ServiceContainer;
use crate::dto::{
    AdapterInfo, CacheStats, ExtractOptions, KeywordSearchResult, StructuredExtractionResult,
    TextExtractionResult,
};
use crate::engine::{LopdfEngine, PdfExtractEngine, PdfiumEngine};
use crate::error::PdfResult;
use crate::keyword::KeywordExtractor;
use crate::metrics::metrics_def;
use crate::validator::FileValidator;
use pdf_common::circuit_breaker::EngineCircuitBreaker;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

// ============ Smart Router ============

/// Smart router for automatic engine selection based on document characteristics
pub struct SmartRouter {
    small_doc_threshold: u32,
    small_file_size_mb: u64,
    route_cache: std::sync::Mutex<
        std::collections::HashMap<std::path::PathBuf, (String, std::time::Instant)>,
    >,
    route_cache_ttl: u64,
}

impl SmartRouter {
    pub fn new() -> Self {
        Self {
            small_doc_threshold: 5,
            small_file_size_mb: 1,
            route_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
            route_cache_ttl: 300,
        }
    }

    /// Route to the best engine based on document characteristics
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

        let engine = if page_count <= self.small_doc_threshold {
            if !Self::detect_complex_layout(&doc) {
                "pdf-extract".to_string()
            } else {
                "lopdf".to_string()
            }
        } else if Self::detect_special_encoding(&doc) {
            "pdfium".to_string()
        } else {
            "lopdf".to_string()
        };

        self.cache_route(file_path, &engine);
        Some(engine)
    }

    fn cache_route(&self, file_path: &Path, engine: &str) {
        let mut cache = self.route_cache.lock().unwrap();
        cache.insert(
            file_path.to_path_buf(),
            (engine.to_string(), std::time::Instant::now()),
        );
        if cache.len() > 100 {
            let now = std::time::Instant::now();
            cache.retain(|_, (_, timestamp)| {
                now.duration_since(*timestamp).as_secs() < self.route_cache_ttl
            });
        }
    }

    fn detect_complex_layout(doc: &lopdf::Document) -> bool {
        for (_, page_obj_id) in doc.get_pages().iter().take(3) {
            if let Ok(lopdf::Object::Dictionary(ref dict)) = doc.get_object(*page_obj_id) {
                if let Ok(lopdf::Object::Dictionary(ref res_dict)) = dict.get(b"Resources") {
                    if res_dict.get(b"XObject").is_ok() {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn detect_special_encoding(doc: &lopdf::Document) -> bool {
        for (_, page_obj_id) in doc.get_pages().iter().take(3) {
            if let Ok(lopdf::Object::Dictionary(ref dict)) = doc.get_object(*page_obj_id) {
                if let Ok(lopdf::Object::Dictionary(ref res_dict)) = dict.get(b"Resources") {
                    if let Ok(lopdf::Object::Dictionary(ref font_dict)) = res_dict.get(b"Font") {
                        for (_, font_obj) in font_dict.iter() {
                            if let Ok((_, lopdf::Object::Dictionary(ref f))) = doc.dereference(font_obj) {
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

// ============ PDF Extractor Service ============

/// PDF extractor service orchestrator.
///
/// Uses `ServiceContainer` for engine registry and `EngineCircuitBreaker`
/// from `pdf-common` for fault tolerance (replacing the previous inline
/// per-engine circuit breaker).
pub struct PdfExtractorService {
    container: ServiceContainer,
    validator: FileValidator,
    cache: Option<ExtractionCache>,
    keyword_extractor: KeywordExtractor,
    router: SmartRouter,
    circuit_breaker: EngineCircuitBreaker,
    enable_smart_routing: bool,
}

impl PdfExtractorService {
    /// Create a new extractor service from configuration.
    pub fn new(config: &ServerConfig) -> PdfResult<Self> {
        // Build container with all engines using the register_engines! macro
        let mut container = ServiceContainer::new();

        crate::register_engines!(container,
            LopdfEngine::new() => ["lopdf", "pymupdf", "fitz"],
            PdfExtractEngine::new() => ["pdf-extract", "pdfplumber"],
            PdfiumEngine::new()? => ["pdfium"],
        );

        // Initialize cache if enabled
        let cache = if config.cache.enabled {
            Some(ExtractionCache::new(
                config.cache.max_size,
                config.cache.ttl_seconds,
            ))
        } else {
            None
        };

        // Initialize circuit breaker using pdf-common
        let circuit_breaker = EngineCircuitBreaker::new(5, Duration::from_secs(60));
        circuit_breaker.register_engine("lopdf");
        circuit_breaker.register_engine("pdf-extract");
        circuit_breaker.register_engine("pdfium");

        Ok(Self {
            container,
            validator: FileValidator::new(config.security.max_file_size_mb as u32),
            cache,
            keyword_extractor: KeywordExtractor::new(),
            router: SmartRouter::new(),
            circuit_breaker,
            enable_smart_routing: true,
        })
    }

    /// Get an engine by name (supports aliases), delegated to container.
    pub fn get_engine(&self, name: &str) -> PdfResult<Arc<dyn crate::engine::PdfEngine>> {
        self.container.get_engine_or_err(name)
    }

    /// Select engine using smart routing or specified name.
    fn select_engine(&self, file_path: &Path, engine_name: Option<&str>) -> String {
        match engine_name {
            Some(name) => name.to_string(),
            None if self.enable_smart_routing => self
                .router
                .route(file_path)
                .unwrap_or_else(|| self.container.default_engine_name().to_string()),
            None => self.container.default_engine_name().to_string(),
        }
    }

    /// Extract plain text from PDF
    pub async fn extract_text(
        &self,
        file_path: &Path,
        engine_name: Option<&str>,
    ) -> PdfResult<TextExtractionResult> {
        let engine_name = self.select_engine(file_path, engine_name);
        let start = Instant::now();

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
                self.container.fallback_engine_name()
            );
            self.container.fallback_engine_name().to_string()
        } else {
            engine_name.clone()
        };

        let engine = self.get_engine(&actual_engine)?;

        let result = match engine.extract_text(file_path).await {
            Ok(r) => {
                self.circuit_breaker.record_success(&actual_engine);
                metrics_def::extraction_total(&actual_engine, "success");
                r
            }
            Err(e) => {
                tracing::warn!(
                    engine = %actual_engine,
                    error = %e,
                    "Primary engine failed, attempting fallback to {}",
                    self.container.fallback_engine_name()
                );
                self.circuit_breaker.record_failure(&actual_engine);
                metrics_def::extraction_total(&actual_engine, "failure");

                let fallback_name = self.container.fallback_engine_name();
                if actual_engine != fallback_name {
                    let fallback = self.get_engine(fallback_name)?;
                    fallback.extract_text(file_path).await?
                } else {
                    return Err(e);
                }
            }
        };

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        metrics_def::extraction_duration_ms(&actual_engine, elapsed);
        metrics_def::file_size_mb(file_info.file_size_mb);
        metrics_def::route_distribution(&actual_engine);

        if let Some(ref cache) = self.cache {
            cache.set(file_path, &engine_name, &result)?;
        }

        Ok(result)
    }

    /// Extract structured data from PDF
    pub async fn extract_structured(
        &self,
        file_path: &Path,
        engine_name: Option<&str>,
        options: &ExtractOptions,
    ) -> PdfResult<StructuredExtractionResult> {
        let engine_name = self.select_engine(file_path, engine_name);
        let start = Instant::now();

        let file_info = self.validator.validate(file_path)?;

        let use_fallback = !self.circuit_breaker.is_available(&engine_name);
        let actual_engine = if use_fallback {
            self.container.fallback_engine_name().to_string()
        } else {
            engine_name.clone()
        };

        let engine = self.get_engine(&actual_engine)?;

        let result = match engine.extract_structured(file_path, options).await {
            Ok(r) => {
                self.circuit_breaker.record_success(&actual_engine);
                metrics_def::extraction_total(&actual_engine, "success");
                r
            }
            Err(e) => {
                tracing::warn!(
                    engine = %actual_engine,
                    error = %e,
                    "Primary engine failed for structured extraction, fallback to {}",
                    self.container.fallback_engine_name()
                );
                self.circuit_breaker.record_failure(&actual_engine);
                metrics_def::extraction_total(&actual_engine, "failure");

                let fallback_name = self.container.fallback_engine_name();
                if actual_engine != fallback_name {
                    let fallback = self.get_engine(fallback_name)?;
                    fallback.extract_structured(file_path, options).await?
                } else {
                    return Err(e);
                }
            }
        };

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        metrics_def::extraction_duration_ms(&actual_engine, elapsed);
        metrics_def::file_size_mb(file_info.file_size_mb);
        metrics_def::route_distribution(&actual_engine);

        Ok(result)
    }

    /// Get page count
    pub async fn get_page_count(&self, file_path: &Path) -> PdfResult<u32> {
        self.validator.validate(file_path)?;
        let engine = self.get_engine(self.container.default_engine_name())?;
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

        let structured = self
            .extract_structured(file_path, None, &ExtractOptions::default())
            .await?;

        let extractor = KeywordExtractor::with_case_sensitive(case_sensitive);
        let result = extractor.search_keywords(&structured.pages, keywords, context_length);

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

        let text_result = self.extract_text(file_path, None).await?;

        let result = self.keyword_extractor.extract_by_frequency(
            &text_result.extracted_text,
            min_length,
            max_length,
            top_n,
        );

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        metrics_def::keyword_search_duration_ms(elapsed);

        Ok(result)
    }

    /// List available primary engines (not aliases)
    pub fn list_engines(&self) -> Vec<AdapterInfo> {
        let primary_engines = ["lopdf", "pdf-extract", "pdfium"];
        primary_engines
            .iter()
            .filter_map(|name| {
                self.container.get_engine(name).map(|engine| AdapterInfo {
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
        assert!(engines.len() >= 3);
    }

    #[test]
    fn test_engine_aliases() {
        let config = ServerConfig::default();
        let service = PdfExtractorService::new(&config).unwrap();

        assert!(service.get_engine("pymupdf").is_ok());
        assert!(service.get_engine("fitz").is_ok());
        assert!(service.get_engine("pdfplumber").is_ok());
        assert!(service.get_engine("nonexistent").is_err());
    }

    #[test]
    fn test_smart_router() {
        let router = SmartRouter::new();
        let _ = router;
    }
}
