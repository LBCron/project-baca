//! Embeddings from the self-hosted model service, behind one trait.

use async_trait::async_trait;
use serde::Serialize;
use shared::AppError;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use crate::AiConfig;

/// Async embedding provider contract. All implementations must be `Send + Sync`
/// so they can be wrapped in `Arc` and shared across Axum handlers.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Embeds a reader's search query.
    async fn embed_query(&self, text: &str) -> Result<Vec<f32>, AppError>;

    /// Embeds book passages (chunks), one vector per text, in order.
    async fn embed_passages(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError>;

    /// The fixed output dimension (768, the width of `book_chunks.embedding`).
    fn dimension(&self) -> usize;
}

/// multilingual-e5 was trained with these prefixes; a query embedded without
/// its prefix ranks passages noticeably worse. The worker uses the same two.
pub const QUERY_PREFIX: &str = "query: ";
pub const PASSAGE_PREFIX: &str = "passage: ";

/// A reader waits this long at most; past it the search answers 503.
const QUERY_TIMEOUT: Duration = Duration::from_secs(15);
/// One 400-word passage takes about 2 s on one CPU.
const PASSAGE_TIMEOUT: Duration = Duration::from_secs(120);
/// Seconds a client is told to wait while the model service loads or restarts.
const NOT_READY_RETRY_S: u64 = 10;

#[derive(Serialize)]
struct EmbedRequest<'a> {
    inputs: &'a [String],
    normalize: bool,
    truncate: bool,
}

/// Client for the Hugging Face text-embeddings-inference service running
/// next to the server (`embedder` in compose). Nothing leaves the host.
pub struct TeiEmbeddingProvider {
    client: reqwest::Client,
    embed_url: String,
    dimension: usize,
}

impl TeiEmbeddingProvider {
    pub fn new(config: &AiConfig) -> Result<Self, AppError> {
        let base = config.url.trim_end_matches('/');
        if !(base.starts_with("http://") || base.starts_with("https://")) {
            return Err(AppError::ValidationError(format!(
                "EMBEDDING_URL '{base}' must be an http(s) URL"
            )));
        }
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .user_agent("project-baca/0.1")
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to build reqwest client: {e}")))?;
        Ok(Self {
            client,
            embed_url: format!("{base}/embed"),
            dimension: config.dimension,
        })
    }

    async fn embed(&self, inputs: &[String], timeout: Duration) -> Result<Vec<Vec<f32>>, AppError> {
        let body = EmbedRequest {
            inputs,
            normalize: true,
            truncate: true,
        };
        let response = self
            .client
            .post(&self.embed_url)
            .timeout(timeout)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                // Refused or timed out: the service is restarting or still loading.
                tracing::warn!(target: "infra::embedding", error = %e, "embedding service unreachable");
                AppError::AiUnavailable {
                    retry_after: NOT_READY_RETRY_S,
                }
            })?;
        let status = response.status();
        // 429: TEI's queue is full; 503 and 424: the model is not loaded.
        if matches!(status.as_u16(), 429 | 503 | 424) {
            tracing::warn!(target: "infra::embedding", %status, "embedding service not ready");
            return Err(AppError::AiUnavailable {
                retry_after: NOT_READY_RETRY_S,
            });
        }
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Embedding service returned HTTP {status}: {}",
                text.chars().take(200).collect::<String>()
            )));
        }
        let vectors: Vec<Vec<f32>> = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse embedding response: {e}")))?;
        if vectors.len() != inputs.len() {
            return Err(AppError::Internal(format!(
                "Embedding service returned {} vectors for {} inputs",
                vectors.len(),
                inputs.len()
            )));
        }
        if let Some(bad) = vectors.iter().find(|v| v.len() != self.dimension) {
            return Err(AppError::Internal(format!(
                "Embedding dimension {} != {}",
                bad.len(),
                self.dimension
            )));
        }
        Ok(vectors)
    }
}

#[async_trait]
impl EmbeddingProvider for TeiEmbeddingProvider {
    async fn embed_query(&self, text: &str) -> Result<Vec<f32>, AppError> {
        let mut vectors = self
            .embed(&[format!("{QUERY_PREFIX}{text}")], QUERY_TIMEOUT)
            .await?;
        vectors
            .pop()
            .ok_or_else(|| AppError::Internal("Embedding service returned nothing".to_string()))
    }

    async fn embed_passages(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError> {
        // One passage per request: the service queues requests in order, so
        // a reader's query waits behind one passage (about 2 s), not a batch.
        let mut vectors = Vec::with_capacity(texts.len());
        for text in texts {
            vectors.extend(
                self.embed(&[format!("{PASSAGE_PREFIX}{text}")], PASSAGE_TIMEOUT)
                    .await?,
            );
        }
        Ok(vectors)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Deterministic mock embedding provider for tests and CI.
/// Returns a one-hot unit vector (index 0 = 1.0, rest 0.0) without outbound
/// network calls, so seeded test chunks with known embeddings produce stable,
/// rank-meaningful cosine similarities (exact match = 1.0, orthogonal = 0.0).
pub struct MockEmbeddingProvider {
    dimension: usize,
}

impl MockEmbeddingProvider {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }

    /// Deterministic one-hot unit vector: index 0 is 1.0, everything else 0.0.
    /// A zero query vector would make pgvector cosine distance degenerate
    /// (zero norm), so tests seed chunks against this exact vector instead.
    fn one_hot(&self) -> Vec<f32> {
        let mut v = vec![0.0f32; self.dimension];
        if !v.is_empty() {
            v[0] = 1.0;
        }
        v
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn embed_query(&self, _text: &str) -> Result<Vec<f32>, AppError> {
        Ok(self.one_hot())
    }

    async fn embed_passages(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError> {
        Ok((0..texts.len()).map(|_| self.one_hot()).collect())
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Constructs the `EmbeddingProvider` named by `AiConfig.provider`.
pub fn build_embedding_provider(config: &AiConfig) -> Result<Arc<dyn EmbeddingProvider>, AppError> {
    match config.provider.to_lowercase().as_str() {
        "tei" => {
            info!(url = %config.url, dimension = config.dimension, "Using the self-hosted embedding service");
            Ok(Arc::new(TeiEmbeddingProvider::new(config)?))
        }
        "mock" => Ok(Arc::new(MockEmbeddingProvider::new(config.dimension))),
        other => Err(AppError::ValidationError(format!(
            "Unknown EMBEDDING_PROVIDER '{other}'. Valid values: 'tei', 'mock'."
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn config(url: String) -> AiConfig {
        AiConfig {
            provider: "tei".to_string(),
            url,
            dimension: 4,
            daily_budget: 0,
        }
    }

    /// A stand-in for text-embeddings-inference: answers every POST with
    /// `status` and one `dim`-wide vector per input, and keeps each body.
    struct FakeTei {
        url: String,
        calls: Arc<AtomicUsize>,
        bodies: Arc<Mutex<Vec<serde_json::Value>>>,
    }

    async fn fake_tei(status: u16, dim: usize) -> FakeTei {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let url = format!("http://{}", listener.local_addr().expect("addr"));
        let calls = Arc::new(AtomicUsize::new(0));
        let bodies = Arc::new(Mutex::new(Vec::new()));
        let (seen, kept) = (calls.clone(), bodies.clone());
        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let mut raw = Vec::new();
                let mut buf = [0u8; 4096];
                let body = loop {
                    let n = socket.read(&mut buf).await.unwrap_or(0);
                    if n == 0 {
                        break None;
                    }
                    raw.extend_from_slice(&buf[..n]);
                    let text = String::from_utf8_lossy(&raw).to_string();
                    if let Some((head, rest)) = text.split_once("\r\n\r\n") {
                        let len = head
                            .lines()
                            .find_map(|l| {
                                l.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .map(|v| v.trim().parse::<usize>().unwrap_or(0))
                            })
                            .unwrap_or(0);
                        if rest.len() >= len {
                            break Some(rest[..len].to_string());
                        }
                    }
                };
                let Some(body) = body else { continue };
                seen.fetch_add(1, Ordering::SeqCst);
                let json: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
                let count = json["inputs"].as_array().map_or(0, Vec::len);
                kept.lock().expect("lock").push(json);
                let vectors = vec![vec![0.5f32; dim]; count];
                let payload = serde_json::to_string(&vectors).expect("json");
                let reply = format!(
                    "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{payload}",
                    payload.len()
                );
                let _ = socket.write_all(reply.as_bytes()).await;
            }
        });
        FakeTei { url, calls, bodies }
    }

    #[tokio::test]
    async fn queries_and_passages_carry_their_e5_prefix() {
        let fake = fake_tei(200, 4).await;
        let provider = TeiEmbeddingProvider::new(&config(fake.url.clone())).expect("provider");

        assert_eq!(
            provider
                .embed_query("kabut London")
                .await
                .expect("query")
                .len(),
            4
        );
        let passages = provider
            .embed_passages(&["satu".into(), "dua".into()])
            .await
            .expect("passages");
        assert_eq!(passages.len(), 2);

        assert_eq!(fake.calls.load(Ordering::SeqCst), 3);
        let bodies = fake.bodies.lock().expect("lock");
        assert_eq!(bodies[0]["inputs"][0], "query: kabut London");
        assert_eq!(bodies[2]["inputs"][0], "passage: dua");
        assert_eq!(bodies[0]["normalize"], true);
        assert_eq!(bodies[0]["truncate"], true);
    }

    #[tokio::test]
    async fn passages_go_one_per_request_so_queries_never_wait_behind_a_batch() {
        let fake = fake_tei(200, 4).await;
        let provider = TeiEmbeddingProvider::new(&config(fake.url.clone())).expect("provider");
        let texts: Vec<String> = (0..5).map(|i| format!("chunk {i}")).collect();

        let vectors = provider.embed_passages(&texts).await.expect("passages");

        assert_eq!(vectors.len(), 5);
        assert_eq!(fake.calls.load(Ordering::SeqCst), 5);
        let bodies = fake.bodies.lock().expect("lock");
        assert!(bodies
            .iter()
            .all(|b| b["inputs"].as_array().map(Vec::len) == Some(1)));
    }

    #[tokio::test]
    async fn a_loading_or_busy_service_is_not_ready() {
        for status in [503, 429] {
            let fake = fake_tei(status, 4).await;
            let provider = TeiEmbeddingProvider::new(&config(fake.url.clone())).expect("provider");
            let err = provider.embed_query("q").await.expect_err("not ready");
            assert!(
                matches!(err, AppError::AiUnavailable { .. }),
                "{status}: {err:?}"
            );
        }
    }

    #[tokio::test]
    async fn a_service_that_is_down_is_not_ready() {
        // Bind and drop: the port is free, so the connect is refused.
        let port = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind")
            .local_addr()
            .expect("addr")
            .port();
        let provider = TeiEmbeddingProvider::new(&config(format!("http://127.0.0.1:{port}")))
            .expect("provider");
        let err = provider.embed_query("q").await.expect_err("down");
        assert!(matches!(err, AppError::AiUnavailable { .. }), "{err:?}");
    }

    #[tokio::test]
    async fn a_wrong_dimension_is_an_internal_error() {
        let fake = fake_tei(200, 3).await;
        let provider = TeiEmbeddingProvider::new(&config(fake.url.clone())).expect("provider");
        let err = provider.embed_query("q").await.expect_err("wrong width");
        assert!(matches!(err, AppError::Internal(_)), "{err:?}");
    }

    #[tokio::test]
    async fn mock_provider_is_one_hot() {
        let provider = MockEmbeddingProvider::new(768);
        let vec = provider.embed_query("test query").await.expect("mock");
        assert_eq!(vec.len(), 768);
        assert_eq!(vec[0], 1.0);
        assert!(vec[1..].iter().all(|&x| x == 0.0));
        let batch = provider
            .embed_passages(&["a".into(), "b".into()])
            .await
            .expect("mock");
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn factory_knows_tei_and_mock_only() {
        let mut cfg = config("http://127.0.0.1:8090".to_string());
        assert!(build_embedding_provider(&cfg).is_ok());
        cfg.provider = "mock".to_string();
        assert!(build_embedding_provider(&cfg).is_ok());
        for retired in ["openai", "fastembed"] {
            cfg.provider = retired.to_string();
            assert!(build_embedding_provider(&cfg).is_err(), "{retired}");
        }
        cfg.provider = "tei".to_string();
        cfg.url = "127.0.0.1:8090".to_string();
        assert!(
            build_embedding_provider(&cfg).is_err(),
            "URL without a scheme"
        );
    }
}
