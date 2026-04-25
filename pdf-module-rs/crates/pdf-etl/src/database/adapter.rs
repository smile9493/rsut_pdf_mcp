//! 数据库适配器 Trait 定义

use async_trait::async_trait;
use schemars::schema::RootSchema;
use serde_json::Value;
use std::collections::HashMap;

use crate::config::DatabaseConfig;
use crate::dto::{PoolStatus, SaveResult};
use crate::error::Result;

/// 数据库适配器 Trait
#[async_trait]
pub trait DatabaseAdapter: Send + Sync {
    /// 保存单条记录
    async fn save(&self, table: &str, data: &Value) -> Result<SaveResult>;

    /// 批量保存记录
    async fn save_batch(&self, table: &str, data: &[Value]) -> Result<Vec<SaveResult>>;

    /// 查询记录
    async fn query(&self, table: &str, filters: HashMap<String, Value>) -> Result<Vec<Value>>;

    /// 创建表（如果不存在）
    async fn create_table_if_not_exists(
        &self,
        table: &str,
        schema: Option<&RootSchema>,
    ) -> Result<()>;

    /// 获取连接池状态
    fn pool_status(&self) -> PoolStatus;

    /// 获取配置
    fn config(&self) -> &DatabaseConfig;

    /// 保存带元数据的记录
    async fn save_with_metadata(
        &self,
        table: &str,
        data: &Value,
        schema_name: &str,
        source_file: Option<&str>,
        extraction_metadata: Option<&Value>,
    ) -> Result<SaveResult>;
}
