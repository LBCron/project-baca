//! REST API route registration and sub-routers.

pub mod admin;
pub mod auth;
pub mod books;
pub mod covers;
pub mod gamification;
pub mod insights;
pub mod progress;
pub mod quotes;

pub use admin::{admin_routes, dropoff_analytics, ingestion_status, upload_book};
pub use auth::{auth_routes, user_routes};
pub use books::{
    books_routes, get_book, get_chapter, get_offline_bundle, list_books, search_books,
};
pub use covers::{covers_routes, get_cover};
pub use gamification::{
    gamification_routes, get_my_streak, list_badges, list_user_badges, record_heartbeat,
};
pub use insights::{get_atomic_cards, get_chapter_recap, insights_routes};
pub use progress::{get_active_progress, merge_guest_progress, progress_routes, update_progress};
pub use quotes::{
    get_quote_card, handle_list_saved_quotes, handle_save_quote, quotes_routes,
    saved_quotes_routes, search_book_quotes,
};
