//! Auth, RBAC, and rate-limit middleware.

pub mod auth;
pub mod rate_limit;
pub mod role;

pub use auth::AuthUser;
pub use rate_limit::{
    ai_rate_limit_middleware, public_rate_limit_middleware, rate_limit_middleware,
};
pub use role::require_admin;
