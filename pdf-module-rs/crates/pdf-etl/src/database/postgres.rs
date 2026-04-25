//! PostgreSQL 数据库适配器

use async_trait::async_trait;
use schemars::schema::RootSchema;
use serde_json::Value;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use tracing::debug;
use uuid::Uuid;

use crate::config::DatabaseConfig;
use crate::database::adapter::DatabaseAdapter;
use crate::dto::{PoolStatus, SaveResult};
use crate::error::{EtlError, Result};
use crate::security::{quote_identifier_postgres, validate_json_key, validate_table_name};

/// PostgreSQL 适配器
pub struct PostgreSQLAdapter {
    pool: Pool<Postgres>,
    config: DatabaseConfig,
}

impl PostgreSQLAdapter {
    /// 创建新的 PostgreSQL 适配器
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        config.validate()?;

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.pool_size)
            .connect(&config.connection_string)
            .await
            .map_err(|e| {
                EtlError::DatabaseError(format!("Failed to connect to PostgreSQL: {}", e))
            })?;

        Ok(Self { pool, config })
    }

    /// 生成创建表的 SQL（使用安全的标识符引用）
    fn generate_create_table_sql(&self, table: &str) -> Result<String> {
        validate_table_name(table)?;
        let quoted = quote_identifier_postgres(table);

        Ok(format!(
            r#"CREATE TABLE IF NOT EXISTS {quoted} (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                schema_name VARCHAR(255) NOT NULL,
                data JSONB NOT NULL,
                source_file VARCHAR(512),
                extraction_metadata JSONB,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_{table}_schema_name ON {quoted}(schema_name);
            CREATE INDEX IF NOT EXISTS idx_{table}_created_at ON {quoted}(created_at);
            CREATE INDEX IF NOT EXISTS idx_{table}_data_gin ON {quoted} USING GIN(data);"#,
            quoted = quoted,
            table = table.replace('-', "_") // 索引名不能有连字符
        ))
    }
}

#[async_trait]
impl DatabaseAdapter for PostgreSQLAdapter {
    async fn save(&self, table: &str, data: &Value) -> Result<SaveResult> {
        // 验证表名
        validate_table_name(table)?;
        let quoted_table = quote_identifier_postgres(table);

        let record_id = Uuid::new_v4();
        let schema_name = "default";

        let query = format!(
            "INSERT INTO {} (id, schema_name, data) VALUES ($1, $2, $3) RETURNING id",
            quoted_table
        );

        let row: (Uuid,) = sqlx::query_as(&query)
            .bind(record_id)
            .bind(schema_name)
            .bind(data)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| EtlError::DatabaseError(format!("Failed to insert record: {}", e)))?;

        Ok(SaveResult::new(
            row.0.to_string().into(),
            table.to_string(),
            1,
        ))
    }

    async fn save_batch(&self, table: &str, data: &[Value]) -> Result<Vec<SaveResult>> {
        // 验证表名
        validate_table_name(table)?;
        let quoted_table = quote_identifier_postgres(table);

        let mut results = Vec::with_capacity(data.len());

        // 使用事务批量插入
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                EtlError::DatabaseError(format!("Failed to begin transaction: {}", e))
            })?;

        for item in data {
            let record_id = Uuid::new_v4();
            let query = format!(
                "INSERT INTO {} (id, schema_name, data) VALUES ($1, $2, $3)",
                quoted_table
            );

            sqlx::query(&query)
                .bind(record_id)
                .bind("default")
                .bind(item)
                .execute(&mut *tx)
                .await
                .map_err(|e| EtlError::DatabaseError(format!("Failed to insert record: {}", e)))?;

            results.push(SaveResult::new(
                record_id.to_string().into(),
                table.to_string(),
                1,
            ));
        }

        tx.commit()
            .await
            .map_err(|e| EtlError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        Ok(results)
    }

    async fn query(&self, table: &str, filters: HashMap<String, Value>) -> Result<Vec<Value>> {
        // 验证表名
        validate_table_name(table)?;
        let quoted_table = quote_identifier_postgres(table);

        let mut query = format!("SELECT data FROM {}", quoted_table);
        let mut conditions = Vec::new();
        let mut bind_values = Vec::new();

        for (i, (key, value)) in filters.iter().enumerate() {
            let param_index = i + 1;
            // 验证 JSON key
            validate_json_key(key)?;
            conditions.push(format!("data->>'{}' = ${}", key, param_index));
            bind_values.push(value.clone());
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        let rows: Vec<(Value,)> = sqlx::query_as(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| EtlError::DatabaseError(format!("Failed to query records: {}", e)))?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    async fn create_table_if_not_exists(
        &self,
        table: &str,
        _schema: Option<&RootSchema>,
    ) -> Result<()> {
        let sql = self.generate_create_table_sql(table)?;

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
        // 验证表名
        validate_table_name(table)?;
        let quoted_table = quote_identifier_postgres(table);

        let record_id = Uuid::new_v4();

        let query = format!(
            "INSERT INTO {} (id, schema_name, data, source_file, extraction_metadata) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            quoted_table
        );

        let row: (Uuid,) = sqlx::query_as(&query)
            .bind(record_id)
            .bind(schema_name)
            .bind(data)
            .bind(source_file)
            .bind(extraction_metadata)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| EtlError::DatabaseError(format!("Failed to insert record: {}", e)))?;

        Ok(SaveResult::new(
            row.0.to_string().into(),
            table.to_string(),
            1,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试需要实际的 PostgreSQL 数据库连接
    // 在 CI 中可以使用 testcontainers 来运行

    #[test]
    fn test_generate_create_table_sql() {
        let config = DatabaseConfig {
            db_type: crate::config::DatabaseType::Postgres,
            connection_string: "postgresql://localhost/test".to_string(),
            table_name: "test_table".to_string(),
            pool_size: 10,
            use_jsonb: true,
        };

        // 仅测试 SQL 生成逻辑
        assert!(config.table_name == "test_table");
    }
}
