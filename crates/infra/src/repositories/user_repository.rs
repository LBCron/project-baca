//! User accounts, credentials, and lifecycle queries.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    SqlErr,
};
use shared::AppError;
use uuid::Uuid;

use crate::entities::users;

/// Finds a user record by their unique email address.
pub async fn find_user_by_email(
    db: &DatabaseConnection,
    email: &str,
) -> Result<Option<users::Model>, AppError> {
    users::Entity::find()
        .filter(users::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query user by email: {e}")))
}

/// Finds a user record by their UUID primary key.
pub async fn find_user_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<users::Model>, AppError> {
    users::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query user by ID: {e}")))
}

/// Inserts a new inactive user awaiting email OTP verification.
pub async fn create_inactive_user(
    db: &DatabaseConnection,
    email: &str,
    display_name: &str,
    password_hash: &str,
) -> Result<users::Model, AppError> {
    let new_user = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(email.to_string()),
        display_name: Set(display_name.to_string()),
        password_hash: Set(Some(password_hash.to_string())),
        role: Set("reader".to_string()),
        avatar_url: Set(None),
        is_active: Set(false),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    new_user.insert(db).await.map_err(|e| match e.sql_err() {
        // Two signups for one address raced past the lookup; the loser
        // answers like any other taken address.
        Some(SqlErr::UniqueConstraintViolation(_)) => {
            AppError::Conflict("An account with this email is already registered".to_string())
        }
        _ => AppError::Database(format!("Failed to insert inactive user: {e}")),
    })
}

/// Creates the account a verified registration asked for, with that
/// registration's name and password. A row left inactive by an older signup
/// takes them over; an address already active is a conflict.
pub async fn activate_registration(
    db: &DatabaseConnection,
    email: &str,
    display_name: &str,
    password_hash: &str,
) -> Result<users::Model, AppError> {
    let existing = users::Entity::find()
        .filter(users::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query user for activation: {e}")))?;
    let mut user: users::ActiveModel = match existing {
        Some(user) if user.is_active => {
            return Err(AppError::Conflict(
                "An account with this email is already registered".to_string(),
            ))
        }
        Some(user) => user.into(),
        None => create_inactive_user(db, email, display_name, password_hash)
            .await?
            .into(),
    };
    user.display_name = Set(display_name.to_string());
    user.password_hash = Set(Some(password_hash.to_string()));
    user.is_active = Set(true);
    user.updated_at = Set(Utc::now().into());
    user.update(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to activate user: {e}")))
}

/// Updates user display name and/or avatar URL.
pub async fn update_user_profile(
    db: &DatabaseConnection,
    id: Uuid,
    display_name: Option<String>,
    avatar_url: Option<String>,
) -> Result<users::Model, AppError> {
    let user = users::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query user for profile update: {e}")))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let mut active: users::ActiveModel = user.into();
    if let Some(name) = display_name {
        active.display_name = Set(name);
    }
    if let Some(avatar) = avatar_url {
        active.avatar_url = Set(Some(avatar));
    }
    active.updated_at = Set(Utc::now().into());

    active
        .update(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to update user profile: {e}")))
}

/// Updates the password hash of an existing user.
pub async fn update_user_password(
    db: &DatabaseConnection,
    id: Uuid,
    new_password_hash: &str,
) -> Result<(), AppError> {
    let user = users::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query user for password update: {e}")))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let mut active: users::ActiveModel = user.into();
    active.password_hash = Set(Some(new_password_hash.to_string()));
    active.updated_at = Set(Utc::now().into());

    active
        .update(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to update user password: {e}")))?;

    Ok(())
}

/// Deletes a user by ID (triggers database cascade deletion).
pub async fn delete_user_by_id(db: &DatabaseConnection, id: Uuid) -> Result<(), AppError> {
    users::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to delete user: {e}")))?;

    Ok(())
}
