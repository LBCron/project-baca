//! Inline stroke icons (24-unit grid). Decorative by default; callers add
//! `aria-label` on the surrounding control when the icon stands alone.

use leptos::prelude::*;

fn icon(path: &'static str, filled: bool) -> impl IntoView {
    let (fill, stroke) = if filled {
        ("currentColor", "none")
    } else {
        ("none", "currentColor")
    };
    view! {
        <svg
            viewBox="0 0 24 24"
            fill=fill
            stroke=stroke
            stroke-width="1.9"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
            focusable="false"
        >
            <path d=path/>
        </svg>
    }
}

/// Simplified watermill barn: seven strokes that stay legible at 24 px,
/// drawn in the current text color with a terracotta hub.
pub fn brand_mark() -> impl IntoView {
    view! {
        <svg
            class="brand-mark"
            viewBox="0 0 48 48"
            fill="none"
            stroke="currentColor"
            stroke-width="2.6"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
            focusable="false"
        >
            <path d="M6 22 24 9l18 13"/>
            <path d="M11 22v17M37 22v17M11 22h26"/>
            <circle cx="24" cy="31" r="8.5" stroke-width="2.4"/>
            <path d="M24 22.5v17M15.5 31h17M18 25l12 12M30 25 18 37" stroke-width="1.9"/>
            <path d="M4 44c3.3-2.4 6.7-2.4 10 0s6.7 2.4 10 0 6.7-2.4 10 0 6.7 2.4 10 0" stroke-width="2.2"/>
            <circle cx="24" cy="31" r="2.8" fill="var(--accent)" stroke="none"/>
        </svg>
    }
}

pub fn arrow_down() -> impl IntoView {
    icon("M12 5v14m-6-6 6 6 6-6", false)
}

pub fn home() -> impl IntoView {
    icon(
        "M3 11 12 4l9 7v9a1 1 0 0 1-1 1h-5v-6H9v6H4a1 1 0 0 1-1-1z",
        false,
    )
}

pub fn search() -> impl IntoView {
    icon("M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zm9 16-3.5-3.5", false)
}

pub fn shelf() -> impl IntoView {
    icon(
        "M4 4h4v16H4zM10 4h4v16h-4zM16.5 5l3.5 1-3.2 14-3.5-1z",
        false,
    )
}

pub fn user() -> impl IntoView {
    icon(
        "M12 4a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM4 21a8 8 0 0 1 16 0",
        false,
    )
}

pub fn moon() -> impl IntoView {
    icon("M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z", false)
}

pub fn sun() -> impl IntoView {
    icon(
        "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4",
        false,
    )
}

pub fn back() -> impl IntoView {
    icon("m15 5-7 7 7 7", false)
}

pub fn download() -> impl IntoView {
    icon("M12 4v11m0 0 4-4m-4 4-4-4M4 19h16", false)
}

pub fn share() -> impl IntoView {
    icon(
        "M12 15V4m0 0 4 4m-4-4L8 8M5 13v6a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-6",
        false,
    )
}

pub fn play() -> impl IntoView {
    icon("M8 5v14l11-7z", true)
}

pub fn check() -> impl IntoView {
    icon("m5 12 5 5L20 7", false)
}

pub fn cards() -> impl IntoView {
    icon(
        "M3 7a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2zM7 9h10M7 13h6",
        false,
    )
}

pub fn recap() -> impl IntoView {
    icon("M3 12a9 9 0 1 0 3-6.7M3 4v5h5", false)
}

pub fn flame() -> impl IntoView {
    icon(
        "M12 3c1 3 4 4.5 4 8.5A4 4 0 0 1 8 11c0-1 .3-2 .8-2.8C9.5 10 10.5 10.5 11 10c-.6-2 0-5 1-7z",
        false,
    )
}

pub fn settings() -> impl IntoView {
    icon(
        "M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6zM19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z",
        false,
    )
}

pub fn close() -> impl IntoView {
    icon("M6 6l12 12M18 6 6 18", false)
}

pub fn arrow_right() -> impl IntoView {
    icon("M5 12h14m-6-6 6 6-6 6", false)
}

pub fn upload() -> impl IntoView {
    icon("M12 16V5m0 0-4 4m4-4 4 4M4 19h16", false)
}

pub fn lines(spacing: u8) -> impl IntoView {
    let gap = match spacing {
        0 => "M4 8h16M4 12h16M4 16h16",
        1 => "M4 6h16M4 12h16M4 18h16",
        _ => "M4 5h16M4 12h16M4 19h16",
    };
    icon(gap, false)
}
