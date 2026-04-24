//! Azure OpenAI 适配器

use async_trait::async_trait;
use reqwest::Client;
use schemars::schema::RootSchema;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

use crate::config::LLMConfig;
use crate::dto::{ModelInfo, TokenUsage, TransformResult};
use crate::error::{EtlError, Result};
use crate::llm::adapter::{
    build_extraction_prompt, LLMAdapter, LLMRequest, LLMResponse, Message,
};
use crate::schema::validator::SchemaValidator;

/// Azure OpenAI 适配器
pub struct AzureOpenAIAdapter {
    client: Client,
    config: LLMConfig,
    endpoint: String,
}

impl AzureOpenAIAdapter {
    /// 创建新的 Azure OpenAI 适配器
    pub fn new(config: LLMConfig) -> Result<Self> {
        config.validate()?;

        let deployment_name = config.deployment_name.as_ref().ok_or_else(|| {
            EtlError::ConfigError("deployment_name is required for Azure".to_string())
        })?;

        let base_url = config
            .base_url
            .as_ref()
            .ok_or_else(|| EtlError::ConfigError("base_url is required for Azure".to_string()))?;

        let endpoint = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            base_url, deployment_name, config.api_version
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .build()
            .map_err(|e| EtlError::HttpError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            config,
            endpoint,
        })
    }

    /// 构建 Azure OpenAI API 请求体
    fn build_azure_request(&self, request: LLMRequest) -> AzureRequestBody {
        let messages: Vec<AzureMessage> = request
            .messages
            .into_iter()
            .map(|m| match m {
                Message::System { content } => AzureMessage {
                    role: "system".to_string(),
                    content,
                },
                Message::User { content } => AzureMessage {
                    role: "user".to_string(),
                    content,
                },
                Message::Assistant { content } => AzureMessage {
                    role: "assistant".to_string(),
                    content,
                },
            })
            .collect();

        AzureRequestBody {
            messages,
            temperature: Some(request.temperature),
            max_tokens: request.max_tokens,
        }
    }

    /// 调用 Azure OpenAI API
    async fn call_api(&self, request: LLMRequest) -> Result<LLMResponse> {
        let azure_request = self.build_azure_request(request);

        let response = self
            .client
            .post(&self.endpoint)
            .header("api-key", self.config.api_key.expose_secret())
            .header("Content-Type", "application/json")
            .json(&azure_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EtlError::LLMError(format!(
                "Azure OpenAI API error: {} - {}",
                status, body
            )));
        }

        let azure_response: AzureResponse = response.json().await?;

        let content = azure_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let usage = TokenUsage::new(
            azure_response.usage.prompt_tokens,
            azure_response.usage.completion_tokens,
        );

        Ok(LLMResponse::new(content, usage, azure_response.model))
    }

    /// 带重试的 API 调用
    async fn call_api_with_retry(
        &self,
        request: LLMRequest,
        max_retries: u32,
    ) -> Result<LLMResponse> {
        let mut delay = 1000u64;

        for attempt in 0..=max_retries {
            match self.call_api(request.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) if attempt < max_retries => {
                    warn!(
                        "Azure OpenAI API call failed (attempt {}/{}): {}. Retrying in {}ms...",
                        attempt + 1,
                        max_retries + 1,
                        e,
                        delay
                    );
                    sleep(Duration::from_millis(delay)).await;
                    delay = (delay * 2).min(10000);
                }
                Err(e) => return Err(e),
            }
        }

        unreachable!()
    }
}

#[async_trait]
impl LLMAdapter for AzureOpenAIAdapter {
    async fn extract_structured(
        &self,
        text: &str,
        schema: &RootSchema,
        prompt_template: Option<&str>,
    ) -> Result<TransformResult> {
        let start = std::time::Instant::now();

        // 构建提示词
        let prompt = build_extraction_prompt(text, schema, prompt_template);

        // 构建 LLM 请求
        let request =
            LLMRequest::new(vec![Message::user(prompt)]).with_temperature(self.config.temperature);

        // 调用 API（带重试）
        let response = self.call_api_with_retry(request, 3).await?;

        // 解析 JSON
        let data = response.parse_json()?;

        // 验证 Schema
        let validator = SchemaValidator::new(schema)?;
        let validation_result = validator.validate(&data)?;

        let processing_time_ms = start.elapsed().as_millis() as u64;

        if validation_result.is_valid {
            Ok(TransformResult::valid(
                data,
                response.usage.prompt_tokens,
                response.usage.completion_tokens,
                processing_time_ms,
            ))
        } else {
            Ok(TransformResult::invalid(
                data,
                validation_result.error_messages(),
                response.usage.prompt_tokens,
                response.usage.completion_tokens,
                processing_time_ms,
            ))
        }
    }

    fn model_info(&self) -> ModelInfo {
        ModelInfo::new(
            "azure".to_string(),
            self.config.model.clone(),
            128000, // Azure GPT-4 通常支持 128k
        )
    }

    fn count_tokens(&self, text: &str) -> Result<usize> {
        // 使用 tiktoken 计算 token
        let bpe = tiktoken_rs::cl100k_base()
            .map_err(|e| EtlError::LLMError(format!("Failed to get encoding: {}", e)))?;
        Ok(bpe.encode_with_special_tokens(text).len())
    }

    fn config(&self) -> &LLMConfig {
        &self.config
    }
}

// ============================================================================
// Azure OpenAI API 数据结构
// ============================================================================

#[derive(Debug, Serialize)]
struct AzureRequestBody {
    messages: Vec<AzureMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
}

#[derive(Debug, Serialize)]
struct AzureMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AzureResponse {
    model: String,
    choices: Vec<AzureChoice>,
    usage: AzureUsage,
}

#[derive(Debug, Deserialize)]
struct AzureChoice {
    message: AzureMessageResponse,
}

#[derive(Debug, Deserialize)]
struct AzureMessageResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct AzureUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LLMProvider, SecretString};

    #[test]
    fn test_azure_adapter_creation() {
        let config = LLMConfig {
            provider: LLMProvider::Azure,
            model: "gpt-4".to_string(),
            api_key: SecretString::from("test-key"),
            base_url: Some("https://test.openai.azure.com".to_string()),
            temperature: 0.0,
            max_tokens: Some(4096),
            timeout: 30,
            deployment_name: Some("gpt-4-deployment".to_string()),
            api_version: "2024-02-15-preview".to_string(),
        };

        let adapter = AzureOpenAIAdapter::new(config);
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_azure_adapter_missing_deployment() {
        let config = LLMConfig {
            provider: LLMProvider::Azure,
            model: "gpt-4".to_string(),
            api_key: SecretString::from("test-key"),
            base_url: Some("https://test.openai.azure.com".to_string()),
            temperature: 0.0,
            max_tokens: None,
            timeout: 30,
            deployment_name: None,
            api_version: "2024-02-15-preview".to_string(),
        };

        let adapter = AzureOpenAIAdapter::new(config);
        assert!(adapter.is_err());
    }
}
