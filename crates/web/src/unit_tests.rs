//! Web unit tests: pure logic, no browser needed.
//! Run natively: `cargo test -p web --lib` (no wasm target required for
//! these — they avoid web-sys/localStorage). Browser-bound logic stays in
//! component/integration layers.

use std::collections::HashSet;

use crate::i18n::{entry_count, Lang};
use crate::storage::OfflineChapterRecord;
use shared::ApiResponse;

/// Every dictionary key renders non-empty in BOTH languages and the key
/// set is identical, so a half-translated string can never ship.
#[test]
fn test_i18n_dictionary_complete_in_both_langs() {
    for lang in [Lang::Id, Lang::En] {
        let dict = lang.dict();
        assert_eq!(dict.len(), entry_count(), "duplicate dictionary key");
        for (k, v) in &dict {
            assert!(!v.trim().is_empty(), "key {k} is empty for {}", lang.code());
        }
    }
    let id_keys: HashSet<_> = Lang::Id.dict().keys().cloned().collect();
    let en_keys: HashSet<_> = Lang::En.dict().keys().cloned().collect();
    assert_eq!(id_keys, en_keys, "ID and EN must share the key set");
    // Unknown keys echo themselves so a typo is visible on screen.
    assert_eq!(Lang::En.text("no_such_key"), "no_such_key");
}

#[test]
fn test_i18n_placeholders_and_durations() {
    assert_eq!(
        Lang::En.text_with("page_of", "a", "4").replace("{b}", "18"),
        "Page 4 of 18"
    );
    assert_eq!(Lang::Id.duration(130), "~2 jam 10 mnt");
    assert_eq!(Lang::En.duration(45), "~45 min");
    assert_eq!(Lang::En.duration(120), "~2 h");
    let october = chrono::DateTime::parse_from_rfc3339("2026-10-08T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    assert_eq!(Lang::Id.month_year(october), "Okt 2026");
    assert_eq!(Lang::En.date_short(october), "8 Oct 2026");
    assert_eq!(Lang::Id.month_short(0), "Jan");
}

/// `Lang::toggle` flips, and the browser language picks the default.
#[test]
fn test_lang_toggle_and_browser_default() {
    assert_eq!(Lang::Id.toggle(), Lang::En);
    assert_eq!(Lang::En.toggle(), Lang::Id);
    assert_eq!(Lang::from_browser(Some("id-ID")), Lang::Id);
    assert_eq!(Lang::from_browser(Some("en-US")), Lang::En);
    assert_eq!(Lang::from_browser(None), Lang::En);
}

/// Reader preferences clamp to the supported range when loaded from a
/// stale or hand-edited store.
#[test]
fn test_reader_prefs_clamp() {
    use crate::theme::{ReaderPrefs, ReadingTheme, FONT_SIZE_MAX, LINE_HEIGHT_MIN};
    let prefs = ReaderPrefs {
        theme: ReadingTheme::Sepia,
        font_size: 99,
        line_height: 1,
    }
    .clamped();
    assert_eq!(prefs.font_size, FONT_SIZE_MAX);
    assert_eq!(prefs.line_height, LINE_HEIGHT_MIN);
    assert_eq!(prefs.line_height_css(), "1.5");
}

/// Typed offline chapter records survive a JSON round-trip with exact
/// keyPath fields.
#[test]
fn test_offline_chapter_record_round_trip() {
    let rec = OfflineChapterRecord {
        chapter_id: uuid::Uuid::new_v4(),
        book_id: uuid::Uuid::new_v4(),
        chapter_number: 3,
        title: "Chapter Three".to_string(),
        html_content: "<p>hello</p>".to_string(),
    };
    let json = serde_json::to_value(&rec).unwrap();
    for field in [
        "chapter_id",
        "book_id",
        "chapter_number",
        "title",
        "html_content",
    ] {
        assert!(json.get(field).is_some(), "missing keyPath field {field}");
    }
    let back: OfflineChapterRecord = serde_json::from_value(json).unwrap();
    assert_eq!(back.chapter_id, rec.chapter_id);
    assert_eq!(back.chapter_number, 3);
}

/// `ApiResponse` envelope: success carries data without error key;
/// failure carries code+message (mirrors SRS §2 wire contract).
#[test]
fn test_api_envelope_shapes() {
    let ok: ApiResponse<String> = ApiResponse::success("x".to_string());
    let v = serde_json::to_value(&ok).unwrap();
    assert_eq!(v["success"], true);
    assert_eq!(v["data"], "x");
    assert!(v.get("error").is_none());

    let err: ApiResponse<String> =
        ApiResponse::error("VALIDATION_FAILED", "Invalid email format", None);
    let v = serde_json::to_value(&err).unwrap();
    assert_eq!(v["success"], false);
    assert_eq!(v["error"]["code"], "VALIDATION_FAILED");
}
