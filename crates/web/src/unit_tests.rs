//! Web unit tests (Task 11e): pure logic, no browser needed.
//! Run natively: `cargo test -p web --lib` (no wasm target required for
//! these — they avoid web-sys/localStorage). Browser-bound logic stays in
//! component/integration layers.

use std::collections::HashSet;

use crate::i18n::Lang;
use crate::storage::OfflineChapterRecord;
use shared::ApiResponse;

/// The 18-key dictionary contract: every key renders non-empty in BOTH
/// languages (guards the 19-key drift that once shipped).
#[test]
fn test_i18n_dictionary_has_18_nonempty_keys_both_langs() {
    for lang in [Lang::Id, Lang::En] {
        let dict = lang.dict();
        assert_eq!(dict.len(), 18, "dictionary must hold exactly 18 keys");
        for (k, v) in &dict {
            assert!(
                !v.trim().is_empty(),
                "key {k} is empty for {:?}",
                lang.code()
            );
        }
    }
    let id_keys: HashSet<_> = Lang::Id.dict().keys().cloned().collect();
    let en_keys: HashSet<_> = Lang::En.dict().keys().cloned().collect();
    assert_eq!(id_keys, en_keys, "ID and EN must share the key set");
}

/// `Lang::toggle` flips (guards the toggle-buta bug: EN click became ID).
#[test]
fn test_lang_toggle_flips() {
    assert_eq!(Lang::Id.toggle(), Lang::En);
    assert_eq!(Lang::En.toggle(), Lang::Id);
}

/// Typed offline chapter records survive a JSON round-trip with exact
/// keyPath fields (guards the TD-06 untyped-Value failure mode).
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
