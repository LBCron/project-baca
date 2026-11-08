//! Re-embeds stored `book_chunks` with the configured embedding service, in
//! place: the rows, their text and anchors stay, only `embedding` changes, so
//! a rerun is harmless and the row count never moves.
//!
//! Usage: `make reembed-chunks` (every book) or
//! `make reembed-chunks BOOK=<uuid>` (one book).

use infra::{build_embedding_provider, entities::book_chunks, init_db_pool, AppConfig};
use sea_orm::entity::prelude::PgVector;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use std::sync::Arc;
use uuid::Uuid;

/// Chunks read, embedded, and written per round.
const BATCH_SIZE: u64 = 32;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let book = match std::env::args().nth(1).filter(|a| !a.is_empty()) {
        Some(raw) => Some(Uuid::parse_str(&raw).map_err(|e| format!("BOOK '{raw}': {e}"))?),
        None => None,
    };
    let config = Arc::new(AppConfig::from_env()?);
    let db = init_db_pool(&config).await?;
    let provider = build_embedding_provider(&config.ai)?;

    let scope = || {
        let query = book_chunks::Entity::find();
        match book {
            Some(id) => query.filter(book_chunks::Column::BookId.eq(id)),
            None => query,
        }
    };
    let before = scope().count(&db).await?;
    println!("chunks to re-embed: {before}");

    // Keyset over the primary key, so updated rows are never read twice.
    let mut last: Option<Uuid> = None;
    let mut done = 0u64;
    loop {
        let mut page = scope()
            .order_by_asc(book_chunks::Column::Id)
            .limit(BATCH_SIZE);
        if let Some(id) = last {
            page = page.filter(book_chunks::Column::Id.gt(id));
        }
        let rows = page.all(&db).await?;
        let Some(tail) = rows.last() else { break };
        last = Some(tail.id);

        let texts: Vec<String> = rows.iter().map(|r| r.chunk_text.clone()).collect();
        let vectors = provider.embed_passages(&texts).await?;
        for (row, vector) in rows.into_iter().zip(vectors) {
            let mut active: book_chunks::ActiveModel = row.into();
            active.embedding = Set(PgVector::from(vector));
            active.update(&db).await?;
        }
        done += texts.len() as u64;
        println!("re-embedded {done}/{before}");
    }

    let after = scope().count(&db).await?;
    println!("done: {done} chunks re-embedded; rows before {before}, after {after}");
    if after != before {
        // Only an ingestion running meanwhile adds or replaces rows; this run never does.
        println!("note: an ingestion changed the row count during the run");
    }
    Ok(())
}
