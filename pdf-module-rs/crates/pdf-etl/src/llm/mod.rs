//! LLM 适配器模块

pub mod adapter;
pub mod azure;
pub mod factory;
pub mod ollama;
pub mod openai;

pub use adapter::{LLMAdapter, LLMRequest, LLMResponse, Message, ResponseFormat};
pub use factory::LLMAdapterFactory;
