//! Public pages as plain HTML, so crawlers and readers without JavaScript get
//! the content in the first response. Pages are rendered per request from the
//! live catalog, so an archived book is gone on the next request.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use shared::AppError;
use uuid::Uuid;

use crate::{error::HttpError, AppState};

pub fn pages_routes() -> Router<Arc<AppState>> {
    Router::new().route("/book/{id}", get(book_page))
}

pub async fn book_page(State(state): State<Arc<AppState>>, Path(book_id): Path<Uuid>) -> Response {
    match infra::get_book_by_id(&state.db, book_id).await {
        Ok(book) => {
            let year = book
                .publication_year
                .map(|y| format!(" ({y})"))
                .unwrap_or_default();
            let body = format!(
                "<main><h1>{}</h1><p>{}{}</p><p>{}</p></main>",
                escape_html(&book.title),
                escape_html(&book.author),
                year,
                escape_html(&book.description),
            );
            page(
                StatusCode::OK,
                &book.language,
                &format!("{} - Rotaria", book.title),
                &book.description,
                &body,
            )
        }
        Err(AppError::NotFound(_)) => page(
            StatusCode::NOT_FOUND,
            "id",
            "Rotaria",
            "",
            "<main><h1>Buku tidak ditemukan</h1></main>",
        ),
        Err(e) => HttpError::from(e).into_response(),
    }
}

fn page(status: StatusCode, lang: &str, title: &str, description: &str, body: &str) -> Response {
    let html = format!(
        "<!doctype html><html lang=\"{}\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>{}</title><meta name=\"description\" content=\"{}\"></head>\
         <body>{}</body></html>",
        escape_html(lang),
        escape_html(title),
        escape_html(description),
        body,
    );
    // Revalidated on every request so a takedown is never served from a cache.
    (status, [(header::CACHE_CONTROL, "no-cache")], Html(html)).into_response()
}

pub(crate) fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::escape_html;

    #[test]
    fn escape_html_neutralises_markup_and_attribute_quotes() {
        assert_eq!(
            escape_html(r#"<a href="x">Tom & "Jerry"</a>"#),
            "&lt;a href=&quot;x&quot;&gt;Tom &amp; &quot;Jerry&quot;&lt;/a&gt;"
        );
    }
}
