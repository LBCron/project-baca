//! Transient notices, provided once by the shell and pushed from anywhere.

use leptos::prelude::*;
use std::time::Duration;

const INFO_MS: u64 = 2800;
const ERROR_MS: u64 = 5200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    pub id: u32,
    pub message: String,
    pub is_error: bool,
}

#[derive(Clone, Copy)]
pub struct Toasts {
    items: RwSignal<Vec<Toast>>,
    next_id: RwSignal<u32>,
}

impl Toasts {
    pub fn provide() -> Self {
        let toasts = Self {
            items: RwSignal::new(Vec::new()),
            next_id: RwSignal::new(1),
        };
        provide_context(toasts);
        toasts
    }

    /// Shows a notice; a message that is already on screen is not stacked
    /// again, so repeated taps at the end of a book yield one toast.
    pub fn push(&self, message: impl Into<String>, is_error: bool) {
        let message = message.into();
        if self
            .items
            .with_untracked(|items| items.iter().any(|t| t.message == message))
        {
            return;
        }
        let id = self.next_id.get_untracked();
        self.next_id.set(id + 1);
        self.items.update(|items| {
            items.push(Toast {
                id,
                message,
                is_error,
            })
        });
        let items = self.items;
        let shown_for = if is_error { ERROR_MS } else { INFO_MS };
        set_timeout(
            move || items.update(|all| all.retain(|t| t.id != id)),
            Duration::from_millis(shown_for),
        );
    }

    pub fn info(&self, message: impl Into<String>) {
        self.push(message, false);
    }

    pub fn error(&self, message: impl Into<String>) {
        self.push(message, true);
    }
}

/// Toasts from context; a no-op sink when rendered outside the shell (tests).
pub fn use_toasts() -> Toasts {
    use_context::<Toasts>().unwrap_or_else(|| Toasts {
        items: RwSignal::new(Vec::new()),
        next_id: RwSignal::new(1),
    })
}

#[component]
pub fn ToastStack(toasts: Toasts) -> impl IntoView {
    view! {
        <div class="toast-stack" aria-live="polite">
            <For each=move || toasts.items.get() key=|t| t.id let:toast>
                <div class="toast" class:error=toast.is_error role=if toast.is_error { "alert" } else { "status" }>{toast.message}</div>
            </For>
        </div>
    }
}
