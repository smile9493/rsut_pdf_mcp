//! Schema 定义与验证模块

pub mod schema_definition;
pub mod validator;

pub use schema_definition::SchemaDefinition;
pub use validator::{SchemaValidator, ValidationResult};
