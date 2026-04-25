//! Schema 验证器
//!
//! 基于 jsonschema 库实现 JSON 数据验证

use schemars::schema::RootSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{EtlError, Result};

/// Schema 验证器
pub struct SchemaValidator {
    /// 编译后的 Schema
    compiled: jsonschema::JSONSchema,
}

impl SchemaValidator {
    /// 创建新的验证器
    pub fn new(schema: &RootSchema) -> Result<Self> {
        let schema_value = serde_json::to_value(schema)
            .map_err(|e| EtlError::SchemaError(format!("Failed to serialize schema: {}", e)))?;

        let compiled = jsonschema::JSONSchema::compile(&schema_value)
            .map_err(|e| EtlError::SchemaError(format!("Failed to compile schema: {}", e)))?;

        Ok(Self { compiled })
    }

    /// 从 JSON 字符串创建验证器
    pub fn from_json(schema_str: &str) -> Result<Self> {
        let schema_value: Value = serde_json::from_str(schema_str)
            .map_err(|e| EtlError::SchemaError(format!("Failed to parse JSON: {}", e)))?;

        let compiled = jsonschema::JSONSchema::compile(&schema_value)
            .map_err(|e| EtlError::SchemaError(format!("Failed to compile schema: {}", e)))?;

        Ok(Self { compiled })
    }

    /// 验证 JSON 数据
    pub fn validate(&self, data: &Value) -> Result<ValidationResult> {
        let result = self.compiled.validate(data);

        match result {
            Ok(_) => Ok(ValidationResult::valid()),
            Err(errors) => {
                let validation_errors: Vec<ValidationError> = errors
                    .map(|e| ValidationError {
                        path: e.instance_path.to_string(),
                        message: e.to_string(),
                        expected: None,
                        actual: None,
                    })
                    .collect();

                Ok(ValidationResult::invalid(validation_errors))
            }
        }
    }

    /// 快速验证（仅返回是否有效）
    pub fn is_valid(&self, data: &Value) -> bool {
        self.compiled.is_valid(data)
    }
}

/// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 是否有效
    pub is_valid: bool,
    /// 验证错误列表
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    /// 创建有效的验证结果
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    /// 创建无效的验证结果
    pub fn invalid(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }

    /// 获取错误消息列表
    pub fn error_messages(&self) -> Vec<String> {
        self.errors.iter().map(|e| e.message.clone()).collect()
    }

    /// 获取格式化的错误消息
    pub fn formatted_errors(&self) -> String {
        if self.is_valid {
            "Validation passed".to_string()
        } else {
            format!(
                "Validation failed:\n{}",
                self.errors
                    .iter()
                    .map(|e| format!("  - [{}] {}", e.path, e.message))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        }
    }
}

/// 验证错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// 错误路径
    pub path: String,
    /// 错误消息
    pub message: String,
    /// 期望值
    pub expected: Option<String>,
    /// 实际值
    pub actual: Option<String>,
}

impl ValidationError {
    /// 创建新的验证错误
    pub fn new(path: String, message: String) -> Self {
        Self {
            path,
            message,
            expected: None,
            actual: None,
        }
    }

    /// 设置期望值
    pub fn with_expected(mut self, expected: String) -> Self {
        self.expected = Some(expected);
        self
    }

    /// 设置实际值
    pub fn with_actual(mut self, actual: String) -> Self {
        self.actual = Some(actual);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::JsonSchema;

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    struct TestSchema {
        name: String,
        age: u32,
        active: bool,
    }

    #[test]
    fn test_validator_new() {
        let schema = schemars::schema_for!(TestSchema);
        let validator = SchemaValidator::new(&schema).unwrap();
        assert!(validator.is_valid(&serde_json::json!({
            "name": "test",
            "age": 25,
            "active": true
        })));
    }

    #[test]
    fn test_validate_valid_data() {
        let schema = schemars::schema_for!(TestSchema);
        let validator = SchemaValidator::new(&schema).unwrap();

        let data = serde_json::json!({
            "name": "test",
            "age": 25,
            "active": true
        });

        let result = validator.validate(&data).unwrap();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_invalid_data() {
        let schema = schemars::schema_for!(TestSchema);
        let validator = SchemaValidator::new(&schema).unwrap();

        // 缺少必需字段
        let data = serde_json::json!({
            "name": "test"
        });

        let result = validator.validate(&data).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_type_mismatch() {
        let schema = schemars::schema_for!(TestSchema);
        let validator = SchemaValidator::new(&schema).unwrap();

        // 类型错误
        let data = serde_json::json!({
            "name": "test",
            "age": "not a number",
            "active": true
        });

        let result = validator.validate(&data).unwrap();
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validation_result() {
        let result = ValidationResult::valid();
        assert!(result.is_valid);
        assert!(result.error_messages().is_empty());

        let errors = vec![
            ValidationError::new("field1".to_string(), "is required".to_string()),
            ValidationError::new("field2".to_string(), "must be positive".to_string()),
        ];
        let result = ValidationResult::invalid(errors);
        assert!(!result.is_valid);
        assert_eq!(result.error_messages().len(), 2);
    }

    #[test]
    fn test_validation_error() {
        let error = ValidationError::new("path".to_string(), "message".to_string())
            .with_expected("expected".to_string())
            .with_actual("actual".to_string());

        assert_eq!(error.path, "path");
        assert_eq!(error.message, "message");
        assert_eq!(error.expected, Some("expected".to_string()));
        assert_eq!(error.actual, Some("actual".to_string()));
    }
}
