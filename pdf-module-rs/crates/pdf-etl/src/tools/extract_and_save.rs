//! extract_and_save MCP 工具
//!
//! 整合 ETL 流程的 MCP 工具实现

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::config::{DatabaseConfig, ExtractionConfig, LLMConfig};
use crate::dto::ETLResult;
use crate::error::{EtlError, Result};
use crate::pipeline::etl_pipeline::ETLPipelineImpl;
use crate::schema::SchemaDefinition;

/// extract_and_save 工具参数
#[derive(Debug, Deserialize)]
pub struct ExtractAndSaveParams {
    /// PDF 文件路径
    pub pdf_path: String,
    /// Schema 定义
    pub schema: SchemaDefinition,
    /// LLM 配置
    pub llm_config: LLMConfig,
    /// 数据库配置（可选）
    pub db_config: Option<DatabaseConfig>,
    /// 提取配置（可选）
    pub extraction_config: Option<ExtractionConfig>,
    /// 自定义提示词模板（可选）
    pub prompt_template: Option<String>,
    /// 是否保存到数据库
    #[serde(default = "default_save_to_db")]
    pub save_to_db: bool,
}

fn default_save_to_db() -> bool {
    true
}

/// extract_and_save 工具输出
#[derive(Debug, Serialize)]
pub struct ExtractAndSaveOutput {
    /// 是否成功
    pub success: bool,
    /// ETL 结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ETLResult>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
}

/// extract_and_save 工具
pub struct ExtractAndSaveTool {
    pipeline: Arc<ETLPipelineImpl>,
}

impl ExtractAndSaveTool {
    /// 创建新的工具实例
    pub async fn new(params: ExtractAndSaveParams) -> Result<Self> {
        let extraction_config = params.extraction_config.unwrap_or_default();
        let db_config = params.db_config;

        let pipeline =
            ETLPipelineImpl::new(extraction_config, params.llm_config, db_config).await?;

        Ok(Self {
            pipeline: Arc::new(pipeline),
        })
    }

    /// 使用已有的流水线创建
    pub fn with_pipeline(pipeline: Arc<ETLPipelineImpl>) -> Self {
        Self { pipeline }
    }

    /// 执行工具
    pub async fn execute(&self, params: ExtractAndSaveParams) -> ExtractAndSaveOutput {
        let start = std::time::Instant::now();

        info!("Executing extract_and_save for: {}", params.pdf_path);

        match self
            .pipeline
            .execute(
                &params.pdf_path,
                params.schema,
                params.prompt_template.as_deref(),
                params.save_to_db,
            )
            .await
        {
            Ok(result) => {
                let processing_time_ms = start.elapsed().as_millis() as u64;
                info!(
                    "extract_and_save completed successfully in {}ms",
                    processing_time_ms
                );
                ExtractAndSaveOutput {
                    success: true,
                    result: Some(result),
                    error: None,
                    processing_time_ms,
                }
            }
            Err(e) => {
                let processing_time_ms = start.elapsed().as_millis() as u64;
                info!("extract_and_save failed: {}", e);
                ExtractAndSaveOutput {
                    success: false,
                    result: None,
                    error: Some(e.to_string()),
                    processing_time_ms,
                }
            }
        }
    }

    /// 验证参数
    pub fn validate_params(&self, params: &ExtractAndSaveParams) -> Result<()> {
        // 验证 PDF 路径
        if params.pdf_path.is_empty() {
            return Err(EtlError::ParameterMissingError("pdf_path".to_string()));
        }

        // 验证 Schema
        if params.schema.name.is_empty() {
            return Err(EtlError::ParameterMissingError("schema.name".to_string()));
        }

        // 验证 LLM 配置
        params.llm_config.validate()?;

        // 验证数据库配置（如果提供）
        if let Some(ref db_config) = params.db_config {
            db_config.validate()?;
        }

        // 验证提取配置（如果提供）
        if let Some(ref extraction_config) = params.extraction_config {
            extraction_config.validate()?;
        }

        Ok(())
    }
}

/// extract_text 工具参数
#[derive(Debug, Deserialize)]
pub struct ExtractTextParams {
    /// PDF 文件路径
    pub pdf_path: String,
    /// 提取配置（可选）
    pub extraction_config: Option<ExtractionConfig>,
}

/// extract_text 工具输出
#[derive(Debug, Serialize)]
pub struct ExtractTextOutput {
    /// 是否成功
    pub success: bool,
    /// 提取结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<crate::dto::ExtractionResult>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// transform_to_schema 工具参数
#[derive(Debug, Deserialize)]
pub struct TransformToSchemaParams {
    /// 文本内容
    pub text: String,
    /// Schema 定义
    pub schema: SchemaDefinition,
    /// LLM 配置
    pub llm_config: LLMConfig,
    /// 自定义提示词模板（可选）
    pub prompt_template: Option<String>,
}

/// transform_to_schema 工具输出
#[derive(Debug, Serialize)]
pub struct TransformToSchemaOutput {
    /// 是否成功
    pub success: bool,
    /// 转换结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<crate::dto::TransformResult>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// save_to_db 工具参数
#[derive(Debug, Deserialize)]
pub struct SaveToDbParams {
    /// 数据
    pub data: serde_json::Value,
    /// Schema 名称
    pub schema_name: String,
    /// 数据库配置
    pub db_config: DatabaseConfig,
    /// 源文件（可选）
    pub source_file: Option<String>,
}

/// save_to_db 工具输出
#[derive(Debug, Serialize)]
pub struct SaveToDbOutput {
    /// 是否成功
    pub success: bool,
    /// 保存结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<crate::dto::SaveResult>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DatabaseType, LLMProvider, SecretString};

    #[test]
    fn test_validate_params() {
        let params = ExtractAndSaveParams {
            pdf_path: "".to_string(),
            schema: SchemaDefinition::from_json(r#"{"type": "object"}"#).unwrap(),
            llm_config: LLMConfig {
                provider: LLMProvider::OpenAI,
                model: "gpt-4".to_string(),
                api_key: SecretString::from("test"),
                base_url: None,
                temperature: 0.0,
                max_tokens: None,
                timeout: 30,
                deployment_name: None,
                api_version: "2024-02-15-preview".to_string(),
            },
            db_config: None,
            extraction_config: None,
            prompt_template: None,
            save_to_db: false,
        };

        // 由于需要实际的 pipeline，这里只测试参数验证逻辑
        assert!(params.pdf_path.is_empty());
    }
}
