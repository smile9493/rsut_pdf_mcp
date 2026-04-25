//! Ollama 本地模型适配器

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
use crate::llm::adapter::{build_extraction_prompt, LLMAdapter, LLMRequest, LLMResponse, Message};
use crate::schema::validator::SchemaValidator;

/// Ollama 适配器
pub struct OllamaAdapter {
    client: Client,
    config: LLMConfig,
    base_url: String,
}

impl OllamaAdapter {
    /// 创建新的 Ollama 适配器
    pub fn new(config: LLMConfig) -> Result<Self> {
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

    /// 构建 Ollama API 请求体
    fn build_ollama_request(&self, request: LLMRequest) -> OllamaRequestBody {
        let messages: Vec<OllamaMessage> = request
            .messages
            .into_iter()
            .map(|m| match m {
                Message::System { content } => OllamaMessage {
                    role: "system".to_string(),
                    content,
                },
                Message::User { content } => OllamaMessage {
                    role: "user".to_string(),
                    content,
                },
                Message::Assistant { content } => OllamaMessage {
                    role: "assistant".to_string(),
                    content,
                },
            })
            .collect();

        OllamaRequestBody {
            model: self.config.model.clone(),
            messages,
            format: "json".to_string(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(request.temperature),
                num_predict: request.max_tokens,
            }),
        }
    }

    /// 调用 Ollama API
    async fn call_api(&self, request: LLMRequest) -> Result<LLMResponse> {
        let ollama_request = self.build_ollama_request(request);

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .header("Content-Type", "application/json")
            .json(&ollama_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EtlError::LLMError(format!(
                "Ollama API error: {} - {}",
                status, body
            )));
        }

        let ollama_response: OllamaResponse = response.json().await?;

        let content = ollama_response.message.content;

        // Ollama 的 token 统计可能不准确，使用估算
        let prompt_tokens = self.count_tokens(
            &ollama_request
                .messages
                .iter()
                .map(|m| m.content.clone())
                .collect::<Vec<_>>()
                .join(" "),
        )?;
        let completion_tokens = self.count_tokens(&content)?;

        let usage = TokenUsage::new(prompt_tokens, completion_tokens);

        Ok(LLMResponse::new(content, usage, self.config.model.clone()))
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
                        "Ollama API call failed (attempt {}/{}): {}. Retrying in {}ms...",
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
impl LLMAdapter for OllamaAdapter {
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
        // Ollama 模型的上下文长度取决于具体模型
        let max_context_tokens = match self.config.model.as_str() {
            "llama3" | "llama3:8b" => 8192,
            "llama3:70b" => 8192,
            "mistral" => 32768,
            "mixtral" => 32768,
            "codellama" => 16384,
            _ => 4096,
        };

        ModelInfo::new(
            "ollama".to_string(),
            self.config.model.clone(),
            max_context_tokens,
        )
    }

    fn count_tokens(&self, text: &str) -> Result<usize> {
        // Ollama 使用简单的字符估算（约 4 字符 = 1 token）
        Ok(text.chars().count() / 4)
    }

    fn config(&self) -> &LLMConfig {
        &self.config
    }
}

// ============================================================================
// Ollama API 数据结构
// ============================================================================

#[derive(Debug, Serialize)]
struct OllamaRequestBody {
    model: String,
    messages: Vec<OllamaMessage>,
    format: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaMessageResponse,
}

#[derive(Debug, Deserialize)]
struct OllamaMessageResponse {
    content: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LLMProvider, SecretString};

    #[test]
    fn test_ollama_adapter_creation() {
        let config = LLMConfig {
            provider: LLMProvider::Ollama,
            model: "llama3".to_string(),
            api_key: SecretString::from(""), // Ollama 不需要 API key
            base_url: Some("http://localhost:11434".to_string()),
            temperature: 0.0,
            max_tokens: Some(4096),
            timeout: 60,
            deployment_name: None,
            api_version: "2024-02-15-preview".to_string(),
        };

        let adapter = OllamaAdapter::new(config);
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_model_info() {
        let config = LLMConfig {
            provider: LLMProvider::Ollama,
            model: "llama3".to_string(),
            api_key: SecretString::from(""),
            base_url: None,
            temperature: 0.0,
            max_tokens: None,
            timeout: 60,
            deployment_name: None,
            api_version: "2024-02-15-preview".to_string(),
        };

        let adapter = OllamaAdapter::new(config).unwrap();
        let info = adapter.model_info();

        assert_eq!(info.provider, "ollama");
        assert_eq!(info.model, "llama3");
        assert_eq!(info.max_context_tokens, 8192);
    }

    #[test]
    fn test_count_tokens() {
        let config = LLMConfig {
            provider: LLMProvider::Ollama,
            model: "llama3".to_string(),
            api_key: SecretString::from(""),
            base_url: None,
            temperature: 0.0,
            max_tokens: None,
            timeout: 60,
            deployment_name: None,
            api_version: "2024-02-15-preview".to_string(),
        };

        let adapter = OllamaAdapter::new(config).unwrap();
        let tokens = adapter.count_tokens("Hello, world!").unwrap();
        // "Hello, world!" 约 13 字符，13/4 ≈ 3
        assert!(tokens > 0);
    }
}
