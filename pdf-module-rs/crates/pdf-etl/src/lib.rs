//! PDF ETL Pipeline
//!
//! 提供 PDF 结构化提取与数据库对接的完整 ETL 流水线
//!
//! # 架构
//!
//! - `error`: 统一错误类型
//! - `config`: 配置模型
//! - `dto`: 数据传输对象
//! - `security`: 安全验证
//! - `schema`: Schema 定义与验证
//! - `llm`: LLM 适配器
//! - `database`: 数据库适配器
//! - `pipeline`: ETL 流水线
//! - `tools`: MCP 工具

pub mod config;
pub mod dto;
pub mod error;
pub mod security;

pub mod database;
pub mod llm;
pub mod pipeline;
pub mod schema;
pub mod tools;

// 重导出常用类型
pub use config::{DatabaseConfig, ExtractionConfig, LLMConfig};
pub use dto::{ETLResult, ExtractionResult, SaveResult, TransformResult};
pub use error::{EtlError, Result};
pub use security::{
    quote_identifier_mysql, quote_identifier_postgres, quote_identifier_sqlite, validate_json_key,
    validate_schema_name, validate_table_name,
};
