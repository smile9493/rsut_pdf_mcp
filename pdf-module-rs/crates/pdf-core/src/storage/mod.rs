//! File storage module
//! Provides unified file storage abstraction with multiple backend support

pub mod file_storage;
pub mod local_storage;
#[cfg(feature = "s3")]
pub mod s3_storage;
pub mod factory;

pub use file_storage::{FileStorage, FileStorageConfig};
pub use local_storage::LocalFileStorage;
#[cfg(feature = "s3")]
pub use s3_storage::S3FileStorage;
pub use factory::FileStorageFactory;
