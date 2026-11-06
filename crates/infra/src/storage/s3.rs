//! MinIO/S3 storage service: EPUB upload bucket and cover image bucket.

use aws_credential_types::Credentials;
use aws_sdk_s3::{
    config::{http::HttpResponse, timeout::TimeoutConfig, Region},
    error::{DisplayErrorContext, SdkError},
    primitives::ByteStream,
    Client,
};
use shared::AppError;
use std::time::Duration;

use crate::StorageConfig;

/// S3-compatible storage service for raw EPUBs and cover images.
#[derive(Debug, Clone)]
pub struct StorageService {
    client: Client,
    bucket_epubs: String,
    bucket_covers: String,
}

/// Upper bound for one S3 call including the SDK's retries, so a store that
/// accepts connections but never answers cannot hold a request open.
pub const S3_OPERATION_TIMEOUT: Duration = Duration::from_secs(15);
const S3_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

impl StorageService {
    /// Builds the service from [`StorageConfig`], checking that both buckets
    /// answer with these credentials and creating one only when it is absent
    /// (idempotent; concurrent creates resolve to success or already-owned).
    pub async fn init(config: &StorageConfig) -> Result<Self, AppError> {
        let service = Self::connect(config);
        service.ensure_bucket(&service.bucket_epubs).await?;
        service.ensure_bucket(&service.bucket_covers).await?;
        Ok(service)
    }

    /// Builds the client without touching the store.
    pub fn connect(config: &StorageConfig) -> Self {
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
            .timeout_config(
                TimeoutConfig::builder()
                    .connect_timeout(S3_CONNECT_TIMEOUT)
                    .operation_timeout(S3_OPERATION_TIMEOUT)
                    .build(),
            )
            .behavior_version_latest()
            .build();

        Self {
            client: Client::from_conf(s3_config),
            bucket_epubs: config.bucket_epubs.clone(),
            bucket_covers: config.bucket_covers.clone(),
        }
    }

    async fn ensure_bucket(&self, bucket: &str) -> Result<(), AppError> {
        // An R2 Object Read and Write token may list objects but not create
        // buckets, and may be refused HeadBucket, so a one-key listing is the
        // probe and CreateBucket is sent only for a bucket that is missing.
        let probe = self
            .client
            .list_objects_v2()
            .bucket(bucket)
            .max_keys(1)
            .send()
            .await;
        let Err(e) = probe else {
            return self.ensure_writable(bucket).await;
        };
        if !e.as_service_error().is_some_and(|s| s.is_no_such_bucket()) {
            return Err(refused(bucket, "is not accessible", e));
        }
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

    /// A read-only token passes the listing above and fails only on the first
    /// upload, so boot writes and deletes one empty object as well.
    async fn ensure_writable(&self, bucket: &str) -> Result<(), AppError> {
        let key = format!(".boot-probe/{}", uuid::Uuid::new_v4());
        self.client
            .put_object()
            .bucket(bucket)
            .key(&key)
            .body(ByteStream::from_static(b""))
            .send()
            .await
            .map_err(|e| refused(bucket, "refuses writes", e))?;
        self.client
            .delete_object()
            .bucket(bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| refused(bucket, "refuses deletes", e))?;
        Ok(())
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

    /// Fetches a cover image; `NotFound` when the key is absent so the web
    /// route can answer 404.
    pub async fn get_cover(&self, key: &str) -> Result<Vec<u8>, AppError> {
        self.get_object(&self.bucket_covers.clone(), key).await
    }

    /// Best-effort delete from the EPUB bucket (orphan compensation when a
    /// later upload step fails; failures only surface as logs).
    pub async fn delete_epub(&self, key: &str) -> Result<(), AppError> {
        self.delete_object(&self.bucket_epubs.clone(), key).await
    }

    /// Removes a cover image (re-ingestion cleanup and test teardown).
    pub async fn delete_cover(&self, key: &str) -> Result<(), AppError> {
        self.delete_object(&self.bucket_covers.clone(), key).await
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), AppError> {
        self.client
            .delete_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Storage delete failed: {e:?}")))?;
        Ok(())
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
            .map_err(|e| {
                let service_error = e.into_service_error();
                if service_error.is_no_such_key() {
                    AppError::NotFound("Object not found".to_string())
                } else {
                    AppError::Internal(format!("Storage read failed: {service_error:?}"))
                }
            })?;
        output
            .body
            .collect()
            .await
            .map(|data| data.into_bytes().to_vec())
            .map_err(|e| AppError::Internal(format!("Storage body read failed: {e:?}")))
    }
}

/// Boot-probe failure naming the bucket, the HTTP status, and the store's
/// error code, without the raw response the SDK's Debug output appends.
fn refused<E>(bucket: &str, what: &str, e: SdkError<E, HttpResponse>) -> AppError
where
    E: std::error::Error + 'static,
{
    let status = e.raw_response().map_or_else(
        || "no response".to_string(),
        |r| r.status().as_u16().to_string(),
    );
    let detail = match e.as_service_error() {
        Some(service) => service.to_string(),
        None => DisplayErrorContext(&e).to_string(),
    };
    AppError::Internal(format!(
        "Storage bucket '{bucket}' {what} with the configured credentials (HTTP {status}): {detail}"
    ))
}

/// Live against the dev MinIO (`make db-up`).
#[cfg(test)]
mod tests {
    use super::*;

    fn config_with_buckets(tag: &str) -> StorageConfig {
        let id = uuid::Uuid::new_v4().simple().to_string();
        StorageConfig {
            bucket_epubs: format!("test-{tag}-e-{}", &id[..12]),
            bucket_covers: format!("test-{tag}-c-{}", &id[..12]),
            ..StorageConfig::default()
        }
    }

    async fn drop_buckets(service: &StorageService) {
        for bucket in [&service.bucket_epubs, &service.bucket_covers] {
            let _ = service.client.delete_bucket().bucket(bucket).send().await;
        }
    }

    #[tokio::test]
    async fn init_creates_missing_buckets_and_accepts_existing_ones() {
        let config = config_with_buckets("init");
        let service = StorageService::init(&config)
            .await
            .expect("missing buckets are created");
        let again = StorageService::init(&config).await;
        drop_buckets(&service).await;
        again.expect("existing buckets are accepted without a create");
    }

    /// Runs `mc` inside the dev MinIO container against its own root alias.
    fn minio_admin(command: &str) {
        let status = std::process::Command::new("docker")
            .args(["exec", "project_baca_minio", "sh", "-c"])
            .arg(format!(
                "mc alias set local http://localhost:9000 minioadmin minioadmin >/dev/null && {command}"
            ))
            .status()
            .expect("docker exec into the dev MinIO");
        assert!(status.success(), "mc failed: {command}");
    }

    #[tokio::test]
    async fn init_leaves_no_probe_object_behind() {
        let config = config_with_buckets("clean");
        let service = StorageService::init(&config).await.expect("init succeeds");
        let mut leftovers = 0;
        for bucket in [&service.bucket_epubs, &service.bucket_covers] {
            let listing = service.client.list_objects_v2().bucket(bucket).send().await;
            leftovers += listing.map_or(1, |l| l.key_count().unwrap_or(0));
        }
        drop_buckets(&service).await;
        assert_eq!(leftovers, 0);
    }

    #[tokio::test]
    async fn init_with_read_only_credentials_fails_on_the_write_probe() {
        // List and read only, like an R2 "Object Read" token; MinIO's built-in
        // `readonly` policy also refuses listing, so it would fail earlier.
        minio_admin(
            r#"printf '%s' '{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Action":["s3:ListBucket","s3:GetBucketLocation","s3:GetObject"],"Resource":["arn:aws:s3:::*"]}]}' > /tmp/list-read.json && \
             mc admin policy create local list-read /tmp/list-read.json >/dev/null && \
             mc admin user add local probe-readonly probe-readonly-secret >/dev/null && \
             { mc admin policy detach local list-read --user probe-readonly >/dev/null 2>&1; \
               mc admin policy attach local list-read --user probe-readonly >/dev/null; }"#,
        );
        let config = StorageConfig {
            access_key_id: "probe-readonly".to_string(),
            secret_access_key: "probe-readonly-secret".to_string(),
            ..StorageConfig::default()
        };
        let err = StorageService::init(&config)
            .await
            .expect_err("a token that can list but not write must not pass init");
        let msg = err.to_string();
        assert!(
            msg.contains(&format!(
                "Storage bucket '{}' refuses writes",
                config.bucket_epubs
            )) && msg.contains("HTTP 403"),
            "{msg}"
        );
    }

    #[tokio::test]
    async fn init_with_a_wrong_secret_fails_with_the_bucket_and_status() {
        let config = StorageConfig {
            secret_access_key: "not-the-secret".to_string(),
            ..StorageConfig::default()
        };
        let err = StorageService::init(&config)
            .await
            .expect_err("a wrong secret must not pass init");
        let msg = err.to_string();
        assert!(
            msg.contains(&format!("Storage bucket '{}'", config.bucket_epubs))
                && msg.contains("HTTP 403"),
            "{msg}"
        );
    }
}
