//! Runtime variables protocol
//! Defines the environment variables available during tool execution

use crate::dto::VariableProperty;
use crate::error::{PdfModuleError, PdfResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime environment variables protocol
/// Defines the environment variables available during tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeVariables {
    /// Variable title
    pub title: String,

    /// Variable description
    pub description: String,

    /// Variable type (fixed to "object")
    #[serde(rename = "type")]
    pub var_type: String,

    /// Required variables list
    pub required: Vec<String>,

    /// Variable properties
    pub properties: HashMap<String, VariableProperty>,
}

impl RuntimeVariables {
    /// Create a new runtime variables definition
    pub fn new(title: String, description: String) -> Self {
        Self {
            title,
            description,
            var_type: "object".to_string(),
            required: vec![],
            properties: HashMap::new(),
        }
    }

    /// Validate the runtime variables
    pub fn validate(&self) -> PdfResult<()> {
        // Check var_type is "object"
        if self.var_type != "object" {
            return Err(PdfModuleError::InvalidToolDefinition(
                format!("var_type must be 'object', got '{}'", self.var_type),
            ));
        }

        // Check title is not empty
        if self.title.is_empty() {
            return Err(PdfModuleError::InvalidToolDefinition(
                "title cannot be empty".to_string(),
            ));
        }

        // Check description is not empty
        if self.description.is_empty() {
            return Err(PdfModuleError::InvalidToolDefinition(
                "description cannot be empty".to_string(),
            ));
        }

        // Check that all required variables exist
        for req in &self.required {
            if !self.properties.contains_key(req) {
                return Err(PdfModuleError::InvalidToolDefinition(format!(
                    "Required variable '{}' not found in properties",
                    req
                )));
            }
        }

        // Validate each variable property
        for (name, prop) in &self.properties {
            if name.is_empty() {
                return Err(PdfModuleError::InvalidToolDefinition(
                    "Variable name cannot be empty".to_string(),
                ));
            }
            if prop.title.is_empty() {
                return Err(PdfModuleError::InvalidToolDefinition(format!(
                    "Variable title cannot be empty for variable '{}'",
                    name
                )));
            }
            if prop.description.is_empty() {
                return Err(PdfModuleError::InvalidToolDefinition(format!(
                    "Variable description cannot be empty for variable '{}'",
                    name
                )));
            }
        }

        Ok(())
    }

    /// Check if the runtime variables are valid
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Add a variable
    pub fn add_variable(&mut self, name: String, property: VariableProperty, required: bool) {
        if required && !self.required.contains(&name) {
            self.required.push(name.clone());
        }
        self.properties.insert(name, property);
    }

    /// Get a variable by name
    pub fn get_variable(&self, name: &str) -> Option<&VariableProperty> {
        self.properties.get(name)
    }

    /// Check if a variable is required
    pub fn is_required(&self, name: &str) -> bool {
        self.required.contains(&name.to_string())
    }

    /// Get all required variable names
    pub fn required_variables(&self) -> Vec<&str> {
        self.required.iter().map(|s| s.as_str()).collect()
    }

    /// Get all optional variable names
    pub fn optional_variables(&self) -> Vec<&str> {
        self.properties
            .keys()
            .filter(|name| !self.required.contains(*name))
            .map(|s| s.as_str())
            .collect()
    }

    /// Remove a variable
    pub fn remove_variable(&mut self, name: &str) -> Option<VariableProperty> {
        self.required.retain(|r| r != name);
        self.properties.remove(name)
    }

    /// Clear all variables
    pub fn clear_variables(&mut self) {
        self.required.clear();
        self.properties.clear();
    }

    /// Get the number of variables
    pub fn variable_count(&self) -> usize {
        self.properties.len()
    }

    /// Set a variable value (for runtime use)
    pub fn set_value(&mut self, name: &str, _value: serde_json::Value) -> PdfResult<()> {
        // This is a placeholder for runtime variable setting
        // In a real implementation, you might want to store values separately
        if !self.properties.contains_key(name) {
            return Err(PdfModuleError::InvalidToolDefinition(format!(
                "Variable '{}' not found in runtime variables",
                name
            )));
        }
        Ok(())
    }

    /// Get a variable value (for runtime use)
    pub fn get_value(&self, name: &str) -> PdfResult<serde_json::Value> {
        // This is a placeholder for runtime variable getting
        // In a real implementation, you might want to retrieve values from a separate store
        if !self.properties.contains_key(name) {
            return Err(PdfModuleError::InvalidToolDefinition(format!(
                "Variable '{}' not found in runtime variables",
                name
            )));
        }
        Ok(serde_json::Value::Null)
    }
}

impl Default for RuntimeVariables {
    fn default() -> Self {
        Self::new(
            "Runtime Variables".to_string(),
            "Environment variables for tool execution".to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{PropertyType, VariableProperty};

    #[test]
    fn test_runtime_variables_creation() {
        let vars = RuntimeVariables::new(
            "PDF Extraction Variables".to_string(),
            "Runtime variables for PDF extraction".to_string(),
        );

        assert_eq!(vars.title, "PDF Extraction Variables");
        assert_eq!(vars.var_type, "object");
        assert_eq!(vars.properties.len(), 0);
        assert_eq!(vars.required.len(), 0);
    }

    #[test]
    fn test_runtime_variables_validation() {
        let vars = RuntimeVariables::new(
            "Test Variables".to_string(),
            "Test runtime variables".to_string(),
        );

        assert!(vars.validate().is_ok());
        assert!(vars.is_valid());
    }

    #[test]
    fn test_runtime_variables_validation_invalid_type() {
        let mut vars = RuntimeVariables::new(
            "Test Variables".to_string(),
            "Test runtime variables".to_string(),
        );
        vars.var_type = "invalid".to_string();

        assert!(vars.validate().is_err());
        assert!(!vars.is_valid());
    }

    #[test]
    fn test_runtime_variables_add_variable() {
        let mut vars = RuntimeVariables::new(
            "Test Variables".to_string(),
            "Test runtime variables".to_string(),
        );

        let property = VariableProperty {
            property_type: PropertyType::String,
            title: "Cache Dir".to_string(),
            description: "Cache directory path".to_string(),
        };

        vars.add_variable("cache_dir".to_string(), property, true);

        assert_eq!(vars.properties.len(), 1);
        assert_eq!(vars.required.len(), 1);
        assert!(vars.is_required("cache_dir"));

        let retrieved = vars.get_variable("cache_dir");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Cache Dir");
    }

    #[test]
    fn test_runtime_variables_add_optional_variable() {
        let mut vars = RuntimeVariables::new(
            "Test Variables".to_string(),
            "Test runtime variables".to_string(),
        );

        let property = VariableProperty {
            property_type: PropertyType::String,
            title: "Temp Dir".to_string(),
            description: "Temporary directory path".to_string(),
        };

        vars.add_variable("temp_dir".to_string(), property, false);

        assert_eq!(vars.properties.len(), 1);
        assert_eq!(vars.required.len(), 0);
        assert!(!vars.is_required("temp_dir"));

        let optional = vars.optional_variables();
        assert_eq!(optional.len(), 1);
        assert!(optional.contains(&"temp_dir"));
    }

    #[test]
    fn test_runtime_variables_remove_variable() {
        let mut vars = RuntimeVariables::new(
            "Test Variables".to_string(),
            "Test runtime variables".to_string(),
        );

        let property = VariableProperty {
            property_type: PropertyType::String,
            title: "Cache Dir".to_string(),
            description: "Cache directory path".to_string(),
        };

        vars.add_variable("cache_dir".to_string(), property, true);
        assert_eq!(vars.properties.len(), 1);

        let removed = vars.remove_variable("cache_dir");
        assert!(removed.is_some());
        assert_eq!(vars.properties.len(), 0);
        assert_eq!(vars.required.len(), 0);

        let removed_again = vars.remove_variable("cache_dir");
        assert!(removed_again.is_none());
    }

    #[test]
    fn test_runtime_variables_validation_missing_required() {
        let mut vars = RuntimeVariables::new(
            "Test Variables".to_string(),
            "Test runtime variables".to_string(),
        );

        // Add a variable to required list but not to properties
        vars.required.push("missing_variable".to_string());

        assert!(vars.validate().is_err());
    }

    #[test]
    fn test_runtime_variables_clear_variables() {
        let mut vars = RuntimeVariables::new(
            "Test Variables".to_string(),
            "Test runtime variables".to_string(),
        );

        let property = VariableProperty {
            property_type: PropertyType::String,
            title: "Cache Dir".to_string(),
            description: "Cache directory path".to_string(),
        };

        vars.add_variable("cache_dir".to_string(), property, true);
        assert_eq!(vars.variable_count(), 1);

        vars.clear_variables();
        assert_eq!(vars.variable_count(), 0);
        assert_eq!(vars.required.len(), 0);
    }

    #[test]
    fn test_runtime_variables_default() {
        let vars = RuntimeVariables::default();

        assert_eq!(vars.title, "Runtime Variables");
        assert_eq!(vars.description, "Environment variables for tool execution");
        assert_eq!(vars.var_type, "object");
    }

    #[test]
    fn test_runtime_variables_value_operations() {
        let mut vars = RuntimeVariables::new(
            "Test Variables".to_string(),
            "Test runtime variables".to_string(),
        );

        let property = VariableProperty {
            property_type: PropertyType::String,
            title: "Cache Dir".to_string(),
            description: "Cache directory path".to_string(),
        };

        vars.add_variable("cache_dir".to_string(), property, false);

        // Test setting value (should succeed)
        let result = vars.set_value("cache_dir", serde_json::Value::String("/tmp/cache".to_string()));
        assert!(result.is_ok());

        // Test getting value (should return null placeholder)
        let value = vars.get_value("cache_dir");
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), serde_json::Value::Null);

        // Test setting value for non-existent variable (should fail)
        let result = vars.set_value("nonexistent", serde_json::Value::String("value".to_string()));
        assert!(result.is_err());

        // Test getting value for non-existent variable (should fail)
        let value = vars.get_value("nonexistent");
        assert!(value.is_err());
    }
}
