//! Tool message types
//! Defines the various message types used in MCP tool communication

use crate::dto::{LogLevel, ToolExecutionResult};
use crate::protocol::{ToolDefinition, ToolSpec, RuntimeVariables};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// MCP tool message type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ToolMessage {
    /// Tool specification message
    Spec {
        spec: ToolSpec,
        emitted_at: DateTime<Utc>,
    },

    /// Tool properties message
    Properties {
        properties: ToolDefinition,
        emitted_at: DateTime<Utc>,
    },

    /// Tool icon message
    Icon {
        icon: String, // SVG content
        emitted_at: DateTime<Utc>,
    },

    /// Runtime variables message
    Variables {
        variables: RuntimeVariables,
        emitted_at: DateTime<Utc>,
    },

    /// Log message
    Log {
        level: LogLevel,
        log: String,
        emitted_at: DateTime<Utc>,
    },

    /// Cost tracking message
    Cost {
        cost: f64,
        cost_units: String,
        emitted_at: DateTime<Utc>,
    },

    /// Result message
    Result {
        result: ToolExecutionResult,
        emitted_at: DateTime<Utc>,
    },

    /// Single step debug message
    SingleStepMessage {
        message: String,
        emitted_at: DateTime<Utc>,
    },
}

impl ToolMessage {
    /// Create a new spec message
    pub fn spec(spec: ToolSpec) -> Self {
        Self::Spec {
            spec,
            emitted_at: Utc::now(),
        }
    }

    /// Create a new properties message
    pub fn properties(properties: ToolDefinition) -> Self {
        Self::Properties {
            properties,
            emitted_at: Utc::now(),
        }
    }

    /// Create a new icon message
    pub fn icon(icon: String) -> Self {
        Self::Icon {
            icon,
            emitted_at: Utc::now(),
        }
    }

    /// Create a new variables message
    pub fn variables(variables: RuntimeVariables) -> Self {
        Self::Variables {
            variables,
            emitted_at: Utc::now(),
        }
    }

    /// Create a new log message
    pub fn log(level: LogLevel, log: String) -> Self {
        Self::Log {
            level,
            log,
            emitted_at: Utc::now(),
        }
    }

    /// Create a new cost message
    pub fn cost(cost: f64, cost_units: String) -> Self {
        Self::Cost {
            cost,
            cost_units,
            emitted_at: Utc::now(),
        }
    }

    /// Create a new result message
    pub fn result(result: ToolExecutionResult) -> Self {
        Self::Result {
            result,
            emitted_at: Utc::now(),
        }
    }

    /// Create a new single step message
    pub fn single_step(message: String) -> Self {
        Self::SingleStepMessage {
            message,
            emitted_at: Utc::now(),
        }
    }

    /// Get the message type as a string
    pub fn message_type(&self) -> &'static str {
        match self {
            Self::Spec { .. } => "Spec",
            Self::Properties { .. } => "Properties",
            Self::Icon { .. } => "Icon",
            Self::Variables { .. } => "Variables",
            Self::Log { .. } => "Log",
            Self::Cost { .. } => "Cost",
            Self::Result { .. } => "Result",
            Self::SingleStepMessage { .. } => "SingleStepMessage",
        }
    }

    /// Get the emitted timestamp
    pub fn emitted_at(&self) -> DateTime<Utc> {
        match self {
            Self::Spec { emitted_at, .. } => *emitted_at,
            Self::Properties { emitted_at, .. } => *emitted_at,
            Self::Icon { emitted_at, .. } => *emitted_at,
            Self::Variables { emitted_at, .. } => *emitted_at,
            Self::Log { emitted_at, .. } => *emitted_at,
            Self::Cost { emitted_at, .. } => *emitted_at,
            Self::Result { emitted_at, .. } => *emitted_at,
            Self::SingleStepMessage { emitted_at, .. } => *emitted_at,
        }
    }

    /// Check if the message is a log message
    pub fn is_log(&self) -> bool {
        matches!(self, Self::Log { .. })
    }

    /// Check if the message is a result message
    pub fn is_result(&self) -> bool {
        matches!(self, Self::Result { .. })
    }

    /// Check if the message is a cost message
    pub fn is_cost(&self) -> bool {
        matches!(self, Self::Cost { .. })
    }

    /// Get the log message content if this is a log message
    pub fn as_log(&self) -> Option<(&LogLevel, &str)> {
        match self {
            Self::Log { level, log, .. } => Some((level, log)),
            _ => None,
        }
    }

    /// Get the result content if this is a result message
    pub fn as_result(&self) -> Option<&ToolExecutionResult> {
        match self {
            Self::Result { result, .. } => Some(result),
            _ => None,
        }
    }

    /// Get the cost content if this is a cost message
    pub fn as_cost(&self) -> Option<(&f64, &str)> {
        match self {
            Self::Cost { cost, cost_units, .. } => Some((cost, cost_units)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_message_creation() {
        let message = ToolMessage::log(LogLevel::Info, "Test message".to_string());
        assert_eq!(message.message_type(), "Log");
        assert!(message.is_log());
        assert!(!message.is_result());
        assert!(!message.is_cost());
    }

    #[test]
    fn test_tool_message_log() {
        let message = ToolMessage::log(LogLevel::Info, "Test message".to_string());

        let (level, log) = message.as_log().unwrap();
        assert_eq!(*level, LogLevel::Info);
        assert_eq!(log, "Test message");

        assert_eq!(message.emitted_at().timestamp(), Utc::now().timestamp());
    }

    #[test]
    fn test_tool_message_result() {
        let result = ToolExecutionResult {
            workflow_id: "test-workflow".to_string(),
            elapsed_time: 1000,
            output: serde_json::json!({"status": "success"}),
            metadata: None,
        };

        let message = ToolMessage::result(result.clone());
        assert_eq!(message.message_type(), "Result");
        assert!(message.is_result());
        assert!(!message.is_log());

        let retrieved_result = message.as_result().unwrap();
        assert_eq!(retrieved_result.workflow_id, "test-workflow");
    }

    #[test]
    fn test_tool_message_cost() {
        let message = ToolMessage::cost(0.01, "USD".to_string());
        assert_eq!(message.message_type(), "Cost");
        assert!(message.is_cost());
        assert!(!message.is_log());
        assert!(!message.is_result());

        let (cost, units) = message.as_cost().unwrap();
        assert_eq!(*cost, 0.01);
        assert_eq!(units, "USD");
    }

    #[test]
    fn test_tool_message_spec() {
        let spec = ToolSpec::new(
            "Test Config".to_string(),
            "Test configuration".to_string(),
        );

        let message = ToolMessage::spec(spec);
        assert_eq!(message.message_type(), "Spec");
        assert!(!message.is_log());
        assert!(!message.is_result());
        assert!(!message.is_cost());
    }

    #[test]
    fn test_tool_message_serialization() {
        let message = ToolMessage::log(LogLevel::Info, "Test message".to_string());

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("\"type\":\"Log\""));
        assert!(json.contains("Test message"));

        let deserialized: ToolMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.message_type(), "Log");
    }

    #[test]
    fn test_tool_message_all_types() {
        // Test all message types can be created and serialized
        let messages = vec![
            ToolMessage::spec(ToolSpec::new(
                "Test".to_string(),
                "Test".to_string(),
            )),
            ToolMessage::properties(ToolDefinition::new(
                "Test".to_string(),
                "test".to_string(),
                "Test".to_string(),
                vec![],
                crate::dto::InputType::File,
                crate::dto::OutputType::File,
            )),
            ToolMessage::icon("<svg></svg>".to_string()),
            ToolMessage::variables(RuntimeVariables::new(
                "Test".to_string(),
                "Test".to_string(),
            )),
            ToolMessage::log(LogLevel::Info, "Test".to_string()),
            ToolMessage::cost(0.01, "USD".to_string()),
            ToolMessage::result(ToolExecutionResult {
                workflow_id: "test".to_string(),
                elapsed_time: 1000,
                output: serde_json::json!({}),
                metadata: None,
            }),
            ToolMessage::single_step("Step 1".to_string()),
        ];

        for message in messages {
            let json = serde_json::to_string(&message).unwrap();
            let _: ToolMessage = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_tool_message_timestamp() {
        let before = Utc::now();
        let message = ToolMessage::log(LogLevel::Info, "Test".to_string());
        let after = Utc::now();

        let timestamp = message.emitted_at();
        assert!(timestamp >= before);
        assert!(timestamp <= after);
    }
}
