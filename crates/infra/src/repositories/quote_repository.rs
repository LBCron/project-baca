//! HNSW cosine-distance quote search over `book_chunks.embedding`.
//!
//! Raw SQL is used because SeaORM has no `<=>` operator support.

use sea_orm::{ConnectionTrait, DatabaseConnection, FromQueryResult, JsonValue, Statement};
use serde::{Deserialize, Serialize};
use shared::AppError;
use uuid::Uuid;

// Intermediate query result types (not exposed outside repository)

/// Raw row returned by the HNSW cosine-distance query.
#[derive(Debug, FromQueryResult)]
struct ChunkSearchRow {
    chunk_id: Uuid,
    chapter_number: i32,
    chapter_title: Option<String>,
    chunk_text: String,
    cosine_similarity: f64,
}

/// Raw row returned by the saved_quotes join query.
#[derive(Debug, FromQueryResult)]
struct SavedQuoteRow {
    id: Uuid,
    book_id: Uuid,
    chapter_id: Uuid,
    quote_text: String,
    image_card_url: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

// Public DTOs for this repository layer (also re-exported from server routes)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteSearchRow {
    pub chunk_id: Uuid,
    pub chapter_number: i32,
    pub chapter_title: Option<String>,
    pub cfi_range: Option<String>,
    pub content: String,
    pub similarity_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedQuoteDto {
    pub id: Uuid,
    pub book_id: Uuid,
    pub chapter_id: Uuid,
    pub quote_text: String,
    pub image_card_url: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// Scoped HNSW Semantic Quote Search

/// Execute a cosine-distance query against `book_chunks` scoped to a single book.
pub async fn search_quotes_by_embedding(
    db: &DatabaseConnection,
    book_id: Uuid,
    embedding: &[f32],
    limit: u64,
) -> Result<Vec<QuoteSearchRow>, AppError> {
    let capped_limit = limit.min(20) as i64;

    // Serialize the f32 vector to a pgvector literal string: '[0.1,0.2,...]'
    let vector_literal = format!(
        "[{}]",
        embedding
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    let sql = r#"
        SELECT
            c.id            AS chunk_id,
            ch.chapter_number,
            ch.title        AS chapter_title,
            c.chunk_text,
            (1.0 - (c.embedding <=> $1::vector))::float8 AS cosine_similarity
        FROM book_chunks c
        JOIN chapters ch ON ch.id = c.chapter_id
        WHERE c.book_id = $2
        ORDER BY c.embedding <=> $1::vector
        LIMIT $3
    "#;

    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        sql,
        [
            sea_orm::Value::from(vector_literal),
            sea_orm::Value::from(book_id),
            sea_orm::Value::from(capped_limit),
        ],
    );

    let rows = ChunkSearchRow::find_by_statement(stmt)
        .all(db)
        .await
        .map_err(|e| AppError::Internal(format!("HNSW quote search query failed: {e}")))?;

    Ok(rows
        .into_iter()
        .map(|r| QuoteSearchRow {
            chunk_id: r.chunk_id,
            chapter_number: r.chapter_number,
            chapter_title: r.chapter_title,
            cfi_range: None,
            content: r.chunk_text,
            similarity_score: r.cosine_similarity as f32,
        })
        .collect())
}

// Atomic Cards & Recap from tldr_cache

/// Retrieve precomputed atomic insight cards from `tldr_cache`.
pub async fn get_tldr_cache(
    db: &DatabaseConnection,
    book_id: Uuid,
    chapter_id: Uuid,
    recap_type: &str,
) -> Result<Option<JsonValue>, AppError> {
    let sql = r#"
        SELECT content_json
        FROM tldr_cache
        WHERE book_id = $1
          AND chapter_id = $2
          AND recap_type = $3
        LIMIT 1
    "#;

    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        sql,
        [
            sea_orm::Value::from(book_id),
            sea_orm::Value::from(chapter_id),
            sea_orm::Value::from(recap_type),
        ],
    );

    #[derive(Debug, FromQueryResult)]
    struct CacheRow {
        content_json: JsonValue,
    }

    let row = CacheRow::find_by_statement(stmt)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(format!("tldr_cache query failed: {e}")))?;

    Ok(row.map(|r| r.content_json))
}

/// Retrieve a spoiler-free chapter recap from `tldr_cache`.
pub async fn get_chapter_recap(
    db: &DatabaseConnection,
    book_id: Uuid,
    chapter_id: Uuid,
) -> Result<Option<JsonValue>, AppError> {
    get_tldr_cache(db, book_id, chapter_id, "chapter_recap").await
}

// Saved Quotes Management & Vintage Quote Card

/// Persist a user-selected quote to `saved_quotes`.
pub async fn save_quote(
    db: &DatabaseConnection,
    id: Uuid,
    user_id: Uuid,
    book_id: Uuid,
    chapter_id: Uuid,
    quote_text: &str,
    image_card_url: Option<&str>,
) -> Result<(), AppError> {
    let sql = r#"
        INSERT INTO saved_quotes (id, user_id, book_id, chapter_id, quote_text, image_card_url, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, NOW())
        ON CONFLICT DO NOTHING
    "#;

    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        sql,
        [
            sea_orm::Value::from(id),
            sea_orm::Value::from(user_id),
            sea_orm::Value::from(book_id),
            sea_orm::Value::from(chapter_id),
            sea_orm::Value::from(quote_text),
            sea_orm::Value::from(image_card_url),
        ],
    );

    db.execute(stmt)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to save quote: {e}")))?;

    Ok(())
}

/// Retrieve single saved quote by its ID, scoped to the owning user.
pub async fn get_saved_quote_by_id(
    db: &DatabaseConnection,
    id: Uuid,
    user_id: Uuid,
) -> Result<Option<SavedQuoteDto>, AppError> {
    let sql = r#"
        SELECT id, book_id, chapter_id, quote_text, image_card_url, created_at
        FROM saved_quotes
        WHERE id = $1 AND user_id = $2
    "#;

    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        sql,
        [sea_orm::Value::from(id), sea_orm::Value::from(user_id)],
    );

    let row = SavedQuoteRow::find_by_statement(stmt)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to find saved quote: {e}")))?;

    Ok(row.map(|r| SavedQuoteDto {
        id: r.id,
        book_id: r.book_id,
        chapter_id: r.chapter_id,
        quote_text: r.quote_text,
        image_card_url: r.image_card_url,
        created_at: r.created_at,
    }))
}

/// Retrieve all saved quotes for a user, ordered by most recent first.
pub async fn list_saved_quotes(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Vec<SavedQuoteDto>, AppError> {
    let sql = r#"
        SELECT id, book_id, chapter_id, quote_text, image_card_url, created_at
        FROM saved_quotes
        WHERE user_id = $1
        ORDER BY created_at DESC
    "#;

    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        sql,
        [sea_orm::Value::from(user_id)],
    );

    let rows = SavedQuoteRow::find_by_statement(stmt)
        .all(db)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to list saved quotes: {e}")))?;

    Ok(rows
        .into_iter()
        .map(|r| SavedQuoteDto {
            id: r.id,
            book_id: r.book_id,
            chapter_id: r.chapter_id,
            quote_text: r.quote_text,
            image_card_url: r.image_card_url,
            created_at: r.created_at,
        })
        .collect())
}
