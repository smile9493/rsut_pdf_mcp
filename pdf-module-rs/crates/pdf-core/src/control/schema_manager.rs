//! Schema manager implementation
//! Provides tool schema registration, versioning, and validation

use crate::control::control_plane::SchemaDefinition;
use crate::error::{PdfModuleError, PdfResult};
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Schema manager
/// Manages tool input/output schemas with versioning support
pub struct SchemaManager {
    /// Schema storage: name -> version -> definition
    schemas: RwLock<HashMap<String, HashMap<String, SchemaDefinition>>>,
}

impl SchemaManager {
    /// Create a new schema manager
    pub fn new() -> Self {
        Self {
            schemas: RwLock::new(HashMap::new()),
        }
    }

    /// Register a schema
    pub async fn register(&self, schema: SchemaDefinition) -> PdfResult<()> {
        let mut schemas = self.schemas.write().await;
        schemas
            .entry(schema.name.clone())
            .or_insert_with(HashMap::new)
            .insert(schema.version.clone(), schema);
        Ok(())
    }

    /// Get the latest version of a schema by name
    pub async fn get(&self, name: &str) -> PdfResult<Option<SchemaDefinition>> {
        let schemas = self.schemas.read().await;
        if let Some(versions) = schemas.get(name) {
            // Return the latest version (highest version string)
            let latest = versions
                .iter()
                .max_by_key(|(v, _)| *v)
                .map(|(_, def)| def.clone());
            Ok(latest)
        } else {
            Ok(None)
        }
    }

    /// Get a specific version of a schema
    pub async fn get_version(&self, name: &str, version: &str) -> PdfResult<Option<SchemaDefinition>> {
        let schemas = self.schemas.read().await;
        Ok(schemas
            .get(name)
            .and_then(|versions| versions.get(version))
            .cloned())
    }

    /// List all schema names
    pub async fn list(&self) -> Vec<String> {
        let schemas = self.schemas.read().await;
        schemas.keys().cloned().collect()
    }

    /// List all versions of a schema
    pub async fn list_versions(&self, name: &str) -> Vec<String> {
        let schemas = self.schemas.read().await;
        schemas
            .get(name)
            .map(|versions| versions.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Validate data against a schema
    pub async fn validate(&self, name: &str, data: &Value) -> PdfResult<bool> {
        let schemas = self.schemas.read().await;
        if let Some(versions) = schemas.get(name) {
            if let Some((_version, definition)) = versions.iter().max_by_key(|(v, _)| *v) {
                // Basic JSON Schema validation
                // For production, use jsonschema crate
                return self.validate_against_schema(data, &definition.schema);
            }
        }
        Err(PdfModuleError::SchemaValidationError(format!(
            "Schema '{}' not found",
            name
        )))
    }

    /// Validate data against a specific schema version
    pub async fn validate_version(
        &self,
        name: &str,
        version: &str,
        data: &Value,
    ) -> PdfResult<bool> {
        let schemas = self.schemas.read().await;
        if let Some(versions) = schemas.get(name) {
            if let Some(definition) = versions.get(version) {
                return self.validate_against_schema(data, &definition.schema);
            }
        }
        Err(PdfModuleError::SchemaValidationError(format!(
            "Schema '{}' version '{}' not found",
            name, version
        )))
    }

    /// Remove a schema
    pub async fn remove(&self, name: &str) -> PdfResult<()> {
        let mut schemas = self.schemas.write().await;
        schemas
            .remove(name)
            .ok_or_else(|| PdfModuleError::SchemaValidationError(format!("Schema '{}' not found", name)))?;
        Ok(())
    }

    /// Remove a specific version of a schema
    pub async fn remove_version(&self, name: &str, version: &str) -> PdfResult<()> {
        let mut schemas = self.schemas.write().await;
        if let Some(versions) = schemas.get_mut(name) {
            versions
                .remove(version)
                .ok_or_else(|| {
                    PdfModuleError::SchemaValidationError(format!(
                        "Schema '{}' version '{}' not found",
                        name, version
                    ))
                })?;
            // Clean up empty entries
            if versions.is_empty() {
                schemas.remove(name);
            }
        } else {
            return Err(PdfModuleError::SchemaValidationError(format!(
                "Schema '{}' not found",
                name
            )));
        }
        Ok(())
    }

    /// Get the number of registered schemas
    pub async fn count(&self) -> usize {
        let schemas = self.schemas.read().await;
        schemas.len()
    }

    /// Basic JSON Schema validation (simplified)
    fn validate_against_schema(&self, data: &Value, schema: &Value) -> PdfResult<bool> {
        // Check required fields
        if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
            if let Some(properties) = data.as_object() {
                for field in required {
                    if let Some(field_name) = field.as_str() {
                        if !properties.contains_key(field_name) {
                            return Err(PdfModuleError::SchemaValidationError(format!(
                                "Missing required field: '{}'",
                                field_name
                            )));
                        }
                    }
                }
            }
        }

        // Check type
        if let Some(schema_type) = schema.get("type").and_then(|t| t.as_str()) {
            let matches = match schema_type {
                "object" => data.is_object(),
                "array" => data.is_array(),
                "string" => data.is_string(),
                "number" => data.is_number(),
                "boolean" => data.is_boolean(),
                "null" => data.is_null(),
                _ => true,
            };
            if !matches {
                return Err(PdfModuleError::SchemaValidationError(format!(
                    "Type mismatch: expected '{}'",
                    schema_type
                )));
            }
        }

        Ok(true)
    }
}

impl Default for SchemaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_schema_register_and_get() {
        let manager = SchemaManager::new();

        let schema = SchemaDefinition::new(
            "test_schema".to_string(),
            "1.0.0".to_string(),
            serde_json::json!({
                "type": "object",
                "required": ["name"]
            }),
        );

        manager.register(schema).await.unwrap();

        let result = manager.get("test_schema").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().version, "1.0.0");
    }

    #[tokio::test]
    async fn test_schema_versioning() {
        let manager = SchemaManager::new();

        let v1 = SchemaDefinition::new(
            "test".to_string(),
            "1.0.0".to_string(),
            serde_json::json!({"type": "object"}),
        );
        let v2 = SchemaDefinition::new(
            "test".to_string(),
            "2.0.0".to_string(),
            serde_json::json!({"type": "object"}),
        );

        manager.register(v1).await.unwrap();
        manager.register(v2).await.unwrap();

        let latest = manager.get("test").await.unwrap().unwrap();
        assert_eq!(latest.version, "2.0.0");

        let specific = manager.get_version("test", "1.0.0").await.unwrap().unwrap();
        assert_eq!(specific.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_schema_validation() {
        let manager = SchemaManager::new();

        let schema = SchemaDefinition::new(
            "test".to_string(),
            "1.0.0".to_string(),
            serde_json::json!({
                "type": "object",
                "required": ["name"]
            }),
        );

        manager.register(schema).await.unwrap();

        let valid_data = serde_json::json!({"name": "test"});
        assert!(manager.validate("test", &valid_data).await.unwrap());

        let invalid_data = serde_json::json!({"other": "value"});
        assert!(manager.validate("test", &invalid_data).await.is_err());
    }
}
