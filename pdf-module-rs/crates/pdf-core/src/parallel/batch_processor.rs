//! Batch processor for parallel PDF extraction.
//!
//! Uses async streams with `futures::stream::buffer_unordered` for concurrent
//! file processing, bridging sync pdfium calls safely.

use crate::dto::{ExtractOptions, StructuredExtractionResult};
use crate::error::{PdfModuleError, PdfResult};
use crate::extractor::McpPdfPipeline;
use futures::stream::{self, StreamExt};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during batch processing.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BatchError {
    #[error("Extraction failed: {0}")]
    Extraction(#[from] PdfModuleError),
}

/// Configuration for batch PDF processing.
///
/// # Example
///
/// ```no_run
/// use pdf_core::parallel::BatchConfig;
///
/// let config = BatchConfig {
///     max_files_parallel: 4,
///     max_pages_parallel: 8,
///     chunk_size: 20,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of files to process in parallel.
    /// Default: number of CPU cores.
    pub max_files_parallel: usize,

    /// Maximum number of pages to extract in parallel per file.
    /// Default: 4.
    pub max_pages_parallel: usize,

    /// Number of files to process in each chunk for progress reporting.
    /// Default: 10.
    pub chunk_size: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_files_parallel: num_cpus::get(),
            max_pages_parallel: 4,
            chunk_size: 10,
        }
    }
}

/// Batch processor for parallel PDF extraction.
///
/// Uses async streams with `buffer_unordered` for concurrent file processing,
/// avoiding the deadlock risks of mixing Rayon and Tokio runtimes directly.
pub struct BatchProcessor {
    pipeline: Arc<McpPdfPipeline>,
    config: BatchConfig,
}

impl BatchProcessor {
    pub fn new(pipeline: Arc<McpPdfPipeline>, config: BatchConfig) -> Self {
        Self { pipeline, config }
    }

    /// Process multiple PDF files in parallel using async approach.
    ///
    /// Uses `buffer_unordered` to process files concurrently up to the
    /// configured parallelism limit.
    pub async fn process_batch_async(
        &self,
        files: Vec<PathBuf>,
        options: ExtractOptions,
    ) -> Result<Vec<(PathBuf, PdfResult<StructuredExtractionResult>)>, BatchError> {
        let pipeline = Arc::clone(&self.pipeline);

        let results = stream::iter(files)
            .map(|file_path| {
                let pipeline = Arc::clone(&pipeline);
                let options = options.clone();
                async move {
                    let result = pipeline.extract_structured(&file_path, &options).await;
                    (file_path, result)
                }
            })
            .buffer_unordered(self.config.max_files_parallel)
            .collect::<Vec<_>>()
            .await;

        Ok(results)
    }

    /// Process batch with progress callback.
    ///
    /// # Arguments
    ///
    /// * `files` - List of PDF file paths to process
    /// * `options` - Extraction options
    /// * `progress_callback` - Called with (completed_count, total_count) after each file
    pub async fn process_batch_with_progress<F>(
        &self,
        files: Vec<PathBuf>,
        options: ExtractOptions,
        mut progress_callback: F,
    ) -> Result<Vec<(PathBuf, PdfResult<StructuredExtractionResult>)>, BatchError>
    where
        F: FnMut(usize, usize) + Send,
    {
        let pipeline = Arc::clone(&self.pipeline);
        let total = files.len();
        let processed = std::sync::atomic::AtomicUsize::new(0);

        let results = stream::iter(files)
            .map(|file_path| {
                let pipeline = Arc::clone(&pipeline);
                let options = options.clone();
                let processed = &processed;
                async move {
                    let result = pipeline.extract_structured(&file_path, &options).await;
                    let count = processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    (file_path, result, count + 1)
                }
            })
            .buffer_unordered(self.config.max_files_parallel)
            .collect::<Vec<_>>()
            .await;

        let mut final_results = Vec::new();
        for (file_path, result, count) in results {
            progress_callback(count, total);
            final_results.push((file_path, result));
        }

        Ok(final_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.max_files_parallel, num_cpus::get());
        assert_eq!(config.max_pages_parallel, 4);
        assert_eq!(config.chunk_size, 10);
    }

    #[test]
    fn test_batch_config_custom() {
        let config = BatchConfig {
            max_files_parallel: 2,
            max_pages_parallel: 1,
            chunk_size: 5,
        };
        assert_eq!(config.max_files_parallel, 2);
        assert_eq!(config.max_pages_parallel, 1);
        assert_eq!(config.chunk_size, 5);
    }

    #[tokio::test]
    async fn test_batch_processor_empty_input() {
        use crate::config::ServerConfig;

        let config = ServerConfig::default();
        let pipeline = Arc::new(McpPdfPipeline::new(&config).unwrap());
        let batch_config = BatchConfig::default();
        let processor = BatchProcessor::new(pipeline, batch_config);

        let result = processor
            .process_batch_async(vec![], ExtractOptions::default())
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_batch_processor_with_progress_empty() {
        use crate::config::ServerConfig;

        let config = ServerConfig::default();
        let pipeline = Arc::new(McpPdfPipeline::new(&config).unwrap());
        let batch_config = BatchConfig::default();
        let processor = BatchProcessor::new(pipeline, batch_config);

        let mut call_count = 0;
        let result = processor
            .process_batch_with_progress(vec![], ExtractOptions::default(), |_done, _total| {
                call_count += 1;
            })
            .await;
        assert!(result.is_ok());
        assert_eq!(call_count, 0);
    }
}
