//! 数据库适配器工厂

use std::sync::Arc;

use crate::config::{DatabaseConfig, DatabaseType};
use crate::database::adapter::DatabaseAdapter;
use crate::database::mysql::MySQLAdapter;
use crate::database::postgres::PostgreSQLAdapter;
use crate::database::sqlite::SQLiteAdapter;
use crate::error::Result;

/// 数据库适配器工厂
pub struct DatabaseAdapterFactory;

impl DatabaseAdapterFactory {
    /// 根据配置创建数据库适配器
    pub async fn create(config: DatabaseConfig) -> Result<Arc<dyn DatabaseAdapter>> {
        match config.db_type {
            DatabaseType::Postgres => {
                let adapter = PostgreSQLAdapter::new(config).await?;
                Ok(Arc::new(adapter))
            }
            DatabaseType::MySQL => {
                let adapter = MySQLAdapter::new(config).await?;
                Ok(Arc::new(adapter))
            }
            DatabaseType::SQLite => {
                let adapter = SQLiteAdapter::new(config).await?;
                Ok(Arc::new(adapter))
            }
        }
    }

    /// 从环境变量创建数据库适配器
    pub async fn from_env() -> Result<Arc<dyn DatabaseAdapter>> {
        let config = DatabaseConfig::from_env()?;
        Self::create(config).await
    }

    /// 创建 PostgreSQL 适配器
    pub async fn postgres(
        connection_string: String,
        table_name: Option<String>,
    ) -> Result<Arc<dyn DatabaseAdapter>> {
        let config = DatabaseConfig {
            db_type: DatabaseType::Postgres,
            connection_string,
            table_name: table_name.unwrap_or_else(|| "extracted_documents".to_string()),
            pool_size: 10,
            use_jsonb: true,
        };
        Self::create(config).await
    }

    /// 创建 MySQL 适配器
    pub async fn mysql(
        connection_string: String,
        table_name: Option<String>,
    ) -> Result<Arc<dyn DatabaseAdapter>> {
        let config = DatabaseConfig {
            db_type: DatabaseType::MySQL,
            connection_string,
            table_name: table_name.unwrap_or_else(|| "extracted_documents".to_string()),
            pool_size: 10,
            use_jsonb: false,
        };
        Self::create(config).await
    }

    /// 创建 SQLite 适配器
    pub async fn sqlite(
        connection_string: String,
        table_name: Option<String>,
    ) -> Result<Arc<dyn DatabaseAdapter>> {
        let config = DatabaseConfig {
            db_type: DatabaseType::SQLite,
            connection_string,
            table_name: table_name.unwrap_or_else(|| "extracted_documents".to_string()),
            pool_size: 10,
            use_jsonb: false,
        };
        Self::create(config).await
    }

    /// 创建内存 SQLite 适配器（用于测试）
    pub async fn sqlite_in_memory() -> Result<Arc<dyn DatabaseAdapter>> {
        Self::sqlite("sqlite::memory:".to_string(), None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_sqlite_in_memory() {
        let adapter = DatabaseAdapterFactory::sqlite_in_memory().await;
        assert!(adapter.is_ok());

        let status = adapter.unwrap().pool_status();
        assert!(status.max_connections > 0);
    }

    #[tokio::test]
    async fn test_sqlite_helper() {
        let adapter = DatabaseAdapterFactory::sqlite(
            "sqlite::memory:".to_string(),
            Some("test_table".to_string()),
        )
        .await;

        assert!(adapter.is_ok());

        let adapter = adapter.unwrap();
        let config = adapter.config();
        assert_eq!(config.table_name, "test_table");
    }
}
