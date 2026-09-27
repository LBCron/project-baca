//! Project Baca — Leptos 0.7 WASM Single Page Application
//! Theme: Vintage Literary (1900–1950)

pub mod api;
pub mod storage;

use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let (lang, set_lang) = signal("EN".to_string());

    let toggle_lang = move |_| {
        set_lang.update(|l| {
            if l == "EN" {
                *l = "ID".to_string();
            } else {
                *l = "EN".to_string();
            }
        });
    };

    view! {
        <div class="app-container">
            <header class="header-vintage">
                <a href="/" class="brand-title">
                    <span class="brand-ornament">"❖"</span>
                    <span>"Project Baca"</span>
                </a>
                <nav class="nav-links">
                    <a href="#catalog" class="nav-link">"Catalog"</a>
                    <a href="#quotes" class="nav-link">"Quote Search"</a>
                    <a href="#about" class="nav-link">"About"</a>
                    <button class="lang-switch" on:click=toggle_lang>
                        {move || format!("[ {} ]", lang.get())}
                    </button>
                </nav>
            </header>

            <section class="hero-vintage">
                <div class="hero-subtitle">"Public Domain Classical Literature"</div>
                <h1 class="hero-heading">"The Timelessness of Words in Classic Print"</h1>
                <p class="hero-desc">
                    "Read world literary masterpieces with elegant typography, a reflowable reader layout, and semantic quote discovery."
                </p>
            </section>

            <main>
                <div class="catalog-section-title">
                    <span>"Featured Classics"</span>
                </div>

                <div class="book-grid">
                    <div class="book-card">
                        <div>
                            <div class="book-tag">"Malay Classical"</div>
                            <h2 class="book-title">"Siti Nurbaya"</h2>
                            <div class="book-author">"Marah Rusli (1922)"</div>
                            <p class="book-synopsis">
                                "The tragic romance of Samsulbahri and Siti Nurbaya, thwarted by rigid customs and the schemes of Datuk Maringgih."
                            </p>
                        </div>
                        <div class="book-card-footer">
                            <span>"~78,000 words"</span>
                            <a href="#read/siti-nurbaya" class="btn-read">
                                <span>"Read"</span>
                                <span>"→"</span>
                            </a>
                        </div>
                    </div>

                    <div class="book-card">
                        <div>
                            <div class="book-tag">"World Classic"</div>
                            <h2 class="book-title">"Pride and Prejudice"</h2>
                            <div class="book-author">"Jane Austen (1813)"</div>
                            <p class="book-synopsis">
                                "Elizabeth Bennet navigates social status, family expectations, and misunderstood affections alongside Mr. Darcy."
                            </p>
                        </div>
                        <div class="book-card-footer">
                            <span>"~120,000 words"</span>
                            <a href="#read/pride-and-prejudice" class="btn-read">
                                <span>"Read"</span>
                                <span>"→"</span>
                            </a>
                        </div>
                    </div>

                    <div class="book-card">
                        <div>
                            <div class="book-tag">"Classic Sci-Fi"</div>
                            <h2 class="book-title">"The Time Machine"</h2>
                            <div class="book-author">"H.G. Wells (1895)"</div>
                            <p class="book-synopsis">
                                "A time traveler journeys to the year 802,701 AD, discovering the fractured human species of Eloi and Morlocks."
                            </p>
                        </div>
                        <div class="book-card-footer">
                            <span>"~32,000 words"</span>
                            <a href="#read/the-time-machine" class="btn-read">
                                <span>"Read"</span>
                                <span>"→"</span>
                            </a>
                        </div>
                    </div>
                </div>
            </main>

            <footer class="footer-vintage">
                <div class="footer-fleuron">"❖"</div>
                <p>"Project Baca — Literary manuscripts are in the public domain. Source code licensed under MIT / Apache-2.0."</p>
            </footer>
        </div>
    }
}
