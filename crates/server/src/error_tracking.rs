//! Sentry for panics and server errors, without personal data. Every event
//! and breadcrumb passes one scrubber: the request keeps its method and its
//! path without the query, never headers, cookies or body; text that looks
//! like an email address or a JWT is masked wherever it appears.

use sentry::integrations::tracing::{BreadcrumbLayer, ErrorLayer};
use sentry::protocol::{Breadcrumb, Context, Event, Map, Value};
use std::sync::Arc;
use tracing::{Level, Subscriber};
use tracing_subscriber::filter::{filter_fn, LevelFilter};
use tracing_subscriber::{registry::LookupSpan, Layer};

const REDACTED: &str = "[redacted]";

/// Client options for `sentry::init`: release `server@<crate version>`,
/// environment from `APP_ENV`, no default PII, and the scrubber on events
/// and breadcrumbs. The DSN is set by the caller.
pub fn options(environment: &str) -> sentry::ClientOptions {
    let mut options = sentry::ClientOptions::new();
    options.release = sentry::release_name!();
    options.environment = Some(environment.to_string().into());
    options.send_default_pii = false;
    options.before_send = Some(Arc::new(|event| Some(scrub_event(event))));
    options.before_breadcrumb = Some(Arc::new(|crumb| Some(scrub_breadcrumb(crumb))));
    options
}

/// Tracing into Sentry: an `error` line becomes an event, `info` and `warn`
/// lines become its breadcrumbs, `debug` and `trace` stay local. Inert
/// until `sentry::init` binds a client.
pub fn tracing_layer<S>() -> Vec<Box<dyn Layer<S> + Send + Sync>>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    // Two per-layer filters side by side; joined with `and_then` the outer
    // filter alone decided and no breadcrumb was ever recorded.
    vec![
        BreadcrumbLayer::new()
            .with_filter(filter_fn(|metadata| {
                matches!(*metadata.level(), Level::WARN | Level::INFO)
            }))
            .boxed(),
        ErrorLayer::new().with_filter(LevelFilter::ERROR).boxed(),
    ]
}

fn sensitive(word: &str) -> bool {
    word.contains('@') || word.contains("%40") || word.contains("eyJ")
}

/// Masks every whitespace-separated word that holds an email address
/// (raw or percent-encoded) or a JWT, whose segments start with `eyJ`.
fn redact(text: &str) -> String {
    text.split(' ')
        .map(|word| if sensitive(word) { REDACTED } else { word })
        .collect::<Vec<_>>()
        .join(" ")
}

fn redact_value(value: &mut Value) {
    match value {
        Value::String(text) => *text = redact(text),
        Value::Array(items) => items.iter_mut().for_each(redact_value),
        Value::Object(fields) => fields.values_mut().for_each(redact_value),
        _ => {}
    }
}

fn redact_map(map: &mut Map<String, Value>) {
    map.values_mut().for_each(redact_value);
}

fn scrub_breadcrumb(mut crumb: Breadcrumb) -> Breadcrumb {
    crumb.message = crumb.message.as_deref().map(redact);
    redact_map(&mut crumb.data);
    crumb
}

fn scrub_event(mut event: Event<'static>) -> Event<'static> {
    if let Some(request) = event.request.as_mut() {
        request.data = None;
        request.query_string = None;
        request.cookies = None;
        request.headers.clear();
        request.env.clear();
        if let Some(url) = request.url.as_mut() {
            url.set_query(None);
            url.set_fragment(None);
        }
    }
    event.user = None;
    event.message = event.message.as_deref().map(redact);
    for exception in &mut event.exception.values {
        exception.value = exception.value.as_deref().map(redact);
    }
    event.breadcrumbs.values = std::mem::take(&mut event.breadcrumbs.values)
        .into_iter()
        .map(scrub_breadcrumb)
        .collect();
    redact_map(&mut event.extra);
    for context in event.contexts.values_mut() {
        if let Context::Other(fields) = context {
            redact_map(fields);
        }
    }
    event
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_masks_addresses_and_jwts_only() {
        assert_eq!(
            redact("login for reader@example.com with Bearer eyJhbGciOi.x.y failed"),
            "login for [redacted] with Bearer [redacted] failed"
        );
        assert_eq!(redact("q=reader%40example.com"), REDACTED);
        assert_eq!(redact("Database error: timeout"), "Database error: timeout");
    }

    #[test]
    fn scrubbed_request_keeps_method_and_path_only() {
        let mut event = Event::new();
        event.request = Some(sentry::protocol::Request {
            url: "https://rotaria.site/api/v1/me?email=a%40b.c#frag"
                .parse()
                .ok(),
            method: Some("PATCH".into()),
            data: Some("{\"password\":\"x\"}".into()),
            query_string: Some("email=a%40b.c".into()),
            cookies: Some("s=1".into()),
            headers: [("Authorization".to_string(), "Bearer t".to_string())].into(),
            ..Default::default()
        });
        let request = scrub_event(event).request.unwrap();
        assert_eq!(
            request.url.unwrap().as_str(),
            "https://rotaria.site/api/v1/me"
        );
        assert_eq!(request.method.as_deref(), Some("PATCH"));
        assert!(request.data.is_none() && request.query_string.is_none());
        assert!(request.cookies.is_none() && request.headers.is_empty());
    }
}
