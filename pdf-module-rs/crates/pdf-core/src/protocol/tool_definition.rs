//! Tool definition protocol
//! Defines the metadata and schema for MCP tools

use crate::dto::{InputType, OutputType, Parameter, ToolRequirements};
use crate::error::{PdfModuleError, PdfResult};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Tool definition protocol
/// Contains tool metadata, parameter schema, version history, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool display name
    pub display_name: String,

    /// Tool unique identifier (function name)
    pub function_name: String,

    /// Tool description
    pub description: String,

    /// Tool parameter definitions
    pub parameters: Vec<Parameter>,

    /// Tool version history
    pub versions: Vec<String>,

    /// Whether the tool supports caching
    pub is_cacheable: bool,

    /// Input type
    pub input_type: InputType,

    /// Output type
    pub output_type: OutputType,

    /// Tool requirements
    pub requires: ToolRequirements,
}

impl ToolDefinition {
    /// Create a new tool definition
    pub fn new(
        display_name: String,
        function_name: String,
        description: String,
        parameters: Vec<Parameter>,
        input_type: InputType,
        output_type: OutputType,
    ) -> Self {
        Self {
            display_name,
            function_name,
            description,
            parameters,
            versions: vec!["1.0.0".to_string()],
            is_cacheable: true,
            input_type,
            output_type,
            requires: ToolRequirements {
                files: crate::dto::ResourceRequirement {
                    input: true,
                    output: false,
                },
                databases: crate::dto::ResourceRequirement {
                    input: false,
                    output: false,
                },
            },
        }
    }

    /// Validate the tool definition
    pub fn validate(&self) -> PdfResult<()> {
        // Check function name is not empty
        if self.function_name.is_empty() {
            return Err(PdfModuleError::InvalidToolDefinition(
                "function_name cannot be empty".to_string(),
            ));
        }

        // Check display name is not empty
        if self.display_name.is_empty() {
            return Err(PdfModuleError::InvalidToolDefinition(
                "display_name cannot be empty".to_string(),
            ));
        }

        // Check description is not empty
        if self.description.is_empty() {
            return Err(PdfModuleError::InvalidToolDefinition(
                "description cannot be empty".to_string(),
            ));
        }

        // Check parameter names are unique
        let param_names: HashSet<_> = self.parameters.iter().map(|p| &p.name).collect();
        if param_names.len() != self.parameters.len() {
            return Err(PdfModuleError::InvalidToolDefinition(
                "Parameter names must be unique".to_string(),
            ));
        }

        // Check versions is not empty
        if self.versions.is_empty() {
            return Err(PdfModuleError::InvalidToolDefinition(
                "versions cannot be empty".to_string(),
            ));
        }

        // Validate each parameter
        for param in &self.parameters {
            if param.name.is_empty() {
                return Err(PdfModuleError::InvalidToolDefinition(format!(
                    "Parameter name cannot be empty for parameter: {:?}",
                    param
                )));
            }
            if param.description.is_empty() {
                return Err(PdfModuleError::InvalidToolDefinition(format!(
                    "Parameter description cannot be empty for parameter: {}",
                    param.name
                )));
            }
        }

        Ok(())
    }

    /// Check if the tool definition is valid
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Get the latest version
    pub fn latest_version(&self) -> &str {
        self.versions.last().map(|s| s.as_str()).unwrap_or("1.0.0")
    }

    /// Add a new version
    pub fn add_version(&mut self, version: String) {
        if !self.versions.contains(&version) {
            self.versions.push(version);
        }
    }

    /// Get required parameter names
    pub fn required_parameters(&self) -> Vec<&str> {
        self.parameters
            .iter()
            .filter(|p| p.required)
            .map(|p| p.name.as_str())
            .collect()
    }

    /// Get optional parameter names
    pub fn optional_parameters(&self) -> Vec<&str> {
        self.parameters
            .iter()
            .filter(|p| !p.required)
            .map(|p| p.name.as_str())
            .collect()
    }

    /// Get parameter by name
    pub fn get_parameter(&self, name: &str) -> Option<&Parameter> {
        self.parameters.iter().find(|p| p.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{Parameter, ParameterType};

    #[test]
    fn test_tool_definition_creation() {
        let param = Parameter {
            name: "file_path".to_string(),
            param_type: ParameterType::String,
            description: "Path to the PDF file".to_string(),
            required: true,
            default: None,
            enum_values: None,
        };

        let tool = ToolDefinition::new(
            "Extract Text".to_string(),
            "extract_text".to_string(),
            "Extract text from PDF file".to_string(),
            vec![param],
            InputType::File,
            OutputType::File,
        );

        assert_eq!(tool.function_name, "extract_text");
        assert_eq!(tool.display_name, "Extract Text");
        assert_eq!(tool.versions, vec!["1.0.0"]);
        assert!(tool.is_cacheable);
    }

    #[test]
    fn test_tool_definition_validation() {
        let param = Parameter {
            name: "file_path".to_string(),
            param_type: ParameterType::String,
            description: "Path to the PDF file".to_string(),
            required: true,
            default: None,
            enum_values: None,
        };

        let tool = ToolDefinition::new(
            "Extract Text".to_string(),
            "extract_text".to_string(),
            "Extract text from PDF file".to_string(),
            vec![param],
            InputType::File,
            OutputType::File,
        );

        assert!(tool.validate().is_ok());
        assert!(tool.is_valid());
    }

    #[test]
    fn test_tool_definition_validation_empty_function_name() {
        let tool = ToolDefinition {
            display_name: "Test".to_string(),
            function_name: "".to_string(),
            description: "Test".to_string(),
            parameters: vec![],
            versions: vec!["1.0.0".to_string()],
            is_cacheable: true,
            input_type: InputType::File,
            output_type: OutputType::File,
            requires: ToolRequirements {
                files: crate::dto::ResourceRequirement {
                    input: true,
                    output: false,
                },
                databases: crate::dto::ResourceRequirement {
                    input: false,
                    output: false,
                },
            },
        };

        assert!(tool.validate().is_err());
        assert!(!tool.is_valid());
    }

    #[test]
    fn test_tool_definition_validation_duplicate_parameters() {
        let param1 = Parameter {
            name: "file_path".to_string(),
            param_type: ParameterType::String,
            description: "Path to the PDF file".to_string(),
            required: true,
            default: None,
            enum_values: None,
        };

        let param2 = Parameter {
            name: "file_path".to_string(),
            param_type: ParameterType::String,
            description: "Another path".to_string(),
            required: false,
            default: None,
            enum_values: None,
        };

        let tool = ToolDefinition {
            display_name: "Test".to_string(),
            function_name: "test".to_string(),
            description: "Test".to_string(),
            parameters: vec![param1, param2],
            versions: vec!["1.0.0".to_string()],
            is_cacheable: true,
            input_type: InputType::File,
            output_type: OutputType::File,
            requires: ToolRequirements {
                files: crate::dto::ResourceRequirement {
                    input: true,
                    output: false,
                },
                databases: crate::dto::ResourceRequirement {
                    input: false,
                    output: false,
                },
            },
        };

        assert!(tool.validate().is_err());
    }

    #[test]
    fn test_tool_definition_version_management() {
        let param = Parameter {
            name: "file_path".to_string(),
            param_type: ParameterType::String,
            description: "Path to the PDF file".to_string(),
            required: true,
            default: None,
            enum_values: None,
        };

        let mut tool = ToolDefinition::new(
            "Extract Text".to_string(),
            "extract_text".to_string(),
            "Extract text from PDF file".to_string(),
            vec![param],
            InputType::File,
            OutputType::File,
        );

        assert_eq!(tool.latest_version(), "1.0.0");

        tool.add_version("1.1.0".to_string());
        assert_eq!(tool.latest_version(), "1.1.0");
        assert_eq!(tool.versions.len(), 2);

        // Adding duplicate version should not increase count
        tool.add_version("1.1.0".to_string());
        assert_eq!(tool.versions.len(), 2);
    }

    #[test]
    fn test_tool_definition_parameter_access() {
        let param1 = Parameter {
            name: "file_path".to_string(),
            param_type: ParameterType::String,
            description: "Path to the PDF file".to_string(),
            required: true,
            default: None,
            enum_values: None,
        };

        let param2 = Parameter {
            name: "adapter".to_string(),
            param_type: ParameterType::String,
            description: "PDF adapter to use".to_string(),
            required: false,
            default: None,
            enum_values: None,
        };

        let tool = ToolDefinition {
            display_name: "Test".to_string(),
            function_name: "test".to_string(),
            description: "Test".to_string(),
            parameters: vec![param1, param2],
            versions: vec!["1.0.0".to_string()],
            is_cacheable: true,
            input_type: InputType::File,
            output_type: OutputType::File,
            requires: ToolRequirements {
                files: crate::dto::ResourceRequirement {
                    input: true,
                    output: false,
                },
                databases: crate::dto::ResourceRequirement {
                    input: false,
                    output: false,
                },
            },
        };

        let required = tool.required_parameters();
        assert_eq!(required.len(), 1);
        assert!(required.contains(&"file_path"));

        let optional = tool.optional_parameters();
        assert_eq!(optional.len(), 1);
        assert!(optional.contains(&"adapter"));

        let found_param = tool.get_parameter("file_path");
        assert!(found_param.is_some());
        assert_eq!(found_param.unwrap().name, "file_path");

        let not_found = tool.get_parameter("nonexistent");
        assert!(not_found.is_none());
    }
}
