//! ETL 流水线
//!
//! 协调提取、转换、加载的完整流程

use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};

use crate::config::{DatabaseConfig, ExtractionConfig, LLMConfig};
use crate::database::adapter::DatabaseAdapter;
use crate::database::factory::DatabaseAdapterFactory;
use crate::dto::{ETLResult, ExtractionResult, SaveResult, TransformResult};
use crate::error::Result;
use crate::llm::adapter::LLMAdapter;
use crate::llm::factory::LLMAdapterFactory;
use crate::pipeline::extraction_engine::ExtractionService;
use crate::pipeline::structured_extractor::StructuredExtractor;
use crate::schema::SchemaDefinition;

/// ETL 流水线实现
pub struct ETLPipelineImpl {
    extraction_service: ExtractionService,
    llm_adapter: Arc<dyn LLMAdapter>,
    db_adapter: Option<Arc<dyn DatabaseAdapter>>,
}

impl ETLPipelineImpl {
    /// 创建新的 ETL 流水线
    pub async fn new(
        extraction_config: ExtractionConfig,
        llm_config: LLMConfig,
        db_config: Option<DatabaseConfig>,
    ) -> Result<Self> {
        let extraction_service = ExtractionService::new(extraction_config)?;
        let llm_adapter = LLMAdapterFactory::create(llm_config)?;

        let db_adapter = if let Some(config) = db_config {
            Some(DatabaseAdapterFactory::create(config).await?)
        } else {
            None
        };

        Ok(Self {
            extraction_service,
            llm_adapter,
            db_adapter,
        })
    }

    /// 使用已有的适配器创建
    pub fn with_adapters(
        extraction_config: ExtractionConfig,
        llm_adapter: Arc<dyn LLMAdapter>,
        db_adapter: Option<Arc<dyn DatabaseAdapter>>,
    ) -> Result<Self> {
        let extraction_service = ExtractionService::new(extraction_config)?;

        Ok(Self {
            extraction_service,
            llm_adapter,
            db_adapter,
        })
    }

    /// 执行完整 ETL 流程
    pub async fn execute(
        &self,
        pdf_path: &str,
        schema: SchemaDefinition,
        prompt_template: Option<&str>,
        save_to_db: bool,
    ) -> Result<ETLResult> {
        let start = Instant::now();

        info!("Starting ETL pipeline for: {}", pdf_path);

        // 步骤 1: 提取文本
        info!("Step 1: Extracting text...");
        let extraction = self.extract_only(pdf_path).await?;

        // 步骤 2: 转换为结构化数据
        info!("Step 2: Transforming to structured data...");
        let transform = self
            .transform_only(&extraction.text, &schema, prompt_template)
            .await?;

        // 步骤 3: 持久化到数据库
        let save = if save_to_db {
            if let Some(ref db_adapter) = self.db_adapter {
                info!("Step 3: Saving to database...");
                let table_name = db_adapter.config().table_name.clone();

                // 确保表存在
                db_adapter
                    .create_table_if_not_exists(&table_name, Some(&schema.schema))
                    .await?;

                // 保存数据
                let save_result = db_adapter
                    .save_with_metadata(
                        &table_name,
                        &transform.data,
                        &schema.name,
                        Some(pdf_path),
                        Some(&serde_json::json!({
                            "page_count": extraction.metadata.page_count,
                            "processing_time_ms": extraction.processing_time_ms,
                        })),
                    )
                    .await?;

                Some(save_result)
            } else {
                debug!("Database adapter not configured, skipping save");
                None
            }
        } else {
            None
        };

        let total_time_ms = start.elapsed().as_millis() as u64;

        info!(
            "ETL pipeline completed: total_time={}ms, tokens={}",
            total_time_ms,
            transform.input_tokens + transform.output_tokens
        );

        Ok(ETLResult::new(extraction, transform, save, total_time_ms))
    }

    /// 仅执行提取
    pub async fn extract_only(&self, pdf_path: &str) -> Result<ExtractionResult> {
        self.extraction_service.extract(pdf_path).await
    }

    /// 仅执行转换
    pub async fn transform_only(
        &self,
        text: &str,
        schema: &SchemaDefinition,
        prompt_template: Option<&str>,
    ) -> Result<TransformResult> {
        let extractor = StructuredExtractor::new(self.llm_adapter.clone());
        extractor.transform(text, schema, prompt_template).await
    }

    /// 仅执行加载
    pub async fn load_only(
        &self,
        data: &serde_json::Value,
        schema_name: &str,
        source_file: Option<&str>,
    ) -> Result<SaveResult> {
        let db_adapter = self.db_adapter.as_ref().ok_or_else(|| {
            crate::error::EtlError::ConfigError("Database adapter not configured".to_string())
        })?;

        let table_name = db_adapter.config().table_name.clone();

        db_adapter
            .save_with_metadata(&table_name, data, schema_name, source_file, None)
            .await
    }

    /// 批量处理多个 PDF
    pub async fn execute_batch(
        &self,
        pdf_paths: &[String],
        schema: SchemaDefinition,
        prompt_template: Option<&str>,
        save_to_db: bool,
    ) -> Result<Vec<ETLResult>> {
        let mut results = Vec::with_capacity(pdf_paths.len());

        for pdf_path in pdf_paths {
            let result = self
                .execute(pdf_path, schema.clone(), prompt_template, save_to_db)
                .await?;
            results.push(result);
        }

        Ok(results)
    }

    /// 获取提取服务
    pub fn extraction_service(&self) -> &ExtractionService {
        &self.extraction_service
    }

    /// 获取 LLM 适配器
    pub fn llm_adapter(&self) -> &dyn LLMAdapter {
        self.llm_adapter.as_ref()
    }

    /// 获取数据库适配器
    pub fn db_adapter(&self) -> Option<&dyn DatabaseAdapter> {
        self.db_adapter.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DatabaseType, LLMProvider, SecretString};
    use crate::dto::ModelInfo;
    use schemars::schema::RootSchema;

    // Mock LLM Adapter
    struct MockLLMAdapter;

    #[async_trait::async_trait]
    impl LLMAdapter for MockLLMAdapter {
        async fn extract_structured(
            &self,
            _text: &str,
            _schema: &RootSchema,
            _prompt_template: Option<&str>,
        ) -> Result<TransformResult> {
            Ok(TransformResult::valid(
                serde_json::json!({"test": "value"}),
                10,
                20,
                100,
            ))
        }

        fn model_info(&self) -> ModelInfo {
            ModelInfo::new("mock".to_string(), "mock".to_string(), 4096)
        }

        fn count_tokens(&self, text: &str) -> Result<usize> {
            Ok(text.len() / 4)
        }

        fn config(&self) -> &LLMConfig {
            static CONFIG: std::sync::OnceLock<LLMConfig> = std::sync::OnceLock::new();
            CONFIG.get_or_init(|| LLMConfig {
                provider: LLMProvider::OpenAI,
                model: "mock".to_string(),
                api_key: SecretString::from("mock"),
                base_url: None,
                temperature: 0.0,
                max_tokens: None,
                timeout: 30,
                deployment_name: None,
                api_version: "2024-02-15-preview".to_string(),
            })
        }
    }

    #[tokio::test]
    async fn test_etl_pipeline_creation() {
        let extraction_config = ExtractionConfig::default();
        let llm_adapter = Arc::new(MockLLMAdapter);

        let pipeline = ETLPipelineImpl::with_adapters(extraction_config, llm_adapter, None);
        assert!(pipeline.is_ok());
    }
}
