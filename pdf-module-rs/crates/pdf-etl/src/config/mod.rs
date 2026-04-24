//! ETL 配置模块
//!
//! 定义提取、LLM、数据库等配置结构体

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::env;
use std::fmt;

use crate::error::{EtlError, Result};

/// 安全字符串包装器，防止敏感信息泄露
#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(String);

impl SecretString {
    /// 创建新的安全字符串
    pub fn new(s: String) -> Self {
        Self(s)
    }

    /// 从字符串创建
    pub fn from(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// 获取内部值（谨慎使用）
    pub fn expose_secret(&self) -> &str {
        &self.0
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***SECRET***")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***SECRET***")
    }
}

impl Serialize for SecretString {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 序列化时隐藏真实值
        serializer.serialize_str("***SECRET***")
    }
}

impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Ok(Self(s))
    }
}

impl From<String> for SecretString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SecretString {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// 提取引擎类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ExtractionEngine {
    /// lopdf 纯 Rust 引擎
    #[default]
    Lopdf,
    /// pdfium C++ 引擎
    Pdfium,
}


/// 文本分块策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ChunkStrategy {
    /// 按页分块
    #[default]
    Page,
    /// 按段落分块
    Paragraph,
    /// 按固定长度分块
    FixedLength,
}


/// 提取配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// 提取引擎
    pub engine: ExtractionEngine,
    /// 分块策略
    #[serde(default)]
    pub chunk_strategy: ChunkStrategy,
    /// 分块大小（字符数）
    #[serde(default = "default_chunk_size")]
    pub chunk_size: usize,
    /// 是否包含元数据
    #[serde(default = "default_true")]
    pub include_metadata: bool,
}

fn default_chunk_size() -> usize {
    1000
}

fn default_true() -> bool {
    true
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            engine: ExtractionEngine::default(),
            chunk_strategy: ChunkStrategy::default(),
            chunk_size: default_chunk_size(),
            include_metadata: true,
        }
    }
}

impl ExtractionConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self> {
        let engine = match env::var("EXTRACTION_ENGINE").ok().as_deref() {
            Some("lopdf") => ExtractionEngine::Lopdf,
            Some("pdfium") => ExtractionEngine::Pdfium,
            _ => ExtractionEngine::default(),
        };

        let chunk_strategy = match env::var("EXTRACTION_CHUNK_STRATEGY").ok().as_deref() {
            Some("page") => ChunkStrategy::Page,
            Some("paragraph") => ChunkStrategy::Paragraph,
            Some("fixed_length") => ChunkStrategy::FixedLength,
            _ => ChunkStrategy::default(),
        };

        let chunk_size = env::var("EXTRACTION_CHUNK_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(default_chunk_size());

        let include_metadata = env::var("EXTRACTION_INCLUDE_METADATA")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true);

        Ok(Self {
            engine,
            chunk_strategy,
            chunk_size,
            include_metadata,
        })
    }

    /// 验证配置有效性
    pub fn validate(&self) -> Result<()> {
        if self.chunk_size == 0 {
            return Err(EtlError::ConfigError(
                "chunk_size must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// LLM 提供商类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum LLMProvider {
    /// OpenAI
    #[default]
    OpenAI,
    /// Azure OpenAI
    Azure,
    /// Ollama 本地模型
    Ollama,
}


/// LLM 配置
#[derive(Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// LLM 提供商
    pub provider: LLMProvider,
    /// 模型名称
    pub model: String,
    /// API 密钥（安全存储）
    pub api_key: SecretString,
    /// API 基础 URL
    pub base_url: Option<String>,
    /// 温度参数
    #[serde(default)]
    pub temperature: f32,
    /// 最大输出 token
    pub max_tokens: Option<usize>,
    /// 超时时间（秒）
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Azure 部署名称（仅 Azure 使用）
    pub deployment_name: Option<String>,
    /// Azure API 版本（仅 Azure 使用）
    #[serde(default = "default_azure_api_version")]
    pub api_version: String,
}

impl fmt::Debug for LLMConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LLMConfig")
            .field("provider", &self.provider)
            .field("model", &self.model)
            .field("api_key", &"***REDACTED***")
            .field("base_url", &self.base_url)
            .field("temperature", &self.temperature)
            .field("max_tokens", &self.max_tokens)
            .field("timeout", &self.timeout)
            .field("deployment_name", &self.deployment_name)
            .field("api_version", &self.api_version)
            .finish()
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_azure_api_version() -> String {
    "2024-02-15-preview".to_string()
}

impl LLMConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self> {
        let provider = match env::var("LLM_PROVIDER").ok().as_deref() {
            Some("openai") => LLMProvider::OpenAI,
            Some("azure") => LLMProvider::Azure,
            Some("ollama") => LLMProvider::Ollama,
            _ => LLMProvider::default(),
        };

        let model = env::var("LLM_MODEL").unwrap_or_else(|_| match provider {
            LLMProvider::OpenAI => "gpt-4o".to_string(),
            LLMProvider::Azure => "gpt-4".to_string(),
            LLMProvider::Ollama => "llama3".to_string(),
        });

        let api_key = env::var("LLM_API_KEY")
            .map(SecretString::from)
            .unwrap_or_else(|_| SecretString::from(""));

        let base_url = env::var("LLM_BASE_URL").ok().or_else(|| match provider {
            LLMProvider::OpenAI => Some("https://api.openai.com/v1".to_string()),
            LLMProvider::Azure => None,
            LLMProvider::Ollama => Some("http://localhost:11434".to_string()),
        });

        let temperature = env::var("LLM_TEMPERATURE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        let max_tokens = env::var("LLM_MAX_TOKENS").ok().and_then(|s| s.parse().ok());

        let timeout = env::var("LLM_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(default_timeout());

        let deployment_name = env::var("AZURE_DEPLOYMENT_NAME").ok();
        let api_version =
            env::var("AZURE_API_VERSION").unwrap_or_else(|_| default_azure_api_version());

        Ok(Self {
            provider,
            model,
            api_key,
            base_url,
            temperature,
            max_tokens,
            timeout,
            deployment_name,
            api_version,
        })
    }

    /// 验证配置有效性
    pub fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() && self.provider != LLMProvider::Ollama {
            return Err(EtlError::ConfigError(
                "api_key is required for non-Ollama providers".to_string(),
            ));
        }
        if self.model.is_empty() {
            return Err(EtlError::ConfigError("model is required".to_string()));
        }
        if self.temperature < 0.0 || self.temperature > 2.0 {
            return Err(EtlError::ConfigError(
                "temperature must be between 0.0 and 2.0".to_string(),
            ));
        }
        Ok(())
    }

    /// 获取完整的 API 基础 URL
    pub fn get_base_url(&self) -> String {
        self.base_url
            .clone()
            .unwrap_or_else(|| match self.provider {
                LLMProvider::OpenAI => "https://api.openai.com/v1".to_string(),
                LLMProvider::Azure => "".to_string(), // Azure 使用不同的 URL 格式
                LLMProvider::Ollama => "http://localhost:11434".to_string(),
            })
    }
}

/// 数据库类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum DatabaseType {
    /// PostgreSQL
    #[default]
    Postgres,
    /// MySQL
    MySQL,
    /// SQLite
    SQLite,
}


/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库类型
    pub db_type: DatabaseType,
    /// 连接字符串
    pub connection_string: String,
    /// 目标表名
    pub table_name: String,
    /// 连接池大小
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
    /// 是否使用 JSONB（仅 PostgreSQL）
    #[serde(default = "default_true")]
    pub use_jsonb: bool,
}

fn default_pool_size() -> u32 {
    10
}

impl DatabaseConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self> {
        let db_type = match env::var("DB_TYPE").ok().as_deref() {
            Some("postgres") => DatabaseType::Postgres,
            Some("mysql") => DatabaseType::MySQL,
            Some("sqlite") => DatabaseType::SQLite,
            _ => DatabaseType::default(),
        };

        let connection_string = env::var("DB_CONNECTION_STRING")
            .map_err(|_| EtlError::ConfigError("DB_CONNECTION_STRING not set".to_string()))?;

        let table_name =
            env::var("DB_TABLE_NAME").unwrap_or_else(|_| "extracted_documents".to_string());

        let pool_size = env::var("DB_POOL_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(default_pool_size());

        let use_jsonb = env::var("DB_USE_JSONB")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true);

        Ok(Self {
            db_type,
            connection_string,
            table_name,
            pool_size,
            use_jsonb,
        })
    }

    /// 验证配置有效性
    pub fn validate(&self) -> Result<()> {
        if self.connection_string.is_empty() {
            return Err(EtlError::ConfigError(
                "connection_string is required".to_string(),
            ));
        }
        if self.table_name.is_empty() {
            return Err(EtlError::ConfigError("table_name is required".to_string()));
        }
        if self.pool_size == 0 {
            return Err(EtlError::ConfigError(
                "pool_size must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_config_default() {
        let config = ExtractionConfig::default();
        assert_eq!(config.engine, ExtractionEngine::Lopdf);
        assert_eq!(config.chunk_strategy, ChunkStrategy::Page);
        assert_eq!(config.chunk_size, 1000);
        assert!(config.include_metadata);
    }

    #[test]
    fn test_extraction_config_validate() {
        let config = ExtractionConfig {
            engine: ExtractionEngine::Lopdf,
            chunk_strategy: ChunkStrategy::Page,
            chunk_size: 0,
            include_metadata: true,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_llm_config_validate() {
        let config = LLMConfig {
            provider: LLMProvider::OpenAI,
            model: "gpt-4".to_string(),
            api_key: SecretString::from(""),
            base_url: None,
            temperature: 0.0,
            max_tokens: None,
            timeout: 30,
            deployment_name: None,
            api_version: default_azure_api_version(),
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig {
            db_type: DatabaseType::Postgres,
            connection_string: "postgresql://localhost/db".to_string(),
            table_name: "test".to_string(),
            pool_size: 10,
            use_jsonb: true,
        };
        assert!(config.validate().is_ok());
    }
}
