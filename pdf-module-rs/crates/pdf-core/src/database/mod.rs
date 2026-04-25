//! SurrealDB database module
//! Provides embedded database using SurrealDB with RocksDB backend

pub mod surreal_store;

pub use surreal_store::{SurrealStore, SurrealStoreConfig};
