//! ID/EN dictionary. Youth register: "kamu", never "Anda".

use std::collections::HashMap;

const LANG_KEY: &str = "rotaria_lang";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Id,
    En,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::Id => "ID",
            Lang::En => "EN",
        }
    }

    pub fn toggle(self) -> Lang {
        match self {
            Lang::Id => Lang::En,
            Lang::En => Lang::Id,
        }
    }

    pub fn load() -> Lang {
        let lang = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item(LANG_KEY).ok().flatten())
            .unwrap_or_default();
        if lang == "ID" {
            Lang::Id
        } else {
            Lang::En
        }
    }

    pub fn save(self) {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item(LANG_KEY, self.code());
        }
    }

    pub fn text(self, key: &str) -> String {
        let en = self == Lang::En;
        match key {
            "tagline" => {
                if en {
                    "The circulation of great books.".into()
                } else {
                    "Sirkulasi buku-buku hebat.".into()
                }
            }
            "hero_sub" => {
                if en {
                    "Public domain classics, bound anew.".into()
                } else {
                    "Klasik domain publik, dijilid ulang untukmu.".into()
                }
            }
            "search_ph" => {
                if en {
                    "Search title or author…".into()
                } else {
                    "Cari judul atau penulis…".into()
                }
            }
            "catalog" => {
                if en {
                    "Catalog".into()
                } else {
                    "Katalog".into()
                }
            }
            "continue_reading" => {
                if en {
                    "Continue Reading".into()
                } else {
                    "Lanjut Baca".into()
                }
            }
            "resume" => {
                if en {
                    "Resume".into()
                } else {
                    "Lanjutkan".into()
                }
            }
            "read" => {
                if en {
                    "Read".into()
                } else {
                    "Baca".into()
                }
            }
            "load_more" => {
                if en {
                    "Load More".into()
                } else {
                    "Muat Lagi".into()
                }
            }
            "loading" => {
                if en {
                    "Loading…".into()
                } else {
                    "Memuat…".into()
                }
            }
            "sign_in" => {
                if en {
                    "Sign In".into()
                } else {
                    "Masuk".into()
                }
            }
            "chapters" => {
                if en {
                    "Chapters".into()
                } else {
                    "Bab".into()
                }
            }
            "start_reading" => {
                if en {
                    "Start Reading".into()
                } else {
                    "Mulai Baca".into()
                }
            }
            "save_offline" => {
                if en {
                    "Save Offline".into()
                } else {
                    "Simpan Offline".into()
                }
            }
            "saved_offline" => {
                if en {
                    "Saved Offline".into()
                } else {
                    "Tersimpan".into()
                }
            }
            "saving" => {
                if en {
                    "Saving…".into()
                } else {
                    "Menyimpan…".into()
                }
            }
            _ => key.to_string(),
        }
    }

    pub fn dict(self) -> HashMap<&'static str, String> {
        let keys = [
            "tagline",
            "hero_sub",
            "search_ph",
            "catalog",
            "continue_reading",
            "resume",
            "read",
            "load_more",
            "loading",
            "sign_in",
            "chapters",
            "start_reading",
            "save_offline",
            "saved_offline",
            "saving",
        ];
        keys.into_iter().map(|k| (k, self.text(k))).collect()
    }

    pub fn document_lang(self) {
        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document() {
                if let Some(el) = doc.document_element() {
                    let _ = el.set_attribute("lang", if self == Lang::Id { "id" } else { "en" });
                }
            }
        }
    }
}

pub fn lang_signal() -> (
    leptos::prelude::ReadSignal<Lang>,
    leptos::prelude::WriteSignal<Lang>,
) {
    use leptos::prelude::*;
    let lang = Lang::load();
    lang.document_lang();
    let (l, s) = signal(lang);
    (l, s)
}

pub fn apply_lang(set: leptos::prelude::WriteSignal<Lang>, lang: Lang) {
    use leptos::prelude::*;
    set.set(lang);
    lang.save();
    lang.document_lang();
}
