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

/// Quotes one account may keep; storage per account stays bounded however
/// a script drives the save endpoints.
pub const MAX_SAVED_QUOTES_PER_USER: i64 = 5_000;

fn quota_exceeded() -> AppError {
    AppError::QuotaExceeded(format!(
        "An account keeps at most {MAX_SAVED_QUOTES_PER_USER} quotes"
    ))
}

/// Opens a transaction holding the user's row lock and returns it with the
/// number of quotes the user keeps. The lock serialises one user's saves,
/// so two parallel saves cannot both pass the quota check.
async fn lock_user_quotes(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<(sea_orm::DatabaseTransaction, i64), AppError> {
    use sea_orm::TransactionTrait;

    #[derive(Debug, FromQueryResult)]
    struct CountRow {
        kept: i64,
    }

    let txn = db
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to open quote transaction: {e}")))?;
    let lock = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT id FROM users WHERE id = $1 FOR UPDATE",
        [sea_orm::Value::from(user_id)],
    );
    txn.execute(lock)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to lock user for quotes: {e}")))?;
    let count = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT COUNT(*) AS kept FROM saved_quotes WHERE user_id = $1",
        [sea_orm::Value::from(user_id)],
    );
    let kept = CountRow::find_by_statement(count)
        .one(&txn)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to count saved quotes: {e}")))?
        .map_or(0, |row| row.kept);
    Ok((txn, kept))
}

/// Persists a user-selected quote to `saved_quotes`, or reports the identical
/// quote the user already keeps for that chapter so a retried save is
/// idempotent. A new quote past [`MAX_SAVED_QUOTES_PER_USER`] is refused.
pub async fn save_quote(
    db: &DatabaseConnection,
    id: Uuid,
    user_id: Uuid,
    book_id: Uuid,
    chapter_id: Uuid,
    quote_text: &str,
    image_card_url: Option<&str>,
) -> Result<QuoteSaveOutcome, AppError> {
    let (txn, kept) = lock_user_quotes(db, user_id).await?;
    if let Some(existing) =
        find_saved_quote_duplicate(&txn, user_id, book_id, chapter_id, quote_text).await?
    {
        return Ok(QuoteSaveOutcome::Duplicate(existing));
    }
    if kept >= MAX_SAVED_QUOTES_PER_USER {
        return Err(quota_exceeded());
    }
    insert_quote(
        &txn,
        id,
        user_id,
        book_id,
        chapter_id,
        quote_text,
        image_card_url,
    )
    .await?;
    txn.commit()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to commit quote: {e}")))?;
    Ok(QuoteSaveOutcome::Saved(id))
}

/// One insert statement shared by the single save and the batch, so both
/// write the same row through whatever connection they hold.
async fn insert_quote<C: ConnectionTrait>(
    conn: &C,
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

    conn.execute(stmt)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to save quote: {e}")))?;

    Ok(())
}

/// An identical quote the user already keeps for this chapter, if any.
async fn find_saved_quote_duplicate<C: ConnectionTrait>(
    conn: &C,
    user_id: Uuid,
    book_id: Uuid,
    chapter_id: Uuid,
    quote_text: &str,
) -> Result<Option<Uuid>, AppError> {
    #[derive(Debug, FromQueryResult)]
    struct IdRow {
        id: Uuid,
    }

    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT id FROM saved_quotes WHERE user_id = $1 AND book_id = $2 AND chapter_id = $3 AND quote_text = $4 LIMIT 1",
        [
            sea_orm::Value::from(user_id),
            sea_orm::Value::from(book_id),
            sea_orm::Value::from(chapter_id),
            sea_orm::Value::from(quote_text),
        ],
    );
    IdRow::find_by_statement(stmt)
        .one(conn)
        .await
        .map(|row| row.map(|r| r.id))
        .map_err(|e| AppError::Internal(format!("Failed to check saved quote: {e}")))
}

/// What happened to one quote: written now, or already on the shelf.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteSaveOutcome {
    Saved(Uuid),
    Duplicate(Uuid),
}

/// Saves several already-validated quotes in one transaction. An identical
/// quote on the shelf is reported, not duplicated, so a merge can be replayed.
/// A batch that would carry the account past its quota is refused whole, so
/// a device merging its quotes keeps every one it could not hand over.
pub async fn save_quotes_batch(
    db: &DatabaseConnection,
    user_id: Uuid,
    items: &[(Uuid, Uuid, String)],
    card_url: impl Fn(Uuid) -> String,
) -> Result<Vec<QuoteSaveOutcome>, AppError> {
    let (txn, mut kept) = lock_user_quotes(db, user_id).await?;
    let mut outcomes = Vec::with_capacity(items.len());
    for (book_id, chapter_id, text) in items {
        if let Some(existing) =
            find_saved_quote_duplicate(&txn, user_id, *book_id, *chapter_id, text).await?
        {
            outcomes.push(QuoteSaveOutcome::Duplicate(existing));
            continue;
        }
        if kept >= MAX_SAVED_QUOTES_PER_USER {
            return Err(quota_exceeded());
        }
        kept += 1;
        let id = Uuid::new_v4();
        insert_quote(
            &txn,
            id,
            user_id,
            *book_id,
            *chapter_id,
            text,
            Some(&card_url(id)),
        )
        .await?;
        outcomes.push(QuoteSaveOutcome::Saved(id));
    }
    txn.commit()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to commit quote batch: {e}")))?;
    Ok(outcomes)
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

/// One page of a user's saved quotes, newest first, after the quote
/// `cursor` (keyset on `created_at, id`). A cursor that is not one of the
/// user's quotes yields an empty page rather than someone else's order.
pub async fn list_saved_quotes(
    db: &DatabaseConnection,
    user_id: Uuid,
    cursor: Option<Uuid>,
    limit: u64,
) -> Result<Vec<SavedQuoteDto>, AppError> {
    let sql = r#"
        SELECT id, book_id, chapter_id, quote_text, image_card_url, created_at
        FROM saved_quotes
        WHERE user_id = $1
          AND ($2::uuid IS NULL OR (created_at, id) < (
              SELECT created_at, id FROM saved_quotes WHERE id = $2 AND user_id = $1
          ))
        ORDER BY created_at DESC, id DESC
        LIMIT $3
    "#;

    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        sql,
        [
            sea_orm::Value::from(user_id),
            sea_orm::Value::from(cursor),
            sea_orm::Value::from(limit as i64),
        ],
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
