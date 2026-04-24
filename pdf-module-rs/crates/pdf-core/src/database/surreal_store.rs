//! SurrealDB store implementation
//! Provides embedded database using SurrealDB
//! Ideal for storing dynamic JSON structures from different PDF templates
//!
//! Uses SurrealQL for all operations to avoid complex type conversions

use crate::error::{PdfModuleError, PdfResult};
use serde_json::Value;
use std::path::PathBuf;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

/// Escape a record ID for SurrealQL
/// Record IDs containing special characters (like UUID dashes) need to be wrapped in ⟨⟩
fn escape_record_id(record_id: &str) -> String {
    // If the record_id contains a colon (table:id format), escape the id part
    if let Some((table, id)) = record_id.split_once(':') {
        format!("{}:⟨{}⟩", table, id)
    } else {
        record_id.to_string()
    }
}

/// Unwrap SurrealDB's internal type wrappers
/// SurrealDB serializes values with type tags like {"Array": [...]}, {"Object": {...}},
/// {"Strand": "..."}, {"Number": {"Int": 42}}, {"Thing": {...}}
/// This function recursively unwraps them to plain JSON values
fn unwrap_surreal_types(value: Value) -> Value {
    match value {
        Value::Object(mut map) => {
            // Check for SurrealDB type wrappers
            if let Some(arr) = map.remove("Array") {
                return unwrap_surreal_types(arr);
            }
            if let Some(obj) = map.remove("Object") {
                return unwrap_surreal_types(obj);
            }
            if let Some(s) = map.remove("Strand") {
                return s;
            }
            if let Some(num) = map.remove("Number") {
                return unwrap_surreal_types(num);
            }
            if let Some(thing) = map.remove("Thing") {
                // Convert Thing (record ID) to string
                return thing;
            }
            if let Some(i) = map.remove("Int") {
                return i;
            }
            if let Some(f) = map.remove("Float") {
                return f;
            }
            // Recursively process all values in the object
            let new_map: serde_json::Map<String, Value> = map
                .into_iter()
                .map(|(k, v)| (k, unwrap_surreal_types(v)))
                .collect();
            Value::Object(new_map)
        }
        Value::Array(arr) => {
            Value::Array(arr.into_iter().map(unwrap_surreal_types).collect())
        }
        other => other,
    }
}

/// SurrealDB store configuration
#[derive(Debug, Clone)]
pub struct SurrealStoreConfig {
    /// Database path (for RocksDB, unused for Mem)
    pub path: PathBuf,
    /// Namespace
    pub namespace: String,
    /// Database name
    pub database: String,
}

impl Default for SurrealStoreConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("data/pdf_mcp.db"),
            namespace: "mcp".to_string(),
            database: "etl".to_string(),
        }
    }
}

/// SurrealDB store
/// Provides schema-less storage for ETL results and audit logs
pub struct SurrealStore {
    /// SurrealDB instance
    db: Surreal<Any>,
    /// Configuration
    config: SurrealStoreConfig,
}

impl SurrealStore {
    /// Create and initialize a new SurrealDB store
    /// Uses in-memory storage by default
    pub async fn new(config: SurrealStoreConfig) -> PdfResult<Self> {
        // Create embedded database using Any engine with in-memory storage
        let db: Surreal<Any> = Surreal::init();
        db.connect("memory").await.map_err(|e| {
            PdfModuleError::StorageError(format!("SurrealDB connect failed: {}", e))
        })?;

        // Use namespace and database
        db.use_ns(&config.namespace)
            .use_db(&config.database)
            .await
            .map_err(|e| {
                PdfModuleError::StorageError(format!("SurrealDB use_ns/use_db failed: {}", e))
            })?;

        Ok(Self { db, config })
    }

    /// Create with default configuration
    pub async fn with_defaults() -> PdfResult<Self> {
        Self::new(SurrealStoreConfig::default()).await
    }

    /// Save an ETL result using SurrealQL
    pub async fn save_etl_result(
        &self,
        table: &str,
        id: &str,
        data: Value,
    ) -> PdfResult<String> {
        let record_id = format!("{}:⟨{}⟩", table, id);
        let data_str = serde_json::to_string(&data).map_err(|e| {
            PdfModuleError::StorageError(format!("Serialize data failed: {}", e))
        })?;

        let sql = format!("CREATE {} CONTENT {}", record_id, data_str);

        self.execute_query(&sql).await?;
        Ok(format!("{}:{}", table, id))
    }

    /// Query records from a table
    pub async fn query(
        &self,
        table: &str,
        condition: &str,
    ) -> PdfResult<Vec<Value>> {
        let sql = if condition.is_empty() {
            format!("SELECT * FROM {}", table)
        } else {
            format!("SELECT * FROM {} WHERE {}", table, condition)
        };

        self.execute_query(&sql).await
    }

    /// Get a record by ID using SurrealQL
    pub async fn get(&self, record_id: &str) -> PdfResult<Option<Value>> {
        let sql = format!("SELECT * FROM {}", escape_record_id(record_id));
        let results = self.execute_query(&sql).await?;
        Ok(results.into_iter().next())
    }

    /// Update a record using SurrealQL
    pub async fn update(&self, record_id: &str, data: Value) -> PdfResult<Option<Value>> {
        let data_str = serde_json::to_string(&data).map_err(|e| {
            PdfModuleError::StorageError(format!("Serialize data failed: {}", e))
        })?;
        let sql = format!("UPDATE {} CONTENT {}", escape_record_id(record_id), data_str);
        let results = self.execute_query(&sql).await?;
        Ok(results.into_iter().next())
    }

    /// Delete a record using SurrealQL
    pub async fn delete(&self, record_id: &str) -> PdfResult<Option<Value>> {
        let sql = format!("DELETE FROM {} RETURN BEFORE", escape_record_id(record_id));
        let results = self.execute_query(&sql).await?;
        Ok(results.into_iter().next())
    }

    /// Execute a raw SurrealQL query
    pub async fn execute_query(&self, sql: &str) -> PdfResult<Vec<Value>> {
        let mut response = self
            .db
            .query(sql)
            .await
            .map_err(|e| PdfModuleError::StorageError(format!("SurrealDB query failed: {}", e)))?;

        // Take the first result - SurrealDB returns its own Value type
        let result: surrealdb::Value = response
            .take(0)
            .map_err(|e| {
                PdfModuleError::StorageError(format!("SurrealDB result take failed: {}", e))
            })?;

        // Convert surrealdb Value to JSON string and parse
        // surrealdb::Value implements Display which outputs SurrealQL format
        // We need to use the json() method or convert through serde
        let json_str = serde_json::to_string(&result)
            .map_err(|e| {
                PdfModuleError::StorageError(format!("SurrealDB value conversion failed: {}", e))
            })?;

        let json_result: Value = serde_json::from_str(&json_str)
            .unwrap_or(Value::Null);

        // Unwrap SurrealDB's type wrappers: {"Array": [...]} -> [...], {"Object": {...}} -> {...}
        let json_result = unwrap_surreal_types(json_result);

        // If it's an array, return as vec; otherwise wrap in vec
        match json_result {
            Value::Array(arr) => Ok(arr.into_iter().collect()),
            v => Ok(vec![v]),
        }
    }

    /// Save audit log
    pub async fn save_audit_log(
        &self,
        log: &crate::audit::AuditLog,
    ) -> PdfResult<String> {
        let record_id = format!("audit_log:⟨{}⟩", log.id);
        let data: Value = serde_json::to_value(log).map_err(|e| {
            PdfModuleError::StorageError(format!("Serialize audit log failed: {}", e))
        })?;

        // Remove the id and execution_id fields from data to avoid conflict with SurrealDB's record ID
        let mut data = data;
        if let Some(obj) = data.as_object_mut() {
            obj.remove("id");
            obj.remove("execution_id");
        }

        let data_str = serde_json::to_string(&data).map_err(|e| {
            PdfModuleError::StorageError(format!("Serialize audit log data failed: {}", e))
        })?;

        let sql = format!("CREATE {} CONTENT {}", record_id, data_str);
        self.execute_query(&sql).await?;

        Ok(format!("audit_log:{}", log.id))
    }

    /// Query audit logs
    pub async fn query_audit_logs(
        &self,
        tool_name: Option<&str>,
        limit: u32,
    ) -> PdfResult<Vec<Value>> {
        let sql = match tool_name {
            Some(name) => format!(
                "SELECT * FROM audit_log WHERE tool_name = '{}' ORDER BY timestamp DESC LIMIT {}",
                name, limit
            ),
            None => format!(
                "SELECT * FROM audit_log ORDER BY timestamp DESC LIMIT {}",
                limit
            ),
        };

        self.execute_query(&sql).await
    }

    /// Get the database reference for advanced operations
    pub fn db(&self) -> &Surreal<Any> {
        &self.db
    }

    /// Get the configuration
    pub fn config(&self) -> &SurrealStoreConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surreal_store_config_default() {
        let config = SurrealStoreConfig::default();
        assert_eq!(config.namespace, "mcp");
        assert_eq!(config.database, "etl");
        assert_eq!(config.path, PathBuf::from("data/pdf_mcp.db"));
    }
}
