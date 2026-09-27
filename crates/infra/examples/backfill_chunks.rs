//! One-shot dev backfill: chunk live `chapters` and embed into `book_chunks`.
//!
//! Usage: `cargo run -p infra --example backfill_chunks`
//! Uses the configured embedding provider (`EMBEDDING_PROVIDER`, default
//! `gemini` via `GEMINI_API_KEY`).

use infra::{build_embedding_provider, entities::book_chunks, init_db_pool, AppConfig};
use sea_orm::entity::prelude::PgVector;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use uuid::Uuid;

/// Target chunk size in words.
const CHUNK_WORDS: usize = 350;
/// Minimum stripped characters for a chapter to be worth embedding.
const MIN_CHARS: usize = 20;
/// Embedding batch size per provider call.
const BATCH_SIZE: usize = 25;

/// Strips HTML tags with a tiny state machine (no extra dependencies).
fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Splits plain text into ~CHUNK_WORDS-word pieces at whitespace boundaries.
fn chunk_text(text: &str) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return Vec::new();
    }
    words
        .chunks(CHUNK_WORDS)
        .map(|c| c.join(" "))
        .filter(|c| c.len() >= MIN_CHARS)
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(AppConfig::from_env()?);
    let db = init_db_pool(&config).await?;
    let provider = build_embedding_provider(&config.ai)?;

    let chapters = infra::entities::chapters::Entity::find().all(&db).await?;
    println!("chapters scanned: {}", chapters.len());

    // Collect (book_id, chapter_id, chunk_index, text) for chapters lacking chunks.
    let mut pending: Vec<(Uuid, Uuid, i32, String)> = Vec::new();
    let mut skipped = 0u64;
    for ch in &chapters {
        let existing = book_chunks::Entity::find()
            .filter(book_chunks::Column::ChapterId.eq(ch.id))
            .all(&db)
            .await?
            .len();
        if existing > 0 {
            skipped += 1;
            continue;
        }
        let plain = strip_html(&ch.html_content);
        for (idx, piece) in chunk_text(&plain).into_iter().enumerate() {
            pending.push((ch.book_id, ch.id, idx as i32, piece));
        }
    }
    println!("chapters already embedded: {skipped}");
    println!("chunks to embed: {}", pending.len());

    if pending.is_empty() {
        println!("nothing to do.");
        return Ok(());
    }

    let mut inserted = 0u64;
    for batch in pending.chunks(BATCH_SIZE) {
        let texts: Vec<String> = batch.iter().map(|p| p.3.clone()).collect();
        let vectors = provider.embed_batch(&texts).await?;
        for ((book_id, chapter_id, chunk_index, text), vector) in
            batch.iter().zip(vectors.into_iter())
        {
            let model = book_chunks::ActiveModel {
                id: Set(Uuid::new_v4()),
                book_id: Set(*book_id),
                chapter_id: Set(*chapter_id),
                chunk_index: Set(*chunk_index),
                chunk_text: Set(text.clone()),
                embedding: Set(PgVector::from(vector)),
                created_at: Set(chrono::Utc::now().into()),
            };
            model.insert(&db).await?;
            inserted += 1;
        }
        println!("inserted {inserted}/{} ...", pending.len());
    }

    println!("backfill complete: {inserted} chunks embedded.");
    Ok(())
}
