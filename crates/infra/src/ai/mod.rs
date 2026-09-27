//! Embedding provider trait and factory.

pub mod embedding;

pub use embedding::{
    build_embedding_provider, EmbeddingProvider, FastEmbedProvider, GeminiEmbeddingProvider,
};
