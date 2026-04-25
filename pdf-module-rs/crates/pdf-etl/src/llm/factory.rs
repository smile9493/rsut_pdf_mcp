//! LLM 适配器工厂

use std::sync::Arc;

use crate::config::{LLMConfig, LLMProvider, SecretString};
use crate::error::Result;
use crate::llm::adapter::LLMAdapter;
use crate::llm::azure::AzureOpenAIAdapter;
use crate::llm::ollama::OllamaAdapter;
use crate::llm::openai::OpenAIAdapter;

/// LLM 适配器工厂
pub struct LLMAdapterFactory;

impl LLMAdapterFactory {
    /// 根据配置创建 LLM 适配器
    pub fn create(config: LLMConfig) -> Result<Arc<dyn LLMAdapter>> {
        match config.provider {
            LLMProvider::OpenAI => {
                let adapter = OpenAIAdapter::new(config)?;
                Ok(Arc::new(adapter))
            }
            LLMProvider::Azure => {
                let adapter = AzureOpenAIAdapter::new(config)?;
                Ok(Arc::new(adapter))
            }
            LLMProvider::Ollama => {
                let adapter = OllamaAdapter::new(config)?;
                Ok(Arc::new(adapter))
            }
        }
    }

    /// 从环境变量创建 LLM 适配器
    pub fn from_env() -> Result<Arc<dyn LLMAdapter>> {
        let config = LLMConfig::from_env()?;
        Self::create(config)
    }

    /// 创建 OpenAI 适配器
    pub fn openai(api_key: String, model: Option<String>) -> Result<Arc<dyn LLMAdapter>> {
        let config = LLMConfig {
            provider: LLMProvider::OpenAI,
            model: model.unwrap_or_else(|| "gpt-4o".to_string()),
            api_key: SecretString::from(api_key),
            base_url: None,
            temperature: 0.0,
            max_tokens: None,
            timeout: 30,
            deployment_name: None,
            api_version: "2024-02-15-preview".to_string(),
        };
        Self::create(config)
    }

    /// 创建 Azure OpenAI 适配器
    pub fn azure(
        api_key: String,
        base_url: String,
        deployment_name: String,
        model: Option<String>,
    ) -> Result<Arc<dyn LLMAdapter>> {
        let config = LLMConfig {
            provider: LLMProvider::Azure,
            model: model.unwrap_or_else(|| "gpt-4".to_string()),
            api_key: SecretString::from(api_key),
            base_url: Some(base_url),
            temperature: 0.0,
            max_tokens: None,
            timeout: 30,
            deployment_name: Some(deployment_name),
            api_version: "2024-02-15-preview".to_string(),
        };
        Self::create(config)
    }

    /// 创建 Ollama 适配器
    pub fn ollama(model: Option<String>, base_url: Option<String>) -> Result<Arc<dyn LLMAdapter>> {
        let config = LLMConfig {
            provider: LLMProvider::Ollama,
            model: model.unwrap_or_else(|| "llama3".to_string()),
            api_key: SecretString::from(""),
            base_url: base_url.or(Some("http://localhost:11434".to_string())),
            temperature: 0.0,
            max_tokens: None,
            timeout: 60,
            deployment_name: None,
            api_version: "2024-02-15-preview".to_string(),
        };
        Self::create(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_openai() {
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

        let adapter = LLMAdapterFactory::create(config);
        assert!(adapter.is_ok());

        let info = adapter.unwrap().model_info();
        assert_eq!(info.provider, "openai");
    }

    #[test]
    fn test_create_ollama() {
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

        let adapter = LLMAdapterFactory::create(config);
        assert!(adapter.is_ok());

        let info = adapter.unwrap().model_info();
        assert_eq!(info.provider, "ollama");
    }

    #[test]
    fn test_openai_helper() {
        let adapter = LLMAdapterFactory::openai("test-key".to_string(), Some("gpt-4o".to_string()));
        assert!(adapter.is_ok());

        let info = adapter.unwrap().model_info();
        assert_eq!(info.model, "gpt-4o");
    }

    #[test]
    fn test_ollama_helper() {
        let adapter = LLMAdapterFactory::ollama(Some("mistral".to_string()), None);
        assert!(adapter.is_ok());

        let info = adapter.unwrap().model_info();
        assert_eq!(info.model, "mistral");
    }
}
