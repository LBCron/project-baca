//! AI subsystem module for Project Baca.
//! Exports the dual-mode embedding provider trait and factory.

pub mod embedding;

pub use embedding::{
    build_embedding_provider, EmbeddingProvider, FastEmbedProvider, GeminiEmbeddingProvider,
};
