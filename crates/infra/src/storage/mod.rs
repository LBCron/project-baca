//! S3-compatible object storage for EPUB files and cover images.
//!
//! Backed by `aws-sdk-s3` against MinIO (local), Cloudflare R2, or AWS S3.
//! Buckets are created idempotently at startup. All failures surface as
//! [`shared::AppError::Internal`] with sanitized messages (CWE-209).

pub mod s3;

pub use s3::StorageService;
