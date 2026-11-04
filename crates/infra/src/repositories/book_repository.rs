//! Catalog, search, chapter, and bundle queries.

use sea_orm::sea_query::NullOrdering;
use sea_orm::{
    ColumnTrait, Condition, DatabaseBackend, DatabaseConnection, EntityTrait, FromQueryResult,
    Order, QueryFilter, QueryOrder, QuerySelect, Select, Statement, Value,
};
use shared::{
    AdminBookRowDto, AppError, BookCatalogQuery, BookDetailDto, BookSearchQuery,
    BookSearchResultDto, BookSummaryDto, BookTextHitDto, ChapterDetailDto, ChapterSummaryDto,
    OfflineBundleDto,
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

/// One keyset page of the catalog after `cursor`, in the order
/// `idx_books_published_year_id` stores it: year descending with undated
/// books last, then id. Postgres puts NULLs first on a plain DESC, which both
/// forced a sort and stranded every undated book behind an `id > cursor`
/// that ignored the year.
pub fn catalog_page(
    select: Select<books::Entity>,
    cursor: Option<&books::Model>,
    limit: u64,
) -> Select<books::Entity> {
    let year = books::Column::PublicationYear;
    let select = match cursor {
        Some(c) => select.filter(match c.publication_year {
            Some(y) => Condition::any()
                .add(year.lt(y))
                .add(year.eq(y).and(books::Column::Id.gt(c.id)))
                .add(year.is_null()),
            None => Condition::all()
                .add(year.is_null())
                .add(books::Column::Id.gt(c.id)),
        }),
        None => select,
    };
    select
        .order_by_with_nulls(year, Order::Desc, NullOrdering::Last)
        .order_by_asc(books::Column::Id)
        .limit(limit)
}

/// Lists published books with cursor pagination and taxonomy filters.
pub async fn list_books(
    db: &DatabaseConnection,
    query: &BookCatalogQuery,
) -> Result<Vec<BookSummaryDto>, AppError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100).unsigned_abs();

    let mut select = books::Entity::find().filter(books::Column::Status.eq("published"));

    if let Some(ref lang) = query.language {
        select = select.filter(books::Column::Language.eq(lang));
    }

    if let Some(ref theme) = query.theme {
        select = select.filter(books::Column::PrimaryTheme.eq(theme));
    }

    let cursor_book = match query.cursor {
        Some(cursor_id) => books::Entity::find_by_id(cursor_id)
            .one(db)
            .await
            .map_err(|e| AppError::Database(format!("Failed to retrieve cursor book: {e}")))?,
        None => None,
    };

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

    let book_models = catalog_page(select, cursor_book.as_ref(), limit)
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

/// Paragraphs of one published book that contain `query`, in reading order.
///
/// Reads `chapter_paragraphs`, the plain text the worker indexes per `<p>`
/// with the reader's numbering, matching the folded copy through its
/// trigram index; the snippet comes
/// from [`shared::text_hit_snippet`], as in offline search.
pub async fn search_book_text(
    db: &DatabaseConnection,
    book_id: Uuid,
    query: &str,
) -> Result<Vec<BookTextHitDto>, AppError> {
    ensure_book_published(db, book_id).await?;
    let pattern = format!(
        "%{}%",
        shared::fold_for_search(query)
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    );
    let sql = r#"
        SELECT p.chapter_number, c.title AS chapter_title, p.n AS paragraph, p.text
        FROM chapter_paragraphs p
        JOIN chapters c ON c.book_id = p.book_id AND c.chapter_number = p.chapter_number
        WHERE p.book_id = $1 AND p.folded LIKE $2 ESCAPE '\'
        ORDER BY p.chapter_number, p.n
        LIMIT $3;
    "#;
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        vec![
            Value::from(book_id),
            Value::from(pattern),
            Value::from(shared::BOOK_TEXT_HITS_MAX as i64),
        ],
    );
    let rows = TextHitRow::find_by_statement(stmt)
        .all(db)
        .await
        .map_err(|e| AppError::Database(format!("In-book search failed: {e}")))?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            shared::text_hit_snippet(&row.text, query).map(|snippet| BookTextHitDto {
                chapter_number: row.chapter_number,
                chapter_title: row.chapter_title,
                paragraph: row.paragraph,
                snippet,
            })
        })
        .collect())
}

#[derive(Debug, FromQueryResult)]
struct TextHitRow {
    chapter_number: i32,
    chapter_title: String,
    paragraph: i32,
    text: String,
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
              OR to_tsvector('simple', title || ' ' || author || ' ' || description) @@ plainto_tsquery('simple', $1)
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
            Value::from(limit),
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

/// The one answer for a book that is missing, a draft, or archived, so a
/// public caller cannot tell an unpublished manuscript from a random id.
fn book_not_found() -> AppError {
    AppError::NotFound("Book not found".to_string())
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
        .filter(|book| book.status == "published")
        .ok_or_else(book_not_found)?;

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

/// The chapter's text in the form quotes are matched in: folded, with no
/// whitespace at all. A reader's selection runs paragraphs together with
/// line breaks and inline tags without spaces, while a quote-search passage
/// puts a space at every tag, and either must match the page it came from.
/// A long chapter is megabytes of text, so the work runs off the async
/// threads; a caller checking several quotes reuses one result per chapter.
pub async fn chapter_quote_text(
    db: &DatabaseConnection,
    chapter_id: Uuid,
) -> Result<String, AppError> {
    let html: Option<String> = chapters::Entity::find_by_id(chapter_id)
        .select_only()
        .column(chapters::Column::HtmlContent)
        .into_tuple()
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to retrieve chapter: {e}")))?;
    let html =
        html.ok_or_else(|| AppError::NotFound(format!("Chapter not found: {chapter_id}")))?;
    tokio::task::spawn_blocking(move || squeezed(&decode_entities(&strip_tags(&html))))
        .await
        .map_err(|e| AppError::Internal(format!("Chapter text task failed: {e}")))
}

/// True when `quote` occurs in text from [`chapter_quote_text`].
pub fn quote_in_text(chapter_text: &str, quote: &str) -> bool {
    let needle = squeezed(quote);
    !needle.is_empty() && chapter_text.contains(&needle)
}

fn strip_tags(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    text
}

/// Soft hyphens go too: a browser may leave them out of a selection.
fn squeezed(text: &str) -> String {
    shared::fold_for_search(text).replace([' ', '\u{00AD}'], "")
}

/// Character references left in stored chapter HTML, decoded as a browser
/// decodes them.
fn decode_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let tail = &rest[amp..];
        let decoded = tail.find(';').filter(|end| *end <= 10).and_then(|end| {
            let name = &tail[1..end];
            let ch = match name {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                "nbsp" => Some(' '),
                "shy" => Some('\u{00AD}'),
                _ => name
                    .strip_prefix("#x")
                    .or_else(|| name.strip_prefix("#X"))
                    .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                    .or_else(|| name.strip_prefix('#').and_then(|dec| dec.parse().ok()))
                    .and_then(char::from_u32),
            }?;
            Some((ch, end + 1))
        });
        match decoded {
            Some((ch, len)) => {
                out.push(ch);
                rest = &tail[len..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
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

/// 404 unless the book exists and is published. Public reads that address
/// a book without loading it go through here, so drafts and archived books
/// stay invisible even to a caller holding their UUID.
pub async fn ensure_book_published(db: &DatabaseConnection, book_id: Uuid) -> Result<(), AppError> {
    if get_book_status(db, book_id).await? != "published" {
        return Err(book_not_found());
    }
    Ok(())
}

/// Retrieves single chapter content by chapter number. Only chapters of a
/// published book resolve; a draft or archived book answers 404.
pub async fn get_chapter_by_number(
    db: &DatabaseConnection,
    book_id: Uuid,
    chapter_number: i32,
) -> Result<ChapterDetailDto, AppError> {
    let chapter = chapters::Entity::find()
        .inner_join(books::Entity)
        .filter(chapters::Column::BookId.eq(book_id))
        .filter(chapters::Column::ChapterNumber.eq(chapter_number))
        .filter(books::Column::Status.eq("published"))
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

impl From<AdminBookRow> for AdminBookRowDto {
    fn from(r: AdminBookRow) -> Self {
        Self {
            id: r.id,
            title: r.title,
            author: r.author,
            language: r.language,
            status: r.status,
            chapter_count: r.chapter_count,
            chunk_count: r.chunk_count,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

/// Columns of one curator row; the listing and the single-row lookup share
/// it so both shapes stay identical.
const ADMIN_ROW_SELECT: &str = r#"
        SELECT b.id, b.title, b.author, b.language, b.status, b.created_at, b.updated_at,
               (SELECT count(*) FROM chapters c WHERE c.book_id = b.id) AS chapter_count,
               (SELECT count(*) FROM book_chunks k WHERE k.book_id = b.id) AS chunk_count
        FROM books b"#;

/// Every book regardless of status, newest first, with chapter and chunk
/// counts; keyset-paginated on (created_at, id) by the cursor book.
pub async fn list_books_for_admin(
    db: &DatabaseConnection,
    status: Option<&str>,
    cursor: Option<Uuid>,
    limit: u64,
) -> Result<Vec<AdminBookRowDto>, AppError> {
    let sql = format!(
        r#"{ADMIN_ROW_SELECT}
        WHERE ($1::varchar IS NULL OR b.status = $1)
          AND ($2::uuid IS NULL
               OR (b.created_at, b.id) < (SELECT created_at, id FROM books WHERE id = $2))
        ORDER BY b.created_at DESC, b.id DESC
        LIMIT $3
    "#
    );
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
    Ok(rows.into_iter().map(AdminBookRowDto::from).collect())
}

/// One curator row by id, in any status.
pub async fn get_book_for_admin(
    db: &DatabaseConnection,
    book_id: Uuid,
) -> Result<Option<AdminBookRowDto>, AppError> {
    let sql = format!("{ADMIN_ROW_SELECT} WHERE b.id = $1");
    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        sql,
        [sea_orm::Value::from(book_id)],
    );
    AdminBookRow::find_by_statement(stmt)
        .one(db)
        .await
        .map(|row| row.map(AdminBookRowDto::from))
        .map_err(|e| AppError::Database(format!("Failed to load book for admin: {e}")))
}

/// Titles of the books that still exist among `ids`.
pub async fn book_titles(
    db: &DatabaseConnection,
    ids: Vec<Uuid>,
) -> Result<std::collections::HashMap<Uuid, String>, AppError> {
    if ids.is_empty() {
        return Ok(Default::default());
    }
    let rows: Vec<(Uuid, String)> = books::Entity::find()
        .select_only()
        .column(books::Column::Id)
        .column(books::Column::Title)
        .filter(books::Column::Id.is_in(ids))
        .into_tuple()
        .all(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to load book titles: {e}")))?;
    Ok(rows.into_iter().collect())
}

/// True once ingestion finished at least once: the worker writes the word
/// count only in the step that publishes.
pub async fn is_fully_ingested(db: &DatabaseConnection, book_id: Uuid) -> Result<bool, AppError> {
    books::Entity::find_by_id(book_id)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to load book: {e}")))?
        .map(|b| b.total_words > 0)
        .ok_or_else(book_not_found)
}

/// Current lifecycle status of any book, published or not.
pub async fn get_book_status(db: &DatabaseConnection, book_id: Uuid) -> Result<String, AppError> {
    books::Entity::find_by_id(book_id)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to load book status: {e}")))?
        .map(|b| b.status)
        .ok_or_else(book_not_found)
}

/// True for Postgres `lock_not_available`, raised when `lock_timeout` ends a
/// wait for a row another transaction holds.
fn is_lock_timeout(error: &sea_orm::DbErr) -> bool {
    use sea_orm::{sqlx, DbErr, RuntimeErr};
    match error {
        DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(db)))
        | DbErr::Query(RuntimeErr::SqlxError(sqlx::Error::Database(db))) => {
            db.code().as_deref() == Some("55P03")
        }
        _ => false,
    }
}

/// Writes a status the caller has already validated against the lifecycle.
/// A curator's request waits at most two seconds for a row someone else
/// holds, then answers 409 so it can be retried, instead of holding a pool
/// connection for as long as the other transaction runs.
pub async fn set_book_status(
    db: &DatabaseConnection,
    book_id: Uuid,
    status: &str,
) -> Result<(), AppError> {
    use sea_orm::{ActiveModelTrait, ConnectionTrait, TransactionTrait};
    let failed =
        |e: sea_orm::DbErr| AppError::Database(format!("Failed to update book status: {e}"));
    let txn = db.begin().await.map_err(failed)?;
    txn.execute_unprepared("SET LOCAL lock_timeout = '2s'")
        .await
        .map_err(failed)?;
    let model = books::ActiveModel {
        id: sea_orm::Set(book_id),
        status: sea_orm::Set(status.to_string()),
        updated_at: sea_orm::Set(chrono::Utc::now().into()),
        ..Default::default()
    };
    model.update(&txn).await.map_err(|e| {
        if is_lock_timeout(&e) {
            AppError::Conflict(
                "This manuscript is busy right now; try again in a moment".to_string(),
            )
        } else {
            failed(e)
        }
    })?;
    txn.commit().await.map_err(failed)
}

#[cfg(test)]
mod tests {
    use super::{decode_entities, squeezed, strip_tags};

    #[test]
    fn character_references_decode_like_a_browser() {
        assert_eq!(decode_entities("Tom &amp; Jerry"), "Tom & Jerry");
        assert_eq!(
            decode_entities("it&#8217;s &#x2014; ok"),
            "it\u{2019}s \u{2014} ok"
        );
        assert_eq!(decode_entities("a&nbsp;b &lt;p&gt;"), "a b <p>");
        assert_eq!(
            decode_entities("AT&T and &unknown; stay"),
            "AT&T and &unknown; stay"
        );
    }

    #[test]
    fn a_quote_matches_however_the_page_was_selected() {
        let html = "<h2>Bab I</h2><p>Laras menulis <em>surat</em>-nya &amp; menunggu.</p><p>Kapal tiba.</p>";
        let page = squeezed(&decode_entities(&strip_tags(html)));
        // A browser selection: inline tag without spaces, a paragraph break.
        assert!(page.contains(&squeezed("surat-nya & menunggu.\n\nKapal tiba.")));
        // A quote-search passage: a space at every tag, the heading included.
        assert!(page.contains(&squeezed(
            "Bab I Laras menulis surat -nya & menunggu. Kapal tiba."
        )));
        assert!(!page.contains(&squeezed("Laras menulis puisi")));
    }

    #[test]
    fn a_soft_hyphen_does_not_split_a_quote() {
        let page = squeezed(&decode_entities(&strip_tags(
            "<p>sur&shy;at\u{00AD}nya</p>",
        )));
        assert_eq!(page, squeezed("suratnya"));
    }
}
