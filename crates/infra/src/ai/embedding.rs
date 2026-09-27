//! Dual-mode embeddings (Gemini REST + FastEmbed stub) behind one trait.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shared::AppError;
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::AiConfig;

// Core Trait

/// Async embedding provider contract. All implementations must be `Send + Sync`
/// so they can be wrapped in `Arc` and shared across Axum handlers.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Embed a single text string into a 768-dimensional vector.
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, AppError>;

    /// Batch-embed multiple text strings in a single API call where supported.
    /// Falls back to sequential `embed_text` calls if the backend does not natively
    /// support batching.
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError>;

    /// Returns the fixed output dimension for this provider (always 768 for Project Baca).
    fn dimension(&self) -> usize;
}

// Gemini REST API Provider

/// Request body for the Gemini Embeddings REST endpoint.
#[derive(Debug, Serialize)]
struct GeminiEmbedRequest<'a> {
    model: &'a str,
    content: GeminiContent<'a>,
    #[serde(
        rename = "outputDimensionality",
        skip_serializing_if = "Option::is_none"
    )]
    output_dimensionality: Option<usize>,
}

#[derive(Debug, Serialize)]
struct GeminiContent<'a> {
    parts: Vec<GeminiPart<'a>>,
}

#[derive(Debug, Serialize)]
struct GeminiPart<'a> {
    text: &'a str,
}

/// Batch request body for the Gemini `batchEmbedContents` endpoint.
#[derive(Debug, Serialize)]
struct GeminiBatchEmbedRequest<'a> {
    requests: Vec<GeminiEmbedRequest<'a>>,
}

/// Parsed response from `embedContent` endpoint.
#[derive(Debug, Deserialize)]
struct GeminiEmbedResponse {
    embedding: GeminiEmbedding,
}

#[derive(Debug, Deserialize)]
struct GeminiEmbedding {
    values: Vec<f32>,
}

/// Parsed response from `batchEmbedContents` endpoint.
#[derive(Debug, Deserialize)]
struct GeminiBatchEmbedResponse {
    embeddings: Vec<GeminiEmbedding>,
}

/// HTTP client adapter calling Google GenAI `gemini-embedding-2` or `text-embedding-004`.
///
/// Uses `reqwest` with `rustls-tls` (no OpenSSL dependency) and a persistent
/// connection pool to avoid TCP handshake overhead per request.
pub struct GeminiEmbeddingProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    dimension: usize,
}

impl GeminiEmbeddingProvider {
    /// Construct a new Gemini provider from an `AiConfig`.
    pub fn new(config: &AiConfig) -> Result<Self, AppError> {
        if config.api_key.is_empty() {
            return Err(AppError::Internal(
                "GEMINI_API_KEY (or AI_API_KEY) is not configured. Set GEMINI_API_KEY in your .env file.".to_string(),
            ));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("project-baca/0.1")
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to build reqwest client: {e}")))?;

        Ok(Self {
            client,
            api_key: config.api_key.clone(),
            model: config.model_name.clone(),
            dimension: config.dimension,
        })
    }

    fn embed_url(&self) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:embedContent?key={}",
            self.model, self.api_key
        )
    }

    fn batch_embed_url(&self) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:batchEmbedContents?key={}",
            self.model, self.api_key
        )
    }
}

#[async_trait]
impl EmbeddingProvider for GeminiEmbeddingProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, AppError> {
        debug!(
            text_len = text.len(),
            "Embedding single text via Gemini API"
        );

        let body = GeminiEmbedRequest {
            model: &format!("models/{}", self.model),
            content: GeminiContent {
                parts: vec![GeminiPart { text }],
            },
            output_dimensionality: Some(self.dimension),
        };

        let response = self
            .client
            .post(self.embed_url())
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Gemini API request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Gemini API returned HTTP {status}: {body_text}"
            )));
        }

        let parsed: GeminiEmbedResponse = response.json().await.map_err(|e| {
            AppError::Internal(format!("Failed to parse Gemini embed response: {e}"))
        })?;

        Ok(parsed.embedding.values)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        debug!(
            batch_size = texts.len(),
            "Batch-embedding texts via Gemini batchEmbedContents API"
        );

        let model_ref = format!("models/{}", self.model);
        let requests: Vec<GeminiEmbedRequest<'_>> = texts
            .iter()
            .map(|t| GeminiEmbedRequest {
                model: &model_ref,
                content: GeminiContent {
                    parts: vec![GeminiPart { text: t.as_str() }],
                },
                output_dimensionality: Some(self.dimension),
            })
            .collect();

        let body = GeminiBatchEmbedRequest { requests };

        let response = self
            .client
            .post(self.batch_embed_url())
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Gemini batch API request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Gemini batch API returned HTTP {status}: {body_text}"
            )));
        }

        let parsed: GeminiBatchEmbedResponse = response.json().await.map_err(|e| {
            AppError::Internal(format!("Failed to parse Gemini batch response: {e}"))
        })?;

        Ok(parsed.embeddings.into_iter().map(|e| e.values).collect())
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

// FastEmbed CPU Provider (ONNX Runtime)

/// CPU ONNX-backed embedding provider using the `fastembed` crate.
pub struct FastEmbedProvider {
    dimension: usize,
}

impl FastEmbedProvider {
    pub fn new(config: &AiConfig) -> Self {
        warn!("FastEmbedProvider initialized in stub mode; local ONNX embedding is not bundled.");
        Self {
            dimension: config.dimension,
        }
    }
}

#[async_trait]
impl EmbeddingProvider for FastEmbedProvider {
    async fn embed_text(&self, _text: &str) -> Result<Vec<f32>, AppError> {
        Err(AppError::Internal(
            "FastEmbedProvider is not bundled; configure EMBEDDING_PROVIDER=gemini.".to_string(),
        ))
    }

    async fn embed_batch(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>, AppError> {
        Err(AppError::Internal(
            "FastEmbedProvider batch embedding is not bundled; configure EMBEDDING_PROVIDER=gemini."
                .to_string(),
        ))
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

// Mock Provider (for CI & Integration Tests)

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
    async fn embed_text(&self, _text: &str) -> Result<Vec<f32>, AppError> {
        Ok(self.one_hot())
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError> {
        Ok((0..texts.len()).map(|_| self.one_hot()).collect())
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

// Factory

/// Constructs the correct `EmbeddingProvider` based on `AiConfig.provider`.
pub fn build_embedding_provider(config: &AiConfig) -> Result<Arc<dyn EmbeddingProvider>, AppError> {
    match config.provider.to_lowercase().as_str() {
        "gemini" => {
            info!(
                model = %config.model_name,
                dimension = config.dimension,
                "Initializing GeminiEmbeddingProvider"
            );
            let provider = GeminiEmbeddingProvider::new(config)?;
            Ok(Arc::new(provider))
        }
        "fastembed" => {
            info!(
                dimension = config.dimension,
                "Initializing FastEmbedProvider (stub mode)"
            );
            let provider = FastEmbedProvider::new(config);
            Ok(Arc::new(provider))
        }
        "mock" => {
            info!(
                dimension = config.dimension,
                "Initializing MockEmbeddingProvider (for testing)"
            );
            Ok(Arc::new(MockEmbeddingProvider::new(config.dimension)))
        }
        other => Err(AppError::Internal(format!(
            "Unknown AI provider: '{other}'. Valid values: 'gemini', 'fastembed', 'mock'."
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_provider_missing_key() {
        let config = AiConfig {
            provider: "gemini".to_string(),
            api_key: String::new(),
            model_name: "text-embedding-004".to_string(),
            dimension: 768,
        };
        assert!(GeminiEmbeddingProvider::new(&config).is_err());
    }

    #[test]
    fn test_fastembed_stub_properties() {
        let config = AiConfig {
            provider: "fastembed".to_string(),
            api_key: String::new(),
            model_name: "text-embedding-004".to_string(),
            dimension: 768,
        };
        let provider = FastEmbedProvider::new(&config);
        assert_eq!(provider.dimension(), 768);
    }

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockEmbeddingProvider::new(768);
        assert_eq!(provider.dimension(), 768);
        let res = provider.embed_text("test query").await;
        assert!(res.is_ok());
        if let Ok(vec) = res {
            assert_eq!(vec.len(), 768);
            // One-hot contract: exact-match anchor for seeded test chunks.
            assert_eq!(vec[0], 1.0);
            assert!(vec[1..].iter().all(|&x| x == 0.0));
        }
        let batch = provider.embed_batch(&["a".into(), "b".into()]).await;
        assert!(batch.is_ok());
        if let Ok(b) = batch {
            assert_eq!(b.len(), 2);
            assert_eq!(b[0].len(), 768);
        }
    }

    #[test]
    fn test_build_embedding_provider_unknown() {
        let config = AiConfig {
            provider: "unknown-ai".to_string(),
            api_key: String::new(),
            model_name: "text-embedding-004".to_string(),
            dimension: 768,
        };
        assert!(build_embedding_provider(&config).is_err());
    }

    #[test]
    fn test_build_embedding_provider_mock() {
        let config = AiConfig {
            provider: "mock".to_string(),
            api_key: String::new(),
            model_name: "text-embedding-004".to_string(),
            dimension: 768,
        };
        let provider = build_embedding_provider(&config);
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_gemini_embedding_live_if_key_available() {
        let Ok(key) = std::env::var("GEMINI_API_KEY").or_else(|_| std::env::var("AI_API_KEY"))
        else {
            return;
        };
        if key.is_empty() || key.contains("your-gemini-api-key") {
            return;
        }

        let config = AiConfig {
            provider: "gemini".to_string(),
            api_key: key,
            model_name: "gemini-embedding-2".to_string(),
            dimension: 768,
        };
        let provider = GeminiEmbeddingProvider::new(&config).expect("Must construct provider");
        let vec = provider
            .embed_text("Watson in London fog")
            .await
            .expect("Must embed text");
        assert_eq!(vec.len(), 768);
    }
}
