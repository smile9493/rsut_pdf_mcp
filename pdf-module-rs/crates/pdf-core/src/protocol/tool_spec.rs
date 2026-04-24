//! Tool spec protocol
//! Defines the runtime configuration schema for MCP tools

use crate::dto::Property;
use crate::error::{PdfModuleError, PdfResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool runtime configuration protocol
/// Defines the configuration parameters required for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    /// Configuration title
    pub title: String,

    /// Configuration description
    pub description: String,

    /// Configuration type (fixed to "object")
    #[serde(rename = "type")]
    pub spec_type: String,

    /// Required parameters list
    pub required: Vec<String>,

    /// Configuration properties
    pub properties: HashMap<String, Property>,
}

impl ToolSpec {
    /// Create a new tool spec
    pub fn new(title: String, description: String) -> Self {
        Self {
            title,
            description,
            spec_type: "object".to_string(),
            required: vec![],
            properties: HashMap::new(),
        }
    }

    /// Validate the tool spec
    pub fn validate(&self) -> PdfResult<()> {
        // Check spec_type is "object"
        if self.spec_type != "object" {
            return Err(PdfModuleError::InvalidToolDefinition(format!(
                "spec_type must be 'object', got '{}'",
                self.spec_type
            )));
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

        // Check that all required properties exist
        for req in &self.required {
            if !self.properties.contains_key(req) {
                return Err(PdfModuleError::InvalidToolDefinition(format!(
                    "Required property '{}' not found in properties",
                    req
                )));
            }
        }

        // Validate each property
        for (name, prop) in &self.properties {
            if name.is_empty() {
                return Err(PdfModuleError::InvalidToolDefinition(
                    "Property name cannot be empty".to_string(),
                ));
            }
            if prop.title.is_empty() {
                return Err(PdfModuleError::InvalidToolDefinition(format!(
                    "Property title cannot be empty for property '{}'",
                    name
                )));
            }
            if prop.description.is_empty() {
                return Err(PdfModuleError::InvalidToolDefinition(format!(
                    "Property description cannot be empty for property '{}'",
                    name
                )));
            }
        }

        Ok(())
    }

    /// Check if the tool spec is valid
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Add a property to the spec
    pub fn add_property(&mut self, name: String, property: Property, required: bool) {
        if required && !self.required.contains(&name) {
            self.required.push(name.clone());
        }
        self.properties.insert(name, property);
    }

    /// Get a property by name
    pub fn get_property(&self, name: &str) -> Option<&Property> {
        self.properties.get(name)
    }

    /// Check if a property is required
    pub fn is_required(&self, name: &str) -> bool {
        self.required.contains(&name.to_string())
    }

    /// Get all required property names
    pub fn required_properties(&self) -> Vec<&str> {
        self.required.iter().map(|s| s.as_str()).collect()
    }

    /// Get all optional property names
    pub fn optional_properties(&self) -> Vec<&str> {
        self.properties
            .keys()
            .filter(|name| !self.required.contains(*name))
            .map(|s| s.as_str())
            .collect()
    }

    /// Remove a property
    pub fn remove_property(&mut self, name: &str) -> Option<Property> {
        self.required.retain(|r| r != name);
        self.properties.remove(name)
    }

    /// Clear all properties
    pub fn clear_properties(&mut self) {
        self.required.clear();
        self.properties.clear();
    }

    /// Get the number of properties
    pub fn property_count(&self) -> usize {
        self.properties.len()
    }
}

impl Default for ToolSpec {
    fn default() -> Self {
        Self::new(
            "Tool Configuration".to_string(),
            "Tool runtime configuration".to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{Property, PropertyType};

    #[test]
    fn test_tool_spec_creation() {
        let spec = ToolSpec::new(
            "PDF Extraction Config".to_string(),
            "Configuration for PDF extraction tool".to_string(),
        );

        assert_eq!(spec.title, "PDF Extraction Config");
        assert_eq!(spec.spec_type, "object");
        assert_eq!(spec.properties.len(), 0);
        assert_eq!(spec.required.len(), 0);
    }

    #[test]
    fn test_tool_spec_validation() {
        let spec = ToolSpec::new("Test Config".to_string(), "Test configuration".to_string());

        assert!(spec.validate().is_ok());
        assert!(spec.is_valid());
    }

    #[test]
    fn test_tool_spec_validation_invalid_type() {
        let spec = ToolSpec::new("Test Config".to_string(), "Test configuration".to_string());
        let mut spec = spec;
        spec.spec_type = "invalid".to_string();

        assert!(spec.validate().is_err());
        assert!(!spec.is_valid());
    }

    #[test]
    fn test_tool_spec_add_property() {
        let mut spec = ToolSpec::new("Test Config".to_string(), "Test configuration".to_string());

        let property = Property {
            property_type: PropertyType::String,
            title: "File Path".to_string(),
            description: "Path to the PDF file".to_string(),
            enum_values: None,
            default: None,
        };

        spec.add_property("file_path".to_string(), property, true);

        assert_eq!(spec.properties.len(), 1);
        assert_eq!(spec.required.len(), 1);
        assert!(spec.is_required("file_path"));

        let retrieved = spec.get_property("file_path");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "File Path");
    }

    #[test]
    fn test_tool_spec_add_optional_property() {
        let mut spec = ToolSpec::new("Test Config".to_string(), "Test configuration".to_string());

        let property = Property {
            property_type: PropertyType::String,
            title: "Adapter".to_string(),
            description: "PDF adapter to use".to_string(),
            enum_values: None,
            default: None,
        };

        spec.add_property("adapter".to_string(), property, false);

        assert_eq!(spec.properties.len(), 1);
        assert_eq!(spec.required.len(), 0);
        assert!(!spec.is_required("adapter"));

        let optional = spec.optional_properties();
        assert_eq!(optional.len(), 1);
        assert!(optional.contains(&"adapter"));
    }

    #[test]
    fn test_tool_spec_remove_property() {
        let mut spec = ToolSpec::new("Test Config".to_string(), "Test configuration".to_string());

        let property = Property {
            property_type: PropertyType::String,
            title: "File Path".to_string(),
            description: "Path to the PDF file".to_string(),
            enum_values: None,
            default: None,
        };

        spec.add_property("file_path".to_string(), property, true);
        assert_eq!(spec.properties.len(), 1);

        let removed = spec.remove_property("file_path");
        assert!(removed.is_some());
        assert_eq!(spec.properties.len(), 0);
        assert_eq!(spec.required.len(), 0);

        let removed_again = spec.remove_property("file_path");
        assert!(removed_again.is_none());
    }

    #[test]
    fn test_tool_spec_validation_missing_required() {
        let mut spec = ToolSpec::new("Test Config".to_string(), "Test configuration".to_string());

        // Add a property to required list but not to properties
        spec.required.push("missing_property".to_string());

        assert!(spec.validate().is_err());
    }

    #[test]
    fn test_tool_spec_clear_properties() {
        let mut spec = ToolSpec::new("Test Config".to_string(), "Test configuration".to_string());

        let property = Property {
            property_type: PropertyType::String,
            title: "File Path".to_string(),
            description: "Path to the PDF file".to_string(),
            enum_values: None,
            default: None,
        };

        spec.add_property("file_path".to_string(), property, true);
        assert_eq!(spec.property_count(), 1);

        spec.clear_properties();
        assert_eq!(spec.property_count(), 0);
        assert_eq!(spec.required.len(), 0);
    }

    #[test]
    fn test_tool_spec_default() {
        let spec = ToolSpec::default();

        assert_eq!(spec.title, "Tool Configuration");
        assert_eq!(spec.description, "Tool runtime configuration");
        assert_eq!(spec.spec_type, "object");
    }
}
