//! Progress ring and bar.

use leptos::prelude::*;

const RING_RADIUS: f32 = 27.0;

#[component]
pub fn ProgressRing(#[prop(into)] percent: Signal<f32>) -> impl IntoView {
    let circumference = 2.0 * std::f32::consts::PI * RING_RADIUS;
    let offset = move || {
        let p = percent.get().clamp(0.0, 100.0) / 100.0;
        circumference * (1.0 - p)
    };
    view! {
        <div class="progress-ring" role="img" aria-label=move || format!("{:.0}%", percent.get())>
            <svg viewBox="0 0 64 64" aria-hidden="true">
                <circle class="track" cx="32" cy="32" r=RING_RADIUS/>
                <circle
                    class="value"
                    cx="32"
                    cy="32"
                    r=RING_RADIUS
                    stroke-dasharray=format!("{circumference:.1}")
                    stroke-dashoffset=move || format!("{:.1}", offset())
                />
            </svg>
            <div class="label">{move || format!("{:.0}%", percent.get())}</div>
        </div>
    }
}

#[component]
pub fn ProgressBar(
    #[prop(into)] percent: Signal<f32>,
    #[prop(into)] label: Signal<String>,
    #[prop(optional)] thin: bool,
) -> impl IntoView {
    view! {
        <div
            class="progress-bar"
            class:thin=thin
            role="progressbar"
            aria-label=move || label.get()
            aria-valuemin="0"
            aria-valuemax="100"
            aria-valuenow=move || format!("{:.0}", percent.get())
        >
            <span style=move || format!("width: {:.1}%", percent.get().clamp(0.0, 100.0))></span>
        </div>
    }
}
