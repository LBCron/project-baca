//! Transient notices, provided once by the shell and pushed from anywhere.

use leptos::prelude::*;
use std::time::Duration;

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

    pub fn push(&self, message: impl Into<String>, is_error: bool) {
        let id = self.next_id.get_untracked();
        self.next_id.set(id + 1);
        self.items.update(|items| {
            items.push(Toast {
                id,
                message: message.into(),
                is_error,
            })
        });
        let items = self.items;
        set_timeout(
            move || items.update(|all| all.retain(|t| t.id != id)),
            Duration::from_millis(2800),
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
                <div class="toast" class:error=toast.is_error role="status">{toast.message}</div>
            </For>
        </div>
    }
}
