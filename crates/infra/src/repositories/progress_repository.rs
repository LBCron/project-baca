//! Progress, CFI anchors, heartbeats, and streaks.

use chrono::Utc;
use sea_orm::entity::prelude::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder, Set, Statement, Value,
};
use shared::{
    ActiveProgressDto, AppError, ReadingHeartbeatRequest, ReadingHeartbeatResponse,
    UpdateProgressRequest,
};
use uuid::Uuid;

use crate::entities::{
    books, chapters, reading_activity_logs, user_badges, user_reading_progress,
    user_reading_streaks,
};

/// Retrieves the latest unfinished reading progress for a user.
pub async fn get_active_progress(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Option<ActiveProgressDto>, AppError> {
    let progress = user_reading_progress::Entity::find()
        .filter(user_reading_progress::Column::UserId.eq(user_id))
        .filter(user_reading_progress::Column::IsFinished.eq(false))
        .order_by_desc(user_reading_progress::Column::LastReadAt)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to retrieve active progress: {e}")))?;

    let progress = match progress {
        Some(p) => p,
        None => return Ok(None),
    };

    let book = books::Entity::find_by_id(progress.book_id)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query book for progress: {e}")))?
        .ok_or_else(|| AppError::NotFound(format!("Book not found: {}", progress.book_id)))?;

    let chapter = chapters::Entity::find_by_id(progress.last_chapter_id)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query chapter for progress: {e}")))?
        .ok_or_else(|| {
            AppError::NotFound(format!("Chapter not found: {}", progress.last_chapter_id))
        })?;

    let completion_percentage: f32 = progress
        .completion_percentage
        .to_string()
        .parse()
        .unwrap_or(0.0);

    Ok(Some(ActiveProgressDto {
        book_id: book.id,
        book_title: book.title,
        book_author: book.author,
        book_cover_url: book.cover_url,
        chapter_id: chapter.id,
        chapter_number: chapter.chapter_number,
        chapter_title: chapter.title,
        last_anchor_cfi: progress.last_anchor_cfi,
        completion_percentage,
        is_finished: progress.is_finished,
        last_read_at: progress.last_read_at.with_timezone(&Utc),
    }))
}

/// Upserts user reading progress with CFI position and completion percentage.
pub async fn update_progress(
    db: &DatabaseConnection,
    user_id: Uuid,
    book_id: Uuid,
    req: &UpdateProgressRequest,
) -> Result<(), AppError> {
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
    let today = now.date_naive();

    // Log activity in reading_activity_logs
    let log_entry = reading_activity_logs::ActiveModel {
        user_id: Set(user_id),
        book_id: Set(req.book_id),
        seconds_spent: Set(req.seconds_spent),
        activity_date: Set(today),
        created_at: Set(now.into()),
        ..Default::default()
    };

    log_entry
        .insert(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to record activity log: {e}")))?;

    // Fetch total reading seconds for user today
    let daily_sum_sql = r#"
        SELECT COALESCE(SUM(seconds_spent), 0)::bigint AS total_today
        FROM reading_activity_logs
        WHERE user_id = $1 AND activity_date = $2;
    "#;

    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        daily_sum_sql,
        vec![Value::from(user_id), Value::from(today)],
    );

    let query_result = db
        .query_one(stmt)
        .await
        .map_err(|e| AppError::Database(format!("Failed to aggregate daily reading time: {e}")))?
        .ok_or_else(|| AppError::Internal("Failed to calculate daily reading time".to_string()))?;

    let total_today_seconds: i64 = query_result
        .try_get_by_index(0)
        .map_err(|e| AppError::Database(format!("Failed to parse daily reading sum: {e}")))?;

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
    let total_reading_seconds = streak_record
        .as_ref()
        .map(|s| s.total_reading_seconds)
        .unwrap_or(0)
        + req.seconds_spent as i64;
    let mut total_xp = streak_record.as_ref().map(|s| s.total_xp).unwrap_or(0);
    let last_active_date = streak_record.as_ref().and_then(|s| s.last_activity_date);

    let mut xp_earned = domain::BASE_HEARTBEAT_XP;

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
    let streak_incremented = outcome.incremented;
    xp_earned += outcome.bonus_xp;

    total_xp += xp_earned;

    let updated_last_date = outcome.last_active;

    match streak_record {
        Some(existing) => {
            let mut active: user_reading_streaks::ActiveModel = existing.into();
            active.current_streak_days = Set(current_streak);
            active.longest_streak_days = Set(longest_streak);
            active.total_reading_seconds = Set(total_reading_seconds);
            active.total_xp = Set(total_xp);
            active.last_activity_date = Set(updated_last_date);
            active.updated_at = Set(now.into());
            active
                .update(db)
                .await
                .map_err(|e| AppError::Database(format!("Failed to update streak record: {e}")))?;
        }
        None => {
            let new_streak = user_reading_streaks::ActiveModel {
                user_id: Set(user_id),
                current_streak_days: Set(current_streak),
                longest_streak_days: Set(longest_streak),
                total_reading_seconds: Set(total_reading_seconds),
                total_xp: Set(total_xp),
                last_activity_date: Set(updated_last_date),
                updated_at: Set(now.into()),
            };
            new_streak
                .insert(db)
                .await
                .map_err(|e| AppError::Database(format!("Failed to insert streak record: {e}")))?;
        }
    }

    // Milestone badge evaluation
    check_and_award_badges(db, user_id, current_streak, total_reading_seconds).await?;

    Ok(ReadingHeartbeatResponse {
        current_streak_days: current_streak,
        longest_streak_days: longest_streak,
        total_reading_seconds,
        total_xp,
        xp_earned,
        streak_incremented,
    })
}

/// Evaluates and unlocks achievement badges for reading milestones.
async fn check_and_award_badges(
    db: &DatabaseConnection,
    user_id: Uuid,
    streak_days: i32,
    total_seconds: i64,
) -> Result<(), AppError> {
    // Domain-owned eligibility matrix (ids must exist in `badges` seed data).
    let eligible = domain::eligible_badges(streak_days, total_seconds);
    if eligible.is_empty() {
        return Ok(());
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
        return Ok(());
    }

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
    let sql = format!(
        "INSERT INTO user_badges (id, user_id, badge_id, unlocked_at) \
         VALUES {values_sql} ON CONFLICT (user_id, badge_id) DO NOTHING"
    );

    db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        values,
    ))
    .await
    .map_err(|e| AppError::Database(format!("Failed to award badges: {e}")))?;

    Ok(())
}
