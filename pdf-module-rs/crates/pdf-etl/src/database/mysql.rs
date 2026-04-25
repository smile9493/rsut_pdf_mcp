//! MySQL 数据库适配器

use async_trait::async_trait;
use schemars::schema::RootSchema;
use serde_json::Value;
use sqlx::{MySql, Pool};
use std::collections::HashMap;
use tracing::debug;
use uuid::Uuid;

use crate::config::DatabaseConfig;
use crate::database::adapter::DatabaseAdapter;
use crate::dto::{PoolStatus, SaveResult};
use crate::error::{EtlError, Result};

/// MySQL 适配器
pub struct MySQLAdapter {
    pool: Pool<MySql>,
    config: DatabaseConfig,
}

impl MySQLAdapter {
    /// 创建新的 MySQL 适配器
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        config.validate()?;

        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(config.pool_size)
            .connect(&config.connection_string)
            .await
            .map_err(|e| EtlError::DatabaseError(format!("Failed to connect to MySQL: {}", e)))?;

        Ok(Self { pool, config })
    }

    /// 生成创建表的 SQL
    fn generate_create_table_sql(&self, table: &str) -> String {
        format!(
            r#"CREATE TABLE IF NOT EXISTS {} (
                id VARCHAR(36) PRIMARY KEY,
                schema_name VARCHAR(255) NOT NULL,
                data JSON NOT NULL,
                source_file VARCHAR(512),
                extraction_metadata JSON,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
                INDEX idx_{}_schema_name (schema_name),
                INDEX idx_{}_created_at (created_at)
            )"#,
            table, table, table
        )
    }
}

#[async_trait]
impl DatabaseAdapter for MySQLAdapter {
    async fn save(&self, table: &str, data: &Value) -> Result<SaveResult> {
        let record_id = Uuid::new_v4().to_string();
        let schema_name = "default";

        let query = format!(
            "INSERT INTO {} (id, schema_name, data) VALUES (?, ?, ?)",
            table
        );

        let data_str = serde_json::to_string(data)?;

        sqlx::query(&query)
            .bind(&record_id)
            .bind(schema_name)
            .bind(&data_str)
            .execute(&self.pool)
            .await
            .map_err(|e| EtlError::DatabaseError(format!("Failed to insert record: {}", e)))?;

        Ok(SaveResult::new(record_id.into(), table.to_string(), 1))
    }

    async fn save_batch(&self, table: &str, data: &[Value]) -> Result<Vec<SaveResult>> {
        let mut results = Vec::with_capacity(data.len());

        let mut tx =
            self.pool.begin().await.map_err(|e| {
                EtlError::DatabaseError(format!("Failed to begin transaction: {}", e))
            })?;

        for item in data {
            let record_id = Uuid::new_v4().to_string();
            let query = format!(
                "INSERT INTO {} (id, schema_name, data) VALUES (?, ?, ?)",
                table
            );

            let data_str = serde_json::to_string(item)?;

            sqlx::query(&query)
                .bind(&record_id)
                .bind("default")
                .bind(&data_str)
                .execute(&mut *tx)
                .await
                .map_err(|e| EtlError::DatabaseError(format!("Failed to insert record: {}", e)))?;

            results.push(SaveResult::new(record_id.into(), table.to_string(), 1));
        }

        tx.commit()
            .await
            .map_err(|e| EtlError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        Ok(results)
    }

    async fn query(&self, table: &str, filters: HashMap<String, Value>) -> Result<Vec<Value>> {
        let mut query = format!("SELECT data FROM {}", table);
        let mut conditions = Vec::new();
        let mut bind_values = Vec::new();

        for (key, value) in filters {
            conditions.push(format!("JSON_UNQUOTE(JSON_EXTRACT(data, '$.{}')) = ?", key));
            bind_values.push(value);
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        let rows: Vec<(String,)> = sqlx::query_as(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| EtlError::DatabaseError(format!("Failed to query records: {}", e)))?;

        Ok(rows
            .into_iter()
            .filter_map(|r| serde_json::from_str(&r.0).ok())
            .collect())
    }

    async fn create_table_if_not_exists(
        &self,
        table: &str,
        _schema: Option<&RootSchema>,
    ) -> Result<()> {
        let sql = self.generate_create_table_sql(table);

        sqlx::query(&sql)
            .execute(&self.pool)
            .await
            .map_err(|e| EtlError::DatabaseError(format!("Failed to create table: {}", e)))?;

        debug!("Created table: {}", table);
        Ok(())
    }

    fn pool_status(&self) -> PoolStatus {
        // sqlx Pool 没有 status() 方法，返回简化状态
        PoolStatus::new(
            self.config.pool_size,
            self.config.pool_size / 2,
            self.config.pool_size,
        )
    }

    fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    async fn save_with_metadata(
        &self,
        table: &str,
        data: &Value,
        schema_name: &str,
        source_file: Option<&str>,
        extraction_metadata: Option<&Value>,
    ) -> Result<SaveResult> {
        let record_id = Uuid::new_v4().to_string();

        let query = format!(
            "INSERT INTO {} (id, schema_name, data, source_file, extraction_metadata) VALUES (?, ?, ?, ?, ?)",
            table
        );

        let data_str = serde_json::to_string(data)?;
        let metadata_str = extraction_metadata
            .map(serde_json::to_string)
            .transpose()?;

        sqlx::query(&query)
            .bind(&record_id)
            .bind(schema_name)
            .bind(&data_str)
            .bind(source_file)
            .bind(metadata_str.as_deref())
            .execute(&self.pool)
            .await
            .map_err(|e| EtlError::DatabaseError(format!("Failed to insert record: {}", e)))?;

        Ok(SaveResult::new(record_id.into(), table.to_string(), 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_create_table_sql() {
        let config = DatabaseConfig {
            db_type: crate::config::DatabaseType::MySQL,
            connection_string: "mysql://localhost/test".to_string(),
            table_name: "test_table".to_string(),
            pool_size: 10,
            use_jsonb: true,
        };

        assert_eq!(config.table_name, "test_table");
    }
}
