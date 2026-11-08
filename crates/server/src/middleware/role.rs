//! Role-Based Access Control (RBAC) authorization guards.

use super::auth::AuthUser;
use crate::AppState;
use infra::find_user_by_id;
use shared::AppError;

fn forbidden(user: &AuthUser) -> AppError {
    tracing::warn!(target: "server::admin", event = "admin.access.denied", user_id = %user.id, "Admin route refused");
    AppError::Forbidden("Administrator privileges required to perform this action".to_string())
}

/// Enforces the 'admin' role. The JWT claim is only a fast pre-check: the
/// current row is consulted so a demoted or deactivated admin loses access
/// immediately instead of after the token expires.
pub async fn require_admin(state: &AppState, user: &AuthUser) -> Result<(), AppError> {
    if user.role != "admin" {
        return Err(forbidden(user));
    }
    let current = find_user_by_id(&state.db, user.id)
        .await?
        .ok_or_else(|| forbidden(user))?;
    if !current.is_active || current.role != "admin" {
        return Err(forbidden(user));
    }
    Ok(())
}
