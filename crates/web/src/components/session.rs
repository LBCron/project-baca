//! Signed-in state shared by the shell, pages, and the auth sheet.

use crate::api;
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::UserProfileDto;

#[derive(Clone, Copy)]
pub struct Session {
    pub authed: RwSignal<bool>,
    pub profile: RwSignal<Option<UserProfileDto>>,
    pub sheet_open: RwSignal<bool>,
}

impl Session {
    pub fn provide() -> Self {
        let session = Self {
            authed: RwSignal::new(api::is_authed()),
            profile: RwSignal::new(None),
            sheet_open: RwSignal::new(false),
        };
        provide_context(session);
        session.refresh();
        session
    }

    /// Re-reads the token and, when present, the profile behind it. A dead
    /// token is cleared by the API layer, which flips `authed` back off.
    pub fn refresh(&self) {
        let authed = self.authed;
        let profile = self.profile;
        if !api::is_authed() {
            authed.set(false);
            profile.set(None);
            return;
        }
        authed.set(true);
        spawn_local(async move {
            match api::me().await {
                Ok(me) => profile.set(Some(me)),
                Err(_) => {
                    profile.set(None);
                    authed.set(api::is_authed());
                }
            }
        });
    }

    pub fn is_admin(&self) -> bool {
        self.profile
            .get()
            .map(|p| p.role == "admin")
            .unwrap_or(false)
    }

    pub fn initial(&self) -> String {
        self.profile
            .get()
            .and_then(|p| p.display_name.chars().next())
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "R".to_string())
    }

    pub fn open_sheet(&self) {
        self.sheet_open.set(true);
    }

    pub fn sign_out(&self) {
        let session = *self;
        spawn_local(async move {
            let _ = api::logout().await;
            session.profile.set(None);
            session.authed.set(false);
        });
    }
}

/// Session from context; a detached one outside the shell (gallery, tests).
pub fn use_session() -> Session {
    use_context::<Session>().unwrap_or_else(|| Session {
        authed: RwSignal::new(api::is_authed()),
        profile: RwSignal::new(None),
        sheet_open: RwSignal::new(false),
    })
}
