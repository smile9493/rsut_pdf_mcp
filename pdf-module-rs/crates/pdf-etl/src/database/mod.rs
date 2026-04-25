//! 数据库适配器模块

pub mod adapter;
pub mod factory;
pub mod mysql;
pub mod postgres;
pub mod sqlite;

pub use adapter::DatabaseAdapter;
pub use factory::DatabaseAdapterFactory;
