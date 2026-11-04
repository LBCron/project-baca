//! Progress, CFI anchors, heartbeats, and streaks.

use chrono::Utc;
use sea_orm::entity::prelude::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set, Statement, Value,
};
use shared::{
    ActiveProgressDto, AppError, ReadingHeartbeatRequest, ReadingHeartbeatResponse,
    ReadingStreakDto, UpdateProgressRequest,
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::entities::{
    books, chapters, reading_activity_logs, user_badges, user_reading_progress,
    user_reading_streaks,
};

/// Most unfinished books one shelf lists; a reader with more sees the most
/// recently read ones.
pub const READING_SHELF_LIMIT: u64 = 100;

/// Retrieves the latest unfinished reading progress for a user.
pub async fn get_active_progress(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Option<ActiveProgressDto>, AppError> {
    Ok(list_active_progress(db, user_id, 1)
        .await?
        .into_iter()
        .next())
}

/// Unfinished reading positions, most recently read first. Progress on a
/// book that was unpublished later stays stored but must not surface its
/// title or cover on the shelf.
pub async fn list_active_progress(
    db: &DatabaseConnection,
    user_id: Uuid,
    limit: u64,
) -> Result<Vec<ActiveProgressDto>, AppError> {
    let rows = user_reading_progress::Entity::find()
        .find_also_related(books::Entity)
        .filter(books::Column::Status.eq("published"))
        .filter(user_reading_progress::Column::UserId.eq(user_id))
        .filter(user_reading_progress::Column::IsFinished.eq(false))
        .order_by_desc(user_reading_progress::Column::LastReadAt)
        .limit(limit)
        .all(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to retrieve active progress: {e}")))?;

    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let chapter_ids: Vec<Uuid> = rows.iter().map(|(p, _)| p.last_chapter_id).collect();
    let chapters: HashMap<Uuid, chapters::Model> = chapters::Entity::find()
        .filter(chapters::Column::Id.is_in(chapter_ids))
        .all(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query chapters for progress: {e}")))?
        .into_iter()
        .map(|c| (c.id, c))
        .collect();

    Ok(rows
        .into_iter()
        .filter_map(|(progress, book)| {
            let book = book?;
            let chapter = chapters.get(&progress.last_chapter_id)?;
            Some(ActiveProgressDto {
                book_id: book.id,
                book_title: book.title,
                book_author: book.author,
                book_cover_url: book.cover_url,
                chapter_id: chapter.id,
                chapter_number: chapter.chapter_number,
                chapter_title: chapter.title.clone(),
                last_anchor_cfi: progress.last_anchor_cfi,
                completion_percentage: progress
                    .completion_percentage
                    .to_string()
                    .parse()
                    .unwrap_or(0.0),
                is_finished: progress.is_finished,
                last_read_at: progress.last_read_at.with_timezone(&Utc),
            })
        })
        .collect())
}

/// Upserts user reading progress with CFI position and completion percentage.
pub async fn update_progress(
    db: &DatabaseConnection,
    user_id: Uuid,
    book_id: Uuid,
    req: &UpdateProgressRequest,
) -> Result<(), AppError> {
    crate::ensure_book_published(db, book_id).await?;

    // Validate chapter belongs to book
    let chapter = chapters::Entity::find_by_id(req.chapter_id)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to verify chapter: {e}")))?
        .ok_or_else(|| AppError::NotFound(format!("Chapter not found: {}", req.chapter_id)))?;

    if chapter.book_id != book_id {
        return Err(AppError::BadRequest(
            "Chapter does not belong to the specified book".to_string(),
        ));
    }

    // Domain-owned invariant: completion lives in 0.0-100.0 and finishes
    // exactly at 100 (the API edge validates the wire range; this guards
    // every non-HTTP caller such as workers and tests).
    let percentage = domain::Percentage::new(req.completion_percentage)?;
    let is_finished = percentage.is_finished();
    let percentage_dec: Decimal = percentage
        .decimal_string()
        .parse()
        .map_err(|e| AppError::BadRequest(format!("Invalid percentage value: {e}")))?;

    let sql = r#"
        INSERT INTO user_reading_progress (
            id, user_id, book_id, last_chapter_id, last_anchor_cfi, completion_percentage, is_finished, last_read_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, NOW(), NOW()
        )
        ON CONFLICT (user_id, book_id) DO UPDATE SET
            last_chapter_id = EXCLUDED.last_chapter_id,
            last_anchor_cfi = EXCLUDED.last_anchor_cfi,
            completion_percentage = EXCLUDED.completion_percentage,
            is_finished = EXCLUDED.is_finished,
            last_read_at = NOW(),
            updated_at = NOW();
    "#;

    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        vec![
            Value::from(Uuid::new_v4()),
            Value::from(user_id),
            Value::from(book_id),
            Value::from(req.chapter_id),
            Value::from(req.last_anchor_cfi.clone()),
            Value::from(percentage_dec),
            Value::from(is_finished),
        ],
    );

    db.execute(stmt)
        .await
        .map_err(|e| AppError::Database(format!("Failed to update reading progress: {e}")))?;

    Ok(())
}

async fn seconds_read_on(
    db: &DatabaseConnection,
    user_id: Uuid,
    day: chrono::NaiveDate,
) -> Result<i64, AppError> {
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        r#"
        SELECT COALESCE(SUM(seconds_spent), 0)::bigint AS total_today
        FROM reading_activity_logs
        WHERE user_id = $1 AND activity_date = $2;
    "#,
        vec![Value::from(user_id), Value::from(day)],
    );
    let row = db
        .query_one(stmt)
        .await
        .map_err(|e| AppError::Database(format!("Failed to aggregate daily reading time: {e}")))?
        .ok_or_else(|| AppError::Internal("Failed to calculate daily reading time".to_string()))?;
    row.try_get_by_index(0)
        .map_err(|e| AppError::Database(format!("Failed to parse daily reading sum: {e}")))
}

/// Streak standing for the profile page; zeros before the first heartbeat.
pub async fn get_streak(
    db: &DatabaseConnection,
    user_id: Uuid,
    utc_offset_minutes: i32,
) -> Result<ReadingStreakDto, AppError> {
    let record = user_reading_streaks::Entity::find_by_id(user_id)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch streak record: {e}")))?;
    let today = domain::local_date(Utc::now(), utc_offset_minutes);
    let today_seconds = seconds_read_on(db, user_id, today).await?;

    Ok(ReadingStreakDto {
        current_streak_days: record.as_ref().map(|s| s.current_streak_days).unwrap_or(0),
        longest_streak_days: record.as_ref().map(|s| s.longest_streak_days).unwrap_or(0),
        total_reading_seconds: record
            .as_ref()
            .map(|s| s.total_reading_seconds)
            .unwrap_or(0),
        total_xp: record.as_ref().map(|s| s.total_xp).unwrap_or(0),
        last_activity_date: record.and_then(|s| s.last_activity_date),
        today_seconds,
        daily_threshold_seconds: domain::DAILY_THRESHOLD_SECONDS,
    })
}

/// Logs reading activity heartbeat, calculates daily streak, and awards XP.
pub async fn record_heartbeat(
    db: &DatabaseConnection,
    user_id: Uuid,
    req: &ReadingHeartbeatRequest,
) -> Result<ReadingHeartbeatResponse, AppError> {
    let book_exists = books::Entity::find_by_id(req.book_id)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to verify book: {e}")))?
        .ok_or_else(|| AppError::NotFound(format!("Book not found: {}", req.book_id)))?;
    if book_exists.status != "published" {
        return Err(AppError::NotFound(format!(
            "Book not found: {}",
            req.book_id
        )));
    }

    let now = Utc::now();
    let today = domain::local_date(now, req.utc_offset_minutes.unwrap_or(0));
    let credited_seconds = domain::credited_heartbeat_seconds(req.seconds_spent);

    let log_entry = reading_activity_logs::ActiveModel {
        user_id: Set(user_id),
        book_id: Set(req.book_id),
        seconds_spent: Set(credited_seconds),
        activity_date: Set(today),
        created_at: Set(now.into()),
        ..Default::default()
    };

    log_entry
        .insert(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to record activity log: {e}")))?;

    let total_today_seconds = seconds_read_on(db, user_id, today).await?;

    // Fetch or initialize user_reading_streaks
    let streak_record = user_reading_streaks::Entity::find_by_id(user_id)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch streak record: {e}")))?;

    let mut current_streak = streak_record
        .as_ref()
        .map(|s| s.current_streak_days)
        .unwrap_or(0);
    let mut longest_streak = streak_record
        .as_ref()
        .map(|s| s.longest_streak_days)
        .unwrap_or(0);
    let last_active_date = streak_record.as_ref().and_then(|s| s.last_activity_date);

    // Domain-owned transition rules (threshold evaluation + date arithmetic).
    let outcome = domain::advance_streak(
        current_streak,
        longest_streak,
        last_active_date,
        today,
        total_today_seconds >= domain::DAILY_THRESHOLD_SECONDS,
    );
    current_streak = outcome.current_days;
    longest_streak = outcome.longest_days;
    let xp_earned = domain::BASE_HEARTBEAT_XP + outcome.bonus_xp;

    // Totals are added in place, not written back from the read above: two
    // heartbeats racing past a Redis outage must not overwrite each other's
    // seconds or XP (a badge reward included), and the first two cannot
    // collide on the insert.
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO user_reading_streaks AS s \
               (user_id, current_streak_days, longest_streak_days, total_reading_seconds, \
                total_xp, last_activity_date, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (user_id) DO UPDATE SET \
               current_streak_days = EXCLUDED.current_streak_days, \
               longest_streak_days = GREATEST(s.longest_streak_days, EXCLUDED.longest_streak_days), \
               total_reading_seconds = s.total_reading_seconds + EXCLUDED.total_reading_seconds, \
               total_xp = s.total_xp + EXCLUDED.total_xp, \
               last_activity_date = EXCLUDED.last_activity_date, \
               updated_at = EXCLUDED.updated_at \
             RETURNING total_reading_seconds, total_xp",
            [
                Value::from(user_id),
                Value::from(current_streak),
                Value::from(longest_streak),
                Value::from(i64::from(credited_seconds)),
                Value::from(xp_earned),
                Value::from(outcome.last_active),
                Value::from(now),
            ],
        ))
        .await
        .map_err(|e| AppError::Database(format!("Failed to update streak record: {e}")))?
        .ok_or_else(|| AppError::Database("Streak upsert returned no row".to_string()))?;
    let total_reading_seconds: i64 = row
        .try_get("", "total_reading_seconds")
        .map_err(|e| AppError::Database(format!("Failed to read streak totals: {e}")))?;
    let total_xp: i32 = row
        .try_get("", "total_xp")
        .map_err(|e| AppError::Database(format!("Failed to read streak totals: {e}")))?;

    let badge_xp =
        check_and_award_badges(db, user_id, current_streak, total_reading_seconds).await?;

    Ok(ReadingHeartbeatResponse {
        current_streak_days: current_streak,
        longest_streak_days: longest_streak,
        total_reading_seconds,
        total_xp: total_xp + badge_xp,
        xp_earned: xp_earned + badge_xp,
        streak_incremented: outcome.incremented,
        credited: true,
    })
}

/// The reader's standing unchanged, for a heartbeat that arrived inside the
/// per-user interval (a second tab, say) and earns nothing.
pub async fn heartbeat_standing(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<ReadingHeartbeatResponse, AppError> {
    let record = user_reading_streaks::Entity::find_by_id(user_id)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch streak record: {e}")))?;
    Ok(ReadingHeartbeatResponse {
        current_streak_days: record.as_ref().map(|s| s.current_streak_days).unwrap_or(0),
        longest_streak_days: record.as_ref().map(|s| s.longest_streak_days).unwrap_or(0),
        total_reading_seconds: record
            .as_ref()
            .map(|s| s.total_reading_seconds)
            .unwrap_or(0),
        total_xp: record.as_ref().map(|s| s.total_xp).unwrap_or(0),
        xp_earned: 0,
        streak_incremented: false,
        credited: false,
    })
}

/// Unlocks the reading-milestone badges the reader has newly earned and adds
/// their XP to the streak row; returns the XP added.
async fn check_and_award_badges(
    db: &DatabaseConnection,
    user_id: Uuid,
    streak_days: i32,
    total_seconds: i64,
) -> Result<i32, AppError> {
    // Domain-owned eligibility matrix (ids must exist in `badges` seed data).
    let eligible = domain::eligible_badges(streak_days, total_seconds);
    if eligible.is_empty() {
        return Ok(0);
    }

    let existing: Vec<String> = user_badges::Entity::find()
        .filter(user_badges::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query user badges: {e}")))?
        .into_iter()
        .map(|b| b.badge_id)
        .collect();

    let missing: Vec<&str> = eligible
        .into_iter()
        .filter(|id| !existing.iter().any(|e| e == id))
        .collect();
    if missing.is_empty() {
        return Ok(0);
    }
    // The master rows are otherwise seeded by the first badge listing; on a
    // fresh install a reader can earn a badge before anyone lists them, and
    // the insert below would break the foreign key.
    crate::repositories::badge_repository::seed_default_badges_if_empty(db).await?;

    let now = Utc::now();
    let mut values_sql = String::new();
    let mut values: Vec<Value> = Vec::with_capacity(missing.len() * 4);
    for (i, badge_id) in missing.iter().enumerate() {
        if i > 0 {
            values_sql.push_str(", ");
        }
        let base = i * 4 + 1;
        values_sql.push_str(&format!(
            "(${base}, ${}, ${}, ${})",
            base + 1,
            base + 2,
            base + 3
        ));
        values.push(Value::from(Uuid::new_v4()));
        values.push(Value::from(user_id));
        values.push(Value::from((*badge_id).to_string()));
        values.push(Value::from(now));
    }
    // One statement: only rows this insert actually wrote add XP, so a
    // concurrent heartbeat that loses the ON CONFLICT race adds nothing and
    // a badge's reward lands exactly once with the badge.
    let user_param = values.len() + 1;
    values.push(Value::from(user_id));
    let sql = format!(
        "WITH awarded AS (\
            INSERT INTO user_badges (id, user_id, badge_id, unlocked_at) \
            VALUES {values_sql} ON CONFLICT (user_id, badge_id) DO NOTHING \
            RETURNING badge_id), \
         reward AS (\
            SELECT COALESCE(SUM(b.xp_reward), 0)::int AS xp \
            FROM awarded a JOIN badges b ON b.id = a.badge_id) \
         UPDATE user_reading_streaks SET total_xp = total_xp + reward.xp \
         FROM reward WHERE user_id = ${user_param} \
         RETURNING reward.xp"
    );

    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            sql,
            values,
        ))
        .await
        .map_err(|e| AppError::Database(format!("Failed to award badges: {e}")))?;
    row.map(|r| r.try_get::<i32>("", "xp"))
        .transpose()
        .map(|xp| xp.unwrap_or(0))
        .map_err(|e| AppError::Database(format!("Failed to read badge XP: {e}")))
}
