# Project Baca

A timeless digital reading sanctuary for classic literature. Built for readers who cherish typography, deep focus, and open culture.

---

## 1. What is Project Baca?

Project Baca is a modern, open-access e-reader designed to bring the elegance and tactile beauty of 20th-century classic book print into modern web browsers and mobile devices. 

In a world filled with cluttered commercial reading apps, algorithmic distractions, and proprietary walled gardens, Project Baca offers a quiet, dignified space for deep reading. It pairs carefully typeset public domain masterpieces with thoughtful, non-intrusive comprehension tools—such as spoiler-free chapter recaps, atomic insight cards, and natural language quote discovery.

Whether you are revisiting timeless world literature or exploring classic Indonesian heritage, Project Baca is built to make long-form reading effortless, joyful, and completely free from digital noise.

---

## 2. Why Project Baca? (The Problem We Solve)

* **The Problem of Digital Reading Fatigue:** Most modern reading apps are bloated with social feeds, invasive tracking, and popups that destroy the concentration needed for deep literary engagement.
* **Reviving Public Domain Treasures:** Hundreds of thousands of world-changing novels, philosophical texts, and historical essays exist in the public domain (via Project Gutenberg, Standard Ebooks, and Wikisource). However, they are frequently trapped in raw, unformatted files or clunky legacy readers. Project Baca restores these texts with editorial craftsmanship.
* **Overcoming Long-Form Drop-off:** Many readers abandon dense classics because pausing for several weeks makes them lose track of intricate plots and character dynamics. Project Baca introduces gentle, spoiler-free catch-up recaps and chapter summaries to help readers effortlessly resume where they left off.
* **True Privacy & Offline Ownership:** You should never need a constant internet connection to read a book you love. Project Baca lets you download entire books directly into your browser storage with a single tap.

---

## 3. Reader Experience & Key Features

* **Vintage Literary Aesthetic (1900–1950):**
  Designed with the warmth of aged archival paper (`#F9F6F0`), dense ink (`#2B2625`), and terracotta accents (`#9D5A3C`). Uses genuine literary serif typography (*EB Garamond*, *Newsreader*) paired with Victorian fleuron chapter ornaments `❖` to provide a reading experience that treats your eyes with respect.
* **Reflowable Tap-to-Turn E-Reader:**
  Horizontal page pagination engineered to feel like turning the pages of a real physical book. Built-in DOM CFI anchor tracking guarantees you will never lose your reading position when rotating your phone or adjusting font sizes.
* **100% Offline-First Freedom:**
  Download any book in the catalog to your device with one click. Powered by browser IndexedDB, your library, reading progress, and chapter contents remain fully functional on trains, flights, or off-grid retreats.
* **Natural Language Quote Discovery:**
  Search books by thought, emotion, or scene context rather than exact words. If you remember "a rainy night in London" or "hesitation about love and duty", our semantic search identifies the exact chapter, paragraph, and context in milliseconds.
* **Chapter Atomic Insight Cards:**
  Quickly review chapter takeaways through vintage library-style index cards covering core themes, memorable quotes, and historical context—ideal for students and literature enthusiasts.
* **Spoiler-Free Catch-Up Recaps:**
  Returning to a novel after weeks away? Generate an instant summary of prior events that brings you up to speed without ever revealing what happens in future chapters.
* **Mindful Reading Streaks & Gamification:**
  Cultivate a healthy daily reading habit with gentle reading timers, daily streak logs, and milestone badges celebrating reading milestones—without predatory notifications or manipulative algorithms.
* **Bilingual Heritage (Indonesian & English):**
  Instant language toggling `[ ID | EN ]` allowing seamless reading across Indonesian literary history and international classics.

---

## 4. Architectural Highlights & Technology

Project Baca is engineered from the ground up for speed, reliability, and security using modern systems programming:

* **Stateless API Gateway:** Built with **Rust (Axum)** for predictable low-latency performance and memory safety.
* **Client-Side WebAssembly (WASM):** Built with **Leptos 0.7**, delivering native-speed page transitions and smooth 60 FPS pagination inside any modern browser.
* **Postgres for Everything:** Relational tables, sub-5ms typo-tolerant catalog search (`pg_trgm`), and sub-10ms semantic vector traversal (`pgvector` HNSW) all unified within **PostgreSQL 17**.
* **Modern Edge Transport:** Terminated via **Caddy Reverse Proxy** supporting **HTTP/3 over QUIC** (UDP 443) with seamless fallback to HTTP/2 (TCP 443). Features 0-RTT/1-RTT handshakes and connection migration for uninterrupted reading while moving between Wi-Fi and mobile networks.
* **Pragmatic Background Queues:** Asynchronous EPUB processing, HTML cleaning, and transactional notifications managed via **Redis 7 Streams**.

---

## 5. Quickstart for Developers & Self-Hosters

Complete installation steps, environment configurations, and development workflows are documented in [GUIDE.md](GUIDE.md).

```bash
# 1. Start core database and cache services (PostgreSQL 17, Redis 7, MinIO, Mailpit)
make db-up

# 2. Run backend API server with hot reload
make dev-server

# 3. Run frontend Web Reader with hot reload
make dev-web
```

Once running:
* **Web Reader PWA:** `http://localhost:3000`
* **REST API & Documentation:** `http://localhost:8080/swagger-ui`

---

## 6. Project Documentation & Specifications

* **Detailed Setup & Developer Guide:** [GUIDE.md](GUIDE.md)
* **Monorepo Architecture Blueprint:** [ARCHITECTURE.md](ARCHITECTURE.md)
* **Distributed Worker & Ingestion Pipeline:** [DISTRIBUTED.md](DISTRIBUTED.md)
* **Release History & Changelog:** [CHANGELOG.md](CHANGELOG.md)
* **Public Domain Legal Notice:** [NOTICE.md](NOTICE.md)


---

## 7. Open Access & Heritage Notice

All literary texts distributed through Project Baca belong to the public domain and are curated from open archives including Standard Ebooks, Project Gutenberg, and Wikisource. Please consult [NOTICE.md](NOTICE.md) for licensing, attribution, and public domain legal notices.
