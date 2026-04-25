//! 结构化提取器
//!
//! 协调 LLM 转换，将文本转换为结构化数据

use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

use crate::dto::TransformResult;
use crate::error::{EtlError, Result};
use crate::llm::adapter::LLMAdapter;
use crate::schema::SchemaDefinition;

/// 重试策略
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// 最大重试次数
    pub max_retries: u32,
    /// 初始延迟（毫秒）
    pub initial_delay_ms: u64,
    /// 最大延迟（毫秒）
    pub max_delay_ms: u64,
    /// 退避乘数
    pub multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 10000,
            multiplier: 2.0,
        }
    }
}

/// 结构化提取器
pub struct StructuredExtractor {
    llm_adapter: Arc<dyn LLMAdapter>,
    #[allow(dead_code)]
    retry_policy: RetryPolicy,
}

impl StructuredExtractor {
    /// 创建新的结构化提取器
    pub fn new(llm_adapter: Arc<dyn LLMAdapter>) -> Self {
        Self {
            llm_adapter,
            retry_policy: RetryPolicy::default(),
        }
    }

    /// 使用自定义重试策略创建
    pub fn with_retry_policy(llm_adapter: Arc<dyn LLMAdapter>, retry_policy: RetryPolicy) -> Self {
        Self {
            llm_adapter,
            retry_policy,
        }
    }

    /// 转换单个文本块
    pub async fn transform(
        &self,
        text: &str,
        schema: &SchemaDefinition,
        prompt_template: Option<&str>,
    ) -> Result<TransformResult> {
        let start = Instant::now();

        info!(
            "Transforming text to schema: {} ({} chars)",
            schema.name,
            text.len()
        );

        // 调用 LLM 进行结构化提取
        let result = self
            .llm_adapter
            .extract_structured(text, &schema.schema, prompt_template)
            .await?;

        // 如果验证失败，尝试重试一次
        if !result.is_valid {
            warn!("Initial transformation failed validation, retrying...");

            let retry_result = self
                .llm_adapter
                .extract_structured(text, &schema.schema, prompt_template)
                .await?;

            if retry_result.is_valid {
                info!("Retry succeeded");
                return Ok(retry_result);
            } else {
                warn!("Retry also failed validation");
                return Ok(retry_result);
            }
        }

        let processing_time_ms = start.elapsed().as_millis() as u64;
        debug!(
            "Transformation completed: valid={}, tokens={}/{}, time={}ms",
            result.is_valid, result.input_tokens, result.output_tokens, processing_time_ms
        );

        Ok(result)
    }

    /// 转换多个文本块
    pub async fn transform_chunks(
        &self,
        chunks: &[String],
        schema: &SchemaDefinition,
        prompt_template: Option<&str>,
    ) -> Result<Vec<TransformResult>> {
        let mut results = Vec::with_capacity(chunks.len());

        for (i, chunk) in chunks.iter().enumerate() {
            debug!("Transforming chunk {}/{}", i + 1, chunks.len());
            let result = self.transform(chunk, schema, prompt_template).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// 合并多个转换结果
    pub fn merge_results(results: Vec<TransformResult>) -> Result<TransformResult> {
        if results.is_empty() {
            return Err(EtlError::ValidationError(
                "No results to merge".to_string(),
                vec![],
            ));
        }

        if results.len() == 1 {
            return Ok(results.into_iter().next().unwrap());
        }

        // 合并所有数据
        let mut merged_data = serde_json::Map::new();
        let mut all_valid = true;
        let mut all_errors = Vec::new();
        let mut total_input_tokens = 0;
        let mut total_output_tokens = 0;
        let mut total_time = 0;

        for result in results {
            if !result.is_valid {
                all_valid = false;
                all_errors.extend(result.validation_errors);
            }

            total_input_tokens += result.input_tokens;
            total_output_tokens += result.output_tokens;
            total_time += result.processing_time_ms;

            // 合并数据
            if let serde_json::Value::Object(map) = result.data {
                for (key, value) in map {
                    // 如果键已存在且都是数组，则合并数组
                    if let Some(serde_json::Value::Array(existing)) = merged_data.get(&key) {
                        if let serde_json::Value::Array(new) = &value {
                            let mut merged = existing.clone();
                            merged.extend(new.clone());
                            merged_data.insert(key, serde_json::Value::Array(merged));
                            continue;
                        }
                    }
                    merged_data.insert(key, value);
                }
            }
        }

        let data = serde_json::Value::Object(merged_data);

        if all_valid {
            Ok(TransformResult::valid(
                data,
                total_input_tokens,
                total_output_tokens,
                total_time,
            ))
        } else {
            Ok(TransformResult::invalid(
                data,
                all_errors,
                total_input_tokens,
                total_output_tokens,
                total_time,
            ))
        }
    }

    /// 获取 LLM 适配器
    pub fn llm_adapter(&self) -> &dyn LLMAdapter {
        self.llm_adapter.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LLMConfig, SecretString};
    use crate::dto::ModelInfo;
    use schemars::schema::RootSchema;

    // Mock LLM Adapter for testing
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
                provider: crate::config::LLMProvider::OpenAI,
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
    async fn test_structured_extractor() {
        let adapter = Arc::new(MockLLMAdapter);
        let extractor = StructuredExtractor::new(adapter);

        let schema = SchemaDefinition::from_json(
            r#"{"type": "object", "properties": {"test": {"type": "string"}}}"#,
        )
        .unwrap();

        let result = extractor.transform("test text", &schema, None).await;
        assert!(result.is_ok());

        let transform_result = result.unwrap();
        assert!(transform_result.is_valid);
    }

    #[test]
    fn test_merge_results() {
        let results = vec![
            TransformResult::valid(serde_json::json!({"field1": "value1"}), 10, 20, 100),
            TransformResult::valid(serde_json::json!({"field2": "value2"}), 10, 20, 100),
        ];

        let merged = StructuredExtractor::merge_results(results).unwrap();
        assert!(merged.is_valid);
        assert_eq!(merged.input_tokens, 20);
        assert_eq!(merged.output_tokens, 40);
    }
}
