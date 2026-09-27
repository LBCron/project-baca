//! Auth, RBAC, and rate-limit middleware.

pub mod ai_rate_limit;
pub mod auth;
pub mod rate_limit;
pub mod role;

pub use ai_rate_limit::ai_rate_limit_middleware;
pub use auth::AuthUser;
pub use rate_limit::rate_limit_middleware;
pub use role::require_admin;
