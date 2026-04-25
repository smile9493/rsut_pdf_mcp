//! S3 file storage implementation
//! Implements FileStorage trait for AWS S3
//!
//! Note: This module requires the "s3" feature to be enabled
//! and the aws-sdk-s3 crate to be added as a dependency.
//!
//! To enable S3 support:
//! 1. Add `aws-sdk-s3` to Cargo.toml dependencies
//! 2. Enable the "s3" feature: `cargo build --features s3`

// Placeholder for S3 implementation
// When the "s3" feature is enabled and aws-sdk-s3 is available,
// this module will provide full S3 storage support.

#[cfg(feature = "s3")]
compile_error!(
    "S3 storage requires aws-sdk-s3 dependency. \
     Please add aws-sdk-s3 to your Cargo.toml or disable the 's3' feature."
);
