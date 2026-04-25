//! LLM 适配器 Trait 定义

use async_trait::async_trait;
use schemars::schema::RootSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::LLMConfig;
use crate::dto::{ModelInfo, TokenUsage, TransformResult};
use crate::error::Result;

/// LLM 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    /// 消息列表
    pub messages: Vec<Message>,
    /// 响应格式
    pub response_format: ResponseFormat,
    /// 温度参数
    pub temperature: f32,
    /// 最大输出 token
    pub max_tokens: Option<usize>,
}

impl LLMRequest {
    /// 创建新的 LLM 请求
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            messages,
            response_format: ResponseFormat::JsonObject,
            temperature: 0.0,
            max_tokens: None,
        }
    }

    /// 设置温度
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// 设置最大 token
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// 设置响应格式
    pub fn with_response_format(mut self, format: ResponseFormat) -> Self {
        self.response_format = format;
        self
    }
}

/// 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum Message {
    /// 系统消息
    System { content: String },
    /// 用户消息
    User { content: String },
    /// 助手消息
    Assistant { content: String },
}

impl Message {
    /// 创建系统消息
    pub fn system(content: impl Into<String>) -> Self {
        Self::System {
            content: content.into(),
        }
    }

    /// 创建用户消息
    pub fn user(content: impl Into<String>) -> Self {
        Self::User {
            content: content.into(),
        }
    }

    /// 创建助手消息
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::Assistant {
            content: content.into(),
        }
    }
}

/// 响应格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ResponseFormat {
    /// 文本格式
    Text,
    /// JSON 对象格式（强制 JSON 输出）
    #[default]
    JsonObject,
}


/// LLM 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    /// 响应内容
    pub content: String,
    /// Token 使用量
    pub usage: TokenUsage,
    /// 使用的模型
    pub model: String,
}

impl LLMResponse {
    /// 创建新的响应
    pub fn new(content: String, usage: TokenUsage, model: String) -> Self {
        Self {
            content,
            usage,
            model,
        }
    }

    /// 解析为 JSON
    pub fn parse_json(&self) -> Result<Value> {
        serde_json::from_str(&self.content)
            .map_err(|e| crate::error::EtlError::LLMError(format!("Failed to parse JSON: {}", e)))
    }
}

/// LLM 适配器 Trait
#[async_trait]
pub trait LLMAdapter: Send + Sync {
    /// 结构化提取
    ///
    /// 将文本转换为符合 Schema 的结构化数据
    async fn extract_structured(
        &self,
        text: &str,
        schema: &RootSchema,
        prompt_template: Option<&str>,
    ) -> Result<TransformResult>;

    /// 获取模型信息
    fn model_info(&self) -> ModelInfo;

    /// 计算 token 数量
    fn count_tokens(&self, text: &str) -> Result<usize>;

    /// 获取配置
    fn config(&self) -> &LLMConfig;
}

/// 构建结构化提取提示词
pub fn build_extraction_prompt(
    text: &str,
    schema: &RootSchema,
    prompt_template: Option<&str>,
) -> String {
    if let Some(template) = prompt_template {
        // 使用自定义模板
        let schema_json = serde_json::to_string_pretty(schema).unwrap_or_default();
        template
            .replace("{schema}", &schema_json)
            .replace("{text}", text)
    } else {
        // 使用默认模板
        let schema_json = serde_json::to_string_pretty(schema).unwrap_or_default();
        format!(
            r#"Extract structured data from the following text according to the JSON Schema.

Schema:
{}

Text:
{}

Output the extracted data as a JSON object that conforms to the schema. Do not include any explanation or markdown formatting, only the JSON object."#,
            schema_json, text
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let system = Message::system("You are a helpful assistant.");
        let user = Message::user("Hello");
        let assistant = Message::assistant("Hi there!");

        match system {
            Message::System { content } => assert_eq!(content, "You are a helpful assistant."),
            _ => panic!("Expected System message"),
        }

        match user {
            Message::User { content } => assert_eq!(content, "Hello"),
            _ => panic!("Expected User message"),
        }

        match assistant {
            Message::Assistant { content } => assert_eq!(content, "Hi there!"),
            _ => panic!("Expected Assistant message"),
        }
    }

    #[test]
    fn test_llm_request() {
        let request = LLMRequest::new(vec![
            Message::system("System prompt"),
            Message::user("User input"),
        ])
        .with_temperature(0.5)
        .with_max_tokens(1000);

        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.temperature, 0.5);
        assert_eq!(request.max_tokens, Some(1000));
        assert_eq!(request.response_format, ResponseFormat::JsonObject);
    }

    #[test]
    fn test_llm_response() {
        let response = LLMResponse::new(
            r#"{"key": "value"}"#.to_string(),
            TokenUsage::new(10, 20),
            "gpt-4".to_string(),
        );

        assert_eq!(response.model, "gpt-4");
        assert_eq!(response.usage.total_tokens, 30);

        let json = response.parse_json().unwrap();
        assert_eq!(json["key"], "value");
    }

    #[test]
    fn test_build_extraction_prompt() {
        use schemars::JsonSchema;

        #[derive(JsonSchema)]
        struct TestSchema {
            name: String,
        }

        let schema = schemars::schema_for!(TestSchema);
        let prompt = build_extraction_prompt("test text", &schema, None);

        assert!(prompt.contains("test text"));
        assert!(prompt.contains("TestSchema"));
    }
}
