//! Catalog, search, chapter, and bundle queries.

use sea_orm::{
    ColumnTrait, DatabaseBackend, DatabaseConnection, EntityTrait, FromQueryResult, QueryFilter,
    QueryOrder, QuerySelect, Statement, Value,
};
use shared::{
    AdminBookRowDto, AppError, BookCatalogQuery, BookDetailDto, BookSearchQuery,
    BookSearchResultDto, BookSummaryDto, ChapterDetailDto, ChapterSummaryDto, OfflineBundleDto,
};
use uuid::Uuid;

use crate::entities::{book_tags, books, chapters, tags};

#[derive(Debug, FromQueryResult)]
struct SearchResultRow {
    id: Uuid,
    title: String,
    author: String,
    language: String,
    primary_theme: String,
    cover_url: String,
    estimated_reading_minutes: i32,
    score: f32,
}

/// Lists published books with cursor pagination and taxonomy filters.
pub async fn list_books(
    db: &DatabaseConnection,
    query: &BookCatalogQuery,
) -> Result<Vec<BookSummaryDto>, AppError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);

    let mut select = books::Entity::find().filter(books::Column::Status.eq("published"));

    if let Some(ref lang) = query.language {
        select = select.filter(books::Column::Language.eq(lang));
    }

    if let Some(ref theme) = query.theme {
        select = select.filter(books::Column::PrimaryTheme.eq(theme));
    }

    // Cursor pagination based on ID or publication year
    if let Some(cursor_id) = query.cursor {
        if let Some(cursor_book) = books::Entity::find_by_id(cursor_id)
            .one(db)
            .await
            .map_err(|e| AppError::Database(format!("Failed to retrieve cursor book: {e}")))?
        {
            if let Some(cursor_year) = cursor_book.publication_year {
                select = select.filter(
                    books::Column::PublicationYear.lt(cursor_year).or(
                        books::Column::PublicationYear
                            .eq(cursor_year)
                            .and(books::Column::Id.gt(cursor_id)),
                    ),
                );
            } else {
                select = select.filter(books::Column::Id.gt(cursor_id));
            }
        }
    }

    // Tag filter
    if let Some(ref tag_query) = query.tag {
        let matching_tags = tags::Entity::find()
            .filter(
                tags::Column::Slug
                    .eq(tag_query)
                    .or(tags::Column::Name.eq(tag_query)),
            )
            .all(db)
            .await
            .map_err(|e| AppError::Database(format!("Failed to query tags: {e}")))?;

        let tag_ids: Vec<Uuid> = matching_tags.into_iter().map(|t| t.id).collect();
        if tag_ids.is_empty() {
            return Ok(Vec::new());
        }

        let book_tags = book_tags::Entity::find()
            .filter(book_tags::Column::TagId.is_in(tag_ids))
            .all(db)
            .await
            .map_err(|e| AppError::Database(format!("Failed to query book tags: {e}")))?;

        let book_ids: Vec<Uuid> = book_tags.into_iter().map(|bt| bt.book_id).collect();
        if book_ids.is_empty() {
            return Ok(Vec::new());
        }

        select = select.filter(books::Column::Id.is_in(book_ids));
    }

    select = select
        .order_by_desc(books::Column::PublicationYear)
        .order_by_asc(books::Column::Id)
        .limit(limit);

    let book_models = select
        .all(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query catalog: {e}")))?;

    // One batched tag lookup for the whole page (no N+1): a single join over
    // book_tags+tags filtered to the page's book ids.
    let page_ids: Vec<Uuid> = book_models.iter().map(|b| b.id).collect();
    let tag_map: std::collections::HashMap<Uuid, Vec<String>> = if page_ids.is_empty() {
        std::collections::HashMap::new()
    } else {
        let tag_rows = book_tags::Entity::find()
            .filter(book_tags::Column::BookId.is_in(page_ids))
            .find_also_related(tags::Entity)
            .all(db)
            .await
            .map_err(|e| AppError::Database(format!("Failed to query catalog tags: {e}")))?;
        let mut map: std::collections::HashMap<Uuid, Vec<String>> =
            std::collections::HashMap::new();
        for (bt, maybe_tag) in tag_rows {
            if let Some(t) = maybe_tag {
                map.entry(bt.book_id).or_default().push(t.name);
            }
        }
        map
    };

    let mut result = Vec::with_capacity(book_models.len());
    for b in book_models {
        let tag_names = tag_map.get(&b.id).cloned().unwrap_or_default();
        result.push(BookSummaryDto {
            id: b.id,
            title: b.title,
            author: b.author,
            language: b.language,
            primary_theme: b.primary_theme,
            sub_theme: b.sub_theme,
            description: b.description,
            cover_url: b.cover_url,
            total_words: b.total_words,
            estimated_reading_minutes: b.estimated_reading_minutes,
            publication_year: b.publication_year,
            license: b.license,
            tags: tag_names,
        });
    }

    Ok(result)
}

/// Typo-tolerant lexical search via PostgreSQL 17 GIN Trigram and FTS indexes.
pub async fn search_books(
    db: &DatabaseConnection,
    search: &BookSearchQuery,
) -> Result<Vec<BookSearchResultDto>, AppError> {
    let limit = search.limit.unwrap_or(20).clamp(1, 50);
    let term = search.q.trim();

    if term.len() < 2 {
        return Err(AppError::BadRequest(
            "Search query must be at least 2 characters".to_string(),
        ));
    }

    let sql = r#"
        SELECT id, title, author, language, primary_theme, cover_url, estimated_reading_minutes,
               GREATEST(word_similarity($1, title), word_similarity($1, author), similarity(title || ' ' || author, $1))::real AS score
        FROM books
        WHERE status = 'published'
          AND ($2::text IS NULL OR language = $2)
          AND (
              title %> $1
              OR author %> $1
              OR title % $1
              OR author % $1
              OR to_tsvector('simple', title || ' ' || author || ' ' || description) @@ plainto_tsquery($1)
          )
        ORDER BY score DESC, title ASC
        LIMIT $3;
    "#;

    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        vec![
            Value::from(term),
            Value::from(search.language.clone()),
            Value::from(limit as i64),
        ],
    );

    let rows = SearchResultRow::find_by_statement(stmt)
        .all(db)
        .await
        .map_err(|e| AppError::Database(format!("FTS search failed: {e}")))?;

    let dtos = rows
        .into_iter()
        .map(|r| BookSearchResultDto {
            id: r.id,
            title: r.title,
            author: r.author,
            language: r.language,
            primary_theme: r.primary_theme,
            cover_url: r.cover_url,
            estimated_reading_minutes: r.estimated_reading_minutes,
            score: r.score,
        })
        .collect();

    Ok(dtos)
}

/// Retrieves complete book details with tags and chapter summary list.
pub async fn get_book_by_id(
    db: &DatabaseConnection,
    book_id: Uuid,
) -> Result<BookDetailDto, AppError> {
    let book = books::Entity::find_by_id(book_id)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to retrieve book: {e}")))?
        .ok_or_else(|| AppError::NotFound(format!("Book not found: {book_id}")))?;

    if book.status != "published" {
        return Err(AppError::NotFound("Book is not published".to_string()));
    }

    let tags = get_tags_for_book(db, book.id).await?;

    let chapter_records = chapters::Entity::find()
        .filter(chapters::Column::BookId.eq(book.id))
        .order_by_asc(chapters::Column::ChapterNumber)
        .all(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to retrieve chapters: {e}")))?;

    let chapters = chapter_records
        .into_iter()
        .map(|c| ChapterSummaryDto {
            id: c.id,
            chapter_number: c.chapter_number,
            title: c.title,
            word_count: c.word_count,
        })
        .collect();

    Ok(BookDetailDto {
        id: book.id,
        title: book.title,
        author: book.author,
        language: book.language,
        primary_theme: book.primary_theme,
        sub_theme: book.sub_theme,
        description: book.description,
        cover_url: book.cover_url,
        total_words: book.total_words,
        estimated_reading_minutes: book.estimated_reading_minutes,
        source_name: book.source_name,
        source_url: book.source_url,
        license: book.license,
        publication_year: book.publication_year,
        status: book.status,
        tags,
        chapters,
    })
}

/// Returns the owning `book_id` for a chapter UUID.
///
/// Used to validate (book_id, chapter_id) pairs on write paths (e.g. saving a
/// quote): 404 when the chapter does not exist, letting the caller reject
/// cross-book mismatches with a 400 instead of leaking a 500 FK violation.
pub async fn get_chapter_book_id(
    db: &DatabaseConnection,
    chapter_id: Uuid,
) -> Result<Uuid, AppError> {
    let chapter = chapters::Entity::find_by_id(chapter_id)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to retrieve chapter: {e}")))?
        .ok_or_else(|| AppError::NotFound(format!("Chapter not found: {chapter_id}")))?;

    Ok(chapter.book_id)
}

/// Returns the chapter number for a chapter UUID, if it exists.
pub async fn get_chapter_number(
    db: &DatabaseConnection,
    chapter_id: Uuid,
) -> Result<Option<i32>, AppError> {
    let chapter = chapters::Entity::find_by_id(chapter_id)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to retrieve chapter: {e}")))?;

    Ok(chapter.map(|c| c.chapter_number))
}

/// Retrieves single chapter content by chapter number.
pub async fn get_chapter_by_number(
    db: &DatabaseConnection,
    book_id: Uuid,
    chapter_number: i32,
) -> Result<ChapterDetailDto, AppError> {
    let chapter = chapters::Entity::find()
        .filter(chapters::Column::BookId.eq(book_id))
        .filter(chapters::Column::ChapterNumber.eq(chapter_number))
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to retrieve chapter: {e}")))?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Chapter {chapter_number} not found for book {book_id}"
            ))
        })?;

    Ok(ChapterDetailDto {
        id: chapter.id,
        book_id: chapter.book_id,
        chapter_number: chapter.chapter_number,
        title: chapter.title,
        word_count: chapter.word_count,
        html_content: chapter.html_content,
    })
}

/// Bundles book metadata and all chapters for offline IndexedDB synchronization.
pub async fn get_offline_bundle(
    db: &DatabaseConnection,
    book_id: Uuid,
) -> Result<OfflineBundleDto, AppError> {
    let book = get_book_by_id(db, book_id).await?;

    let chapter_records = chapters::Entity::find()
        .filter(chapters::Column::BookId.eq(book_id))
        .order_by_asc(chapters::Column::ChapterNumber)
        .all(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to retrieve chapters for bundle: {e}")))?;

    let chapters = chapter_records
        .into_iter()
        .map(|c| ChapterDetailDto {
            id: c.id,
            book_id: c.book_id,
            chapter_number: c.chapter_number,
            title: c.title,
            word_count: c.word_count,
            html_content: c.html_content,
        })
        .collect();

    Ok(OfflineBundleDto { book, chapters })
}

/// Chapter drop-off funnel for the admin retention analytics.
pub async fn chapter_dropoff(
    db: &DatabaseConnection,
    book_id: Uuid,
) -> Result<Vec<shared::DropOffPointDto>, AppError> {
    #[derive(Debug, FromQueryResult)]
    struct FunnelRow {
        chapter_number: i32,
        title: String,
        reached: i64,
    }

    let sql = r#"
        SELECT ch.chapter_number, ch.title,
               COUNT(DISTINCT p.user_id)::bigint AS reached
        FROM chapters ch
        LEFT JOIN user_reading_progress p
          ON p.book_id = ch.book_id
         AND p.last_chapter_id IN (
                SELECT id FROM chapters
                WHERE book_id = $1 AND chapter_number >= ch.chapter_number
             )
        WHERE ch.book_id = $1
        GROUP BY ch.chapter_number, ch.title
        ORDER BY ch.chapter_number ASC
    "#;

    let stmt =
        Statement::from_sql_and_values(DatabaseBackend::Postgres, sql, [Value::from(book_id)]);

    let rows = FunnelRow::find_by_statement(stmt)
        .all(db)
        .await
        .map_err(|e| AppError::Database(format!("Drop-off funnel query failed: {e}")))?;

    let baseline = rows.first().map(|r| r.reached).unwrap_or(0);
    Ok(rows
        .into_iter()
        .map(|r| {
            let drop_off_pct = if baseline > 0 {
                ((baseline - r.reached).max(0) as f32 / baseline as f32) * 100.0
            } else {
                0.0
            };
            shared::DropOffPointDto {
                chapter_number: r.chapter_number,
                chapter_title: r.title,
                readers_reached: r.reached,
                drop_off_pct,
            }
        })
        .collect())
}

/// Helper function to retrieve tag names for a given book.
async fn get_tags_for_book(
    db: &DatabaseConnection,
    book_id: Uuid,
) -> Result<Vec<String>, AppError> {
    let book_tag_relations = book_tags::Entity::find()
        .filter(book_tags::Column::BookId.eq(book_id))
        .find_also_related(tags::Entity)
        .all(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch book tags: {e}")))?;

    let mut tag_names = Vec::new();
    for (_bt, maybe_tag) in book_tag_relations {
        if let Some(t) = maybe_tag {
            tag_names.push(t.name);
        }
    }

    Ok(tag_names)
}

/// Raw row of the curator listing.
#[derive(Debug, FromQueryResult)]
struct AdminBookRow {
    id: Uuid,
    title: String,
    author: String,
    language: String,
    status: String,
    chapter_count: i64,
    chunk_count: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

/// Every book regardless of status, newest first, with chapter and chunk
/// counts; keyset-paginated on (created_at, id) by the cursor book.
pub async fn list_books_for_admin(
    db: &DatabaseConnection,
    status: Option<&str>,
    cursor: Option<Uuid>,
    limit: u64,
) -> Result<Vec<AdminBookRowDto>, AppError> {
    let sql = r#"
        SELECT b.id, b.title, b.author, b.language, b.status, b.created_at, b.updated_at,
               (SELECT count(*) FROM chapters c WHERE c.book_id = b.id) AS chapter_count,
               (SELECT count(*) FROM book_chunks k WHERE k.book_id = b.id) AS chunk_count
        FROM books b
        WHERE ($1::varchar IS NULL OR b.status = $1)
          AND ($2::uuid IS NULL
               OR (b.created_at, b.id) < (SELECT created_at, id FROM books WHERE id = $2))
        ORDER BY b.created_at DESC, b.id DESC
        LIMIT $3
    "#;
    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        sql,
        [
            sea_orm::Value::from(status.map(str::to_string)),
            sea_orm::Value::from(cursor),
            sea_orm::Value::from(limit.clamp(1, 100) as i64),
        ],
    );
    let rows = AdminBookRow::find_by_statement(stmt)
        .all(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to list books for admin: {e}")))?;
    Ok(rows
        .into_iter()
        .map(|r| AdminBookRowDto {
            id: r.id,
            title: r.title,
            author: r.author,
            language: r.language,
            status: r.status,
            chapter_count: r.chapter_count,
            chunk_count: r.chunk_count,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect())
}

/// Current lifecycle status of any book, published or not.
pub async fn get_book_status(db: &DatabaseConnection, book_id: Uuid) -> Result<String, AppError> {
    books::Entity::find_by_id(book_id)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to load book status: {e}")))?
        .map(|b| b.status)
        .ok_or_else(|| AppError::NotFound("Book not found".to_string()))
}

/// Writes a status the caller has already validated against the lifecycle.
pub async fn set_book_status(
    db: &DatabaseConnection,
    book_id: Uuid,
    status: &str,
) -> Result<(), AppError> {
    use sea_orm::ActiveModelTrait;
    let model = books::ActiveModel {
        id: sea_orm::Set(book_id),
        status: sea_orm::Set(status.to_string()),
        updated_at: sea_orm::Set(chrono::Utc::now().into()),
        ..Default::default()
    };
    model
        .update(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to update book status: {e}")))?;
    Ok(())
}
