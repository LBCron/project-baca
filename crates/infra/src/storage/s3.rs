//! MinIO/S3 storage service: EPUB upload bucket and cover image bucket.

use aws_credential_types::Credentials;
use aws_sdk_s3::{config::Region, primitives::ByteStream, Client};
use shared::AppError;

use crate::StorageConfig;

/// S3-compatible storage service for raw EPUBs and cover images.
#[derive(Debug, Clone)]
pub struct StorageService {
    client: Client,
    bucket_epubs: String,
    bucket_covers: String,
}

impl StorageService {
    /// Builds the service from [`StorageConfig`], creating both buckets when
    /// absent (idempotent; concurrent creates resolve to success or
    /// already-owned).
    pub async fn init(config: &StorageConfig) -> Result<Self, AppError> {
        let credentials = Credentials::new(
            config.access_key_id.clone(),
            config.secret_access_key.clone(),
            None,
            None,
            "project-baca-static",
        );
        let s3_config = aws_sdk_s3::config::Builder::new()
            .endpoint_url(config.endpoint.clone())
            .credentials_provider(credentials)
            .region(Region::new(config.region.clone()))
            .force_path_style(config.force_path_style)
            .behavior_version_latest()
            .build();
        let client = Client::from_conf(s3_config);

        let service = Self {
            client,
            bucket_epubs: config.bucket_epubs.clone(),
            bucket_covers: config.bucket_covers.clone(),
        };
        service.ensure_bucket(&service.bucket_epubs).await?;
        service.ensure_bucket(&service.bucket_covers).await?;
        Ok(service)
    }

    async fn ensure_bucket(&self, bucket: &str) -> Result<(), AppError> {
        match self.client.create_bucket().bucket(bucket).send().await {
            Ok(_) => Ok(()),
            Err(e) => {
                // Idempotent: a bucket owned by us (or a concurrent creator)
                // is not an error. Anything else fails fast with context.
                let msg = format!("{e:?}");
                if msg.contains("BucketAlreadyOwnedByYou") || msg.contains("BucketAlreadyExists") {
                    Ok(())
                } else {
                    Err(AppError::Internal(format!(
                        "Failed to ensure storage bucket '{bucket}': {msg}"
                    )))
                }
            }
        }
    }

    /// Stores raw bytes under `key` in the EPUB bucket (admin upload path).
    pub async fn put_epub(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<(), AppError> {
        self.put_object(&self.bucket_epubs.clone(), key, bytes, content_type)
            .await
    }

    /// Stores raw bytes under `key` in the covers bucket (worker path).
    pub async fn put_cover(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<(), AppError> {
        self.put_object(&self.bucket_covers.clone(), key, bytes, content_type)
            .await
    }

    /// Fetches raw bytes for `key` from the EPUB bucket (worker path).
    pub async fn get_epub(&self, key: &str) -> Result<Vec<u8>, AppError> {
        self.get_object(&self.bucket_epubs.clone(), key).await
    }

    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<(), AppError> {
        self.client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(ByteStream::from(bytes))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Storage write failed: {e:?}")))?;
        Ok(())
    }

    async fn get_object(&self, bucket: &str, key: &str) -> Result<Vec<u8>, AppError> {
        let output = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Storage read failed: {e:?}")))?;
        output
            .body
            .collect()
            .await
            .map(|data| data.into_bytes().to_vec())
            .map_err(|e| AppError::Internal(format!("Storage body read failed: {e:?}")))
    }
}
