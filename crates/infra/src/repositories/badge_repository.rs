//! Master badges, unlocks, and default seeder.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use shared::{AppError, BadgeDto, UserBadgeDto};
use uuid::Uuid;

use crate::entities::{badges, user_badges};

/// Lists all available master badges ordered by XP reward.
pub async fn list_badges(db: &DatabaseConnection) -> Result<Vec<BadgeDto>, AppError> {
    seed_default_badges_if_empty(db).await?;

    let badge_records = badges::Entity::find()
        .order_by_asc(badges::Column::XpReward)
        .all(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query badges: {e}")))?;

    Ok(badge_records
        .into_iter()
        .map(|b| BadgeDto {
            id: b.id,
            title_id: b.title_id,
            title_en: b.title_en,
            description_id: b.description_id,
            description_en: b.description_en,
            icon_asset: b.icon_asset,
            xp_reward: b.xp_reward,
        })
        .collect())
}

/// Lists all badges unlocked by a specific user.
pub async fn list_user_badges(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Vec<UserBadgeDto>, AppError> {
    let user_badge_records = user_badges::Entity::find()
        .filter(user_badges::Column::UserId.eq(user_id))
        .find_also_related(badges::Entity)
        .order_by_desc(user_badges::Column::UnlockedAt)
        .all(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query user badges: {e}")))?;

    let mut result = Vec::new();
    for (ub, maybe_badge) in user_badge_records {
        if let Some(b) = maybe_badge {
            result.push(UserBadgeDto {
                id: ub.id,
                badge_id: ub.badge_id,
                unlocked_at: ub.unlocked_at.with_timezone(&Utc),
                badge: BadgeDto {
                    id: b.id,
                    title_id: b.title_id,
                    title_en: b.title_en,
                    description_id: b.description_id,
                    description_en: b.description_en,
                    icon_asset: b.icon_asset,
                    xp_reward: b.xp_reward,
                },
            });
        }
    }

    Ok(result)
}

/// Seeds master badges if the badges table is currently empty.
pub async fn seed_default_badges_if_empty(db: &DatabaseConnection) -> Result<(), AppError> {
    let count = badges::Entity::find()
        .count(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to count badges: {e}")))?;

    if count > 0 {
        return Ok(());
    }

    let now = Utc::now();
    let seed_data = [
        (
            "first_step",
            "Langkah Pertama",
            "First Step",
            "Menyelesaikan sesi membaca 1 menit pertama.",
            "Read for your first 60 seconds.",
            "badge_first_step.svg",
            50,
        ),
        (
            "streak_3_days",
            "Pembaca Konsisten",
            "Consistent Reader",
            "Membaca 3 hari berturut-turut tanpa jeda.",
            "Maintain a 3-day consecutive reading streak.",
            "badge_streak_3.svg",
            100,
        ),
        (
            "streak_7_days",
            "Dedikasi Sastra",
            "Literary Devotion",
            "Membaca 7 hari berturut-turut.",
            "Maintain a 7-day consecutive reading streak.",
            "badge_streak_7.svg",
            250,
        ),
        (
            "dedicated_reader",
            "Kutu Buku Klasik",
            "Classic Bibliophile",
            "Membaca total lebih dari 1 jam di perpustakaan.",
            "Accumulate over 1 hour of total reading time.",
            "badge_hour_1.svg",
            150,
        ),
        (
            "polyglot",
            "Pembaca Nusantara",
            "Archipelago Reader",
            "Membaca karya sastra dalam ragam bahasa.",
            "Read literature across multiple languages.",
            "badge_polyglot.svg",
            200,
        ),
    ];

    for (id, t_id, t_en, d_id, d_en, icon, xp) in seed_data {
        let badge = badges::ActiveModel {
            id: Set(id.to_string()),
            title_id: Set(t_id.to_string()),
            title_en: Set(t_en.to_string()),
            description_id: Set(d_id.to_string()),
            description_en: Set(d_en.to_string()),
            icon_asset: Set(icon.to_string()),
            xp_reward: Set(xp),
            created_at: Set(now.into()),
        };
        let _ = badge.insert(db).await;
    }

    Ok(())
}
