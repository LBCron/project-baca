//! S3-compatible object storage (MinIO/R2/S3) with idempotent buckets.

pub mod s3;

pub use s3::StorageService;
