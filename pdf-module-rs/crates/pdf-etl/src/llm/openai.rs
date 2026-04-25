//! OpenAI 适配器

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
    build_extraction_prompt, LLMAdapter, LLMRequest, LLMResponse, Message, ResponseFormat,
};
use crate::schema::validator::SchemaValidator;

/// OpenAI 适配器
pub struct OpenAIAdapter {
    client: Client,
    config: LLMConfig,
    base_url: String,
}

impl OpenAIAdapter {
    /// 创建新的 OpenAI 适配器
    pub fn new(config: LLMConfig) -> Result<Self> {
        config.validate()?;

        let base_url = config.get_base_url();

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .build()
            .map_err(|e| EtlError::HttpError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            config,
            base_url,
        })
    }

    /// 构建 OpenAI API 请求体
    fn build_openai_request(&self, request: LLMRequest) -> OpenAIRequestBody {
        let messages: Vec<OpenAIMessage> = request
            .messages
            .into_iter()
            .map(|m| match m {
                Message::System { content } => OpenAIMessage {
                    role: "system".to_string(),
                    content,
                },
                Message::User { content } => OpenAIMessage {
                    role: "user".to_string(),
                    content,
                },
                Message::Assistant { content } => OpenAIMessage {
                    role: "assistant".to_string(),
                    content,
                },
            })
            .collect();

        let response_format = match request.response_format {
            ResponseFormat::Text => None,
            ResponseFormat::JsonObject => Some(OpenAIResponseFormat {
                type_: "json_object".to_string(),
            }),
        };

        OpenAIRequestBody {
            model: self.config.model.clone(),
            messages,
            temperature: Some(request.temperature),
            max_tokens: request.max_tokens,
            response_format,
        }
    }

    /// 调用 OpenAI API
    async fn call_api(&self, request: LLMRequest) -> Result<LLMResponse> {
        let openai_request = self.build_openai_request(request);

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header(
                "Authorization",
                format!("Bearer {}", self.config.api_key.expose_secret()),
            )
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EtlError::LLMError(format!(
                "OpenAI API error: {} - {}",
                status, body
            )));
        }

        let openai_response: OpenAIResponse = response.json().await?;

        let content = openai_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let usage = TokenUsage::new(
            openai_response.usage.prompt_tokens,
            openai_response.usage.completion_tokens,
        );

        Ok(LLMResponse::new(content, usage, openai_response.model))
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
                        "OpenAI API call failed (attempt {}/{}): {}. Retrying in {}ms...",
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
impl LLMAdapter for OpenAIAdapter {
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
        let mut request = LLMRequest::new(vec![Message::user(prompt)])
            .with_temperature(self.config.temperature)
            .with_response_format(ResponseFormat::JsonObject);

        if let Some(max_tokens) = self.config.max_tokens {
            request = request.with_max_tokens(max_tokens);
        }

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
        let max_context_tokens = match self.config.model.as_str() {
            "gpt-4o" => 128000,
            "gpt-4-turbo" | "gpt-4-1106-preview" => 128000,
            "gpt-4" => 8192,
            "gpt-3.5-turbo" => 16385,
            _ => 4096,
        };

        ModelInfo::new(
            "openai".to_string(),
            self.config.model.clone(),
            max_context_tokens,
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
// OpenAI API 数据结构
// ============================================================================

#[derive(Debug, Serialize)]
struct OpenAIRequestBody {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<OpenAIResponseFormat>,
}

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAIResponseFormat {
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageResponse,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessageResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LLMProvider, SecretString};

    #[test]
    fn test_openai_adapter_creation() {
        let config = LLMConfig {
            provider: LLMProvider::OpenAI,
            model: "gpt-4".to_string(),
            api_key: SecretString::from("test-key"),
            base_url: Some("https://api.openai.com/v1".to_string()),
            temperature: 0.0,
            max_tokens: Some(4096),
            timeout: 30,
            deployment_name: None,
            api_version: "2024-02-15-preview".to_string(),
        };

        let adapter = OpenAIAdapter::new(config);
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_model_info() {
        let config = LLMConfig {
            provider: LLMProvider::OpenAI,
            model: "gpt-4o".to_string(),
            api_key: SecretString::from("test-key"),
            base_url: None,
            temperature: 0.0,
            max_tokens: None,
            timeout: 30,
            deployment_name: None,
            api_version: "2024-02-15-preview".to_string(),
        };

        let adapter = OpenAIAdapter::new(config).unwrap();
        let info = adapter.model_info();

        assert_eq!(info.provider, "openai");
        assert_eq!(info.model, "gpt-4o");
        assert_eq!(info.max_context_tokens, 128000);
    }

    #[test]
    fn test_count_tokens() {
        let config = LLMConfig {
            provider: LLMProvider::OpenAI,
            model: "gpt-4".to_string(),
            api_key: SecretString::from("test-key"),
            base_url: None,
            temperature: 0.0,
            max_tokens: None,
            timeout: 30,
            deployment_name: None,
            api_version: "2024-02-15-preview".to_string(),
        };

        let adapter = OpenAIAdapter::new(config).unwrap();
        let tokens = adapter.count_tokens("Hello, world!").unwrap();
        assert!(tokens > 0);
    }
}
