//! Local-first persistence via IndexedDB (`rexie`).
//! Database `project_baca_db` with stores for guest progress, offline books,
//! offline chapters, and the pending sync queue.

use rexie::{ObjectStore, Rexie, TransactionMode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::JsValue;

const DB_NAME: &str = "project_baca_db";
const DB_VERSION: u32 = 1;

pub const STORE_GUEST_PROGRESS: &str = "guest_progress";
pub const STORE_OFFLINE_BOOKS: &str = "offline_books";
pub const STORE_OFFLINE_CHAPTERS: &str = "offline_chapters";
pub const STORE_PENDING_SYNC: &str = "pending_sync_queue";

async fn db() -> Result<Rexie, String> {
    Rexie::builder(DB_NAME)
        .version(DB_VERSION)
        .add_object_store(
            ObjectStore::new(STORE_GUEST_PROGRESS).key_path("book_id"),
        )
        .add_object_store(ObjectStore::new(STORE_OFFLINE_BOOKS).key_path("book_id"))
        .add_object_store(
            ObjectStore::new(STORE_OFFLINE_CHAPTERS).key_path("chapter_id"),
        )
        .add_object_store(ObjectStore::new(STORE_PENDING_SYNC).auto_increment(true))
        .build()
        .await
        .map_err(|e| format!("{e:?}"))
}

pub async fn put<T: Serialize>(store: &str, value: &T) -> Result<(), String> {
    let rexie = db().await?;
    let tx = rexie
        .transaction(&[store], TransactionMode::ReadWrite)
        .map_err(|e| format!("{e:?}"))?;
    let js = serde_wasm_bindgen::to_value(value).map_err(|e| e.to_string())?;
    tx.store(store)
        .map_err(|e| format!("{e:?}"))?
        .put(&js, None)
        .await
        .map_err(|e| format!("{e:?}"))?;
    tx.done().await.map_err(|e| format!("{e:?}"))?;
    Ok(())
}

pub async fn get_all<T: DeserializeOwned>(store: &str) -> Result<Vec<T>, String> {
    let rexie = db().await?;
    let tx = rexie
        .transaction(&[store], TransactionMode::ReadOnly)
        .map_err(|e| format!("{e:?}"))?;
    let items: Vec<JsValue> = tx
        .store(store)
        .map_err(|e| format!("{e:?}"))?
        .get_all(None, None)
        .await
        .map_err(|e| format!("{e:?}"))?;
    tx.done().await.map_err(|e| format!("{e:?}"))?;
    items
        .into_iter()
        .map(|v| serde_wasm_bindgen::from_value(v).map_err(|e| e.to_string()))
        .collect()
}

pub async fn clear(store: &str) -> Result<(), String> {
    let rexie = db().await?;
    let tx = rexie
        .transaction(&[store], TransactionMode::ReadWrite)
        .map_err(|e| format!("{e:?}"))?;
    tx.store(store)
        .map_err(|e| format!("{e:?}"))?
        .clear()
        .await
        .map_err(|e| format!("{e:?}"))?;
    tx.done().await.map_err(|e| format!("{e:?}"))?;
    Ok(())
}
