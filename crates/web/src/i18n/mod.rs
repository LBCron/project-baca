//! ID/EN dictionary. Register: literate and calm, "kamu" never "Anda".
//! Default follows the browser language; the choice persists in localStorage.

use std::collections::HashMap;

const LANG_KEY: &str = "rotaria_lang";

/// (key, Indonesian, English)
const ENTRIES: &[(&str, &str, &str)] = &[
    ("tagline", "Sirkulasi buku-buku hebat.", "The circulation of great books."),
    ("nav_home", "Beranda", "Home"),
    ("nav_search", "Cari", "Search"),
    ("nav_shelf", "Rak", "Shelf"),
    ("nav_profile", "Profil", "Profile"),
    ("nav_shelf_long", "Rak saya", "My shelf"),
    ("nav_main", "Navigasi utama", "Main"),
    ("nav_tabs", "Navigasi bawah", "Primary"),
    ("reading_theme", "Tema baca", "Reading theme"),
    ("streak_label", "Runtunan membaca", "Reading streak"),
    (
        "upload_rejected",
        "Berkas itu bukan EPUB yang bisa kami baca. Periksa berkasnya lalu unggah lagi.",
        "That file is not an EPUB we can read. Check it and upload again.",
    ),
    ("retry", "Coba lagi", "Try again"),
    ("load_failed", "Katalog tidak dapat dimuat.", "The catalog could not be loaded."),
    ("book_load_failed", "Buku tidak dapat dimuat.", "The book could not be loaded."),
    ("chapter_load_failed", "Bab ini belum bisa dimuat.", "This chapter could not be loaded yet."),
    ("page_range", "Halaman {a}\u{2013}{b} dari {n}", "Pages {a}\u{2013}{b} of {n}"),
    ("book_finished_title", "Kamu menamatkan buku ini.", "You finished this book."),
    (
        "book_finished_body",
        "{title} sudah kamu baca sampai halaman terakhir. Simpan kutipan yang paling membekas, atau pilih bacaan berikutnya.",
        "You have read {title} to its last page. Keep the quotes that stayed with you, or choose what to read next.",
    ),
    ("next_book", "Cari buku berikutnya", "Find your next book"),
    ("see_all_catalog", "Lihat semua katalog", "See the whole catalog"),
    ("shelf_empty_title", "Rakmu masih kosong.", "Your shelf is still empty."),
    (
        "shelf_empty_body",
        "Pilih satu buku; posisi baca, kutipan, dan unduhanmu akan tersusun di sini.",
        "Pick a book; your reading position, quotes, and downloads will gather here.",
    ),
    ("browse_catalog", "Jelajahi katalog", "Browse the catalog"),
    ("page_missing", "Halaman ini belum tertulis.", "This page has not been written yet."),
    (
        "page_missing_hint",
        "Tautannya mungkin usang atau bukunya sudah diarsipkan. Katalog selalu terbuka.",
        "The link may be stale or the book archived. The catalog is always open.",
    ),
    ("read_again", "Baca lagi dari awal", "Read again from the start"),
    ("catalog", "Katalog", "Catalog"),
    ("how_it_works", "Cara kerja", "How it works"),
    ("sign_in", "Masuk", "Sign In"),
    ("sign_out", "Keluar", "Sign out"),
    ("register", "Daftar", "Register"),
    ("theme_to_paper", "Ganti ke tema Kertas", "Switch to the Paper theme"),
    ("theme_to_espresso", "Ganti ke tema Espresso", "Switch to the Espresso theme"),
    ("language", "Bahasa", "Language"),
    (
        "hero_sub",
        "Domain publik · gratis · dapat dibaca luring",
        "Public domain · free · readable offline",
    ),
    ("hero_head_a", "Buku klasik, ", "Classic books, "),
    ("hero_head_b", "dibaca dengan cara baru.", "read anew."),
    (
        "hero_desc",
        "Tanpa iklan, tanpa linimasa. Kamu membuka halaman, membaca, dan melanjutkan kapan saja. Lupa alur setelah lama berhenti? Rotaria merangkum bab-bab sebelumnya tanpa membocorkan yang akan datang.",
        "No ads, no feed. Open a page, read, and pick up whenever you like. Lost the thread after a long pause? Rotaria recaps the chapters behind you without revealing what lies ahead.",
    ),
    (
        "search_ph",
        "Cari judul, penulis, atau suasana…",
        "Search a title, an author, or a mood…",
    ),
    ("search_cta", "Cari", "Search"),
    ("chip_all", "Semua", "All"),
    ("chip_id", "Bahasa Indonesia", "Indonesian"),
    ("chip_en", "English", "English"),
    ("chip_short", "Di bawah 2 jam", "Under 2 hours"),
    ("continue_reading", "Lanjutkan bacaanmu", "Pick up where you left off"),
    ("resume", "Lanjutkan", "Resume"),
    (
        "recap_range",
        "Rekap Bab {range} tanpa bocoran",
        "Recap of Chapter {range}, spoiler-free",
    ),
    ("greeting_morning", "Selamat pagi", "Good morning"),
    ("greeting_afternoon", "Selamat siang", "Good afternoon"),
    ("greeting_evening", "Selamat sore", "Good evening"),
    ("greeting_night", "Selamat malam", "Good evening"),
    ("headline_resume_a", "Mau lanjut ", "Pick up "),
    ("headline_resume_b", "yang kemarin?", "where you left off?"),
    ("headline_fresh_a", "Mau membaca ", "What shall we "),
    ("headline_fresh_b", "apa hari ini?", "read today?"),
    ("plate_caption", "Plat I \u{2014} Perpustakaan", "Plate I \u{2014} The Library"),
    (
        "plate_alt",
        "Pembaca memanjat tangga perpustakaan, ilustrasi etsa pena",
        "A reader climbing a library ladder, pen-and-ink engraving",
    ),
    ("theme_filter", "Tema", "Themes"),
    ("see_shelf", "Lihat rak", "See the shelf"),
    ("percent_done", "{n}% selesai", "{n}% read"),
    ("recap_last", "Rekap bab lalu", "Recap so far"),
    ("share_book", "Bagikan buku", "Share this book"),
    ("share_copied", "Tautan disalin.", "Link copied."),
    ("font_smaller", "Perkecil huruf", "Smaller type"),
    ("font_larger", "Perbesar huruf", "Larger type"),
    ("leading_tight", "Rapat", "Tight"),
    ("leading_normal", "Normal", "Normal"),
    ("leading_loose", "Renggang", "Loose"),
    ("reader_since", "Pembaca sejak {date}", "Reader since {date}"),
    ("xp", "{n} XP", "{n} XP"),
    ("quotes_all", "Semua ({n})", "All ({n})"),
    ("saved_count", "{n} tersimpan luring", "{n} saved offline"),
    (
        "no_offline",
        "Belum ada buku tersimpan. Ketuk ikon unduh di halaman buku untuk menyimpannya.",
        "No saved books yet. Tap the download icon on a book page to keep one here.",
    ),
    ("picks", "Pilihan minggu ini", "This week's picks"),
    ("see_all", "Semua", "All"),
    ("load_more", "Muat lebih banyak", "Load more"),
    ("loading", "Memuat…", "Loading…"),
    (
        "no_results",
        "Tidak ada yang cocok. Coba kata lain atau suasana yang berbeda.",
        "Nothing matched. Try another word or a different mood.",
    ),
    ("read", "Baca", "Read"),
    ("words", "kata", "words"),
    ("chapters", "Daftar bab", "Chapters"),
    ("open_chapters", "Buka daftar bab", "Open the chapter list"),
    ("chapter_here", "sedang kamu baca", "reading now"),
    ("chapter_read", "sudah kamu lewati", "read"),
    ("book_search", "Cari di buku ini", "Search this book"),
    ("book_search_ph", "Kata atau frasa dari halamannya", "A word or phrase from its pages"),
    ("book_search_none", "Belum ada halaman yang memuat kata itu.", "No page holds those words."),
    ("book_search_offline", "Dicari di salinan pada perangkatmu.", "Searched in the copy on your device."),
    ("chapter_end_title", "Bab {n} usai", "Chapter {n} finished"),
    (
        "chapter_end_body",
        "Sebelum melangkah ke bab berikutnya, kamu bisa menengok intisari bab ini.",
        "Before you turn to the next chapter, you can look over what this one held.",
    ),
    ("next_chapter", "Bab berikutnya", "Next chapter"),
    ("save_selection", "Simpan kutipan", "Save quote"),
    ("chapters_count", "{n} bab", "{n} chapters"),
    ("chapters_count_one", "{n} bab", "{n} chapter"),
    ("chapter", "Bab", "Chapter"),
    ("reading_progress", "Kemajuan membaca", "Reading progress"),
    ("last_read", "Terakhir dibaca", "Last read"),
    ("start_reading", "Mulai membaca", "Start reading"),
    ("continue_chapter", "Lanjutkan Bab", "Continue Chapter"),
    ("save_offline", "Simpan untuk dibaca luring", "Save for offline reading"),
    ("saved_offline", "Tersimpan di perangkat", "Saved on this device"),
    ("saving", "Menyimpan…", "Saving…"),
    ("quote_finder", "Cari kutipan", "Find a quote"),
    ("atomic_cards", "Kartu insight", "Insight cards"),
    ("how_head_a", "Apa yang bisa Rotaria ", "What can Rotaria "),
    ("how_head_b", "lakukan?", "do?"),
    ("feature_1_t", "Membaca tanpa gangguan", "Read without distraction"),
    (
        "feature_1_d",
        "Halaman dibalik dengan sentuhan, posisi tersimpan sendiri, dan buku dapat diunduh untuk dibaca di perjalanan.",
        "Pages turn with a touch, your place keeps itself, and books can be downloaded for the road.",
    ),
    ("feature_2_t", "Rekap tanpa bocoran", "Recaps without spoilers"),
    (
        "feature_2_d",
        "Kembali setelah lama berhenti? Ringkasan bab-bab sebelumnya menyusul, tanpa membocorkan bab berikutnya.",
        "Back after a long pause? A summary of the chapters behind you follows, never the ones ahead.",
    ),
    ("feature_3_t", "Mencari kutipan dari ingatan", "Find a quote from memory"),
    (
        "feature_3_d",
        "Tulis suasananya, bukan kata persisnya. Kutipan ditemukan, lalu menjadi kartu yang siap dibagikan.",
        "Describe the mood, not the exact words. The passage is found, then becomes a card ready to share.",
    ),
    (
        "footer",
        "Rotaria — sirkulasi buku domain publik. Sumber: Standard Ebooks, Project Gutenberg, Wikisource.",
        "Rotaria — public domain books in circulation. Sources: Standard Ebooks, Project Gutenberg, Wikisource.",
    ),
    ("auth_title_login", "Masuk ke Rotaria", "Sign in to Rotaria"),
    ("auth_title_register", "Buat akun", "Create an account"),
    ("auth_title_otp", "Periksa kotak masukmu", "Check your inbox"),
    (
        "auth_otp_hint",
        "Kami mengirim kode enam digit ke {email}. Kode berlaku sepuluh menit.",
        "We sent a six-digit code to {email}. It is valid for ten minutes.",
    ),
    ("auth_name", "Nama tampilan", "Display name"),
    ("auth_email", "Surel", "Email"),
    ("auth_password", "Kata sandi", "Password"),
    ("auth_password_hint", "Minimal 8 karakter", "At least 8 characters"),
    ("auth_otp", "Kode verifikasi", "Verification code"),
    ("auth_continue", "Lanjutkan", "Continue"),
    ("auth_working", "Sebentar…", "One moment…"),
    ("cancel", "Batal", "Cancel"),
    ("close", "Tutup", "Close"),
    ("auth_switch_register", "Belum punya akun? Daftar", "No account yet? Register"),
    ("auth_switch_login", "Sudah punya akun? Masuk", "Already have an account? Sign in"),
    (
        "auth_login_failed",
        "Surel atau kata sandi tidak cocok. Belum verifikasi? Daftar ulang untuk menerima kode baru.",
        "Email or password did not match. Not verified yet? Register again to receive a new code.",
    ),
    (
        "auth_guest_merge",
        "Progres bacaan di perangkat ini akan disatukan ke akunmu.",
        "Reading progress on this device will be merged into your account.",
    ),
    ("auth_welcome", "Selamat datang kembali.", "Welcome back."),
    ("reader_back", "Kembali ke halaman buku", "Back to the book"),
    ("reader_settings", "Pengaturan tampilan", "Display settings"),
    ("page_of", "Halaman {a} dari {b}", "Page {a} of {b}"),
    ("minutes_left", "~{n} menit lagi di bab ini", "~{n} minutes left in this chapter"),
    ("minutes_left_one", "~{n} menit lagi di bab ini", "~{n} minute left in this chapter"),
    ("chapter_last_page", "Halaman terakhir bab ini", "Last page of this chapter"),
    ("offline_copy", "salinan luring", "offline copy"),
    ("queued", "{n} menunggu sinkron", "{n} awaiting sync"),
    ("theme_paper", "Kertas", "Paper"),
    ("theme_sepia", "Sepia", "Sepia"),
    ("theme_espresso", "Espresso", "Espresso"),
    ("font_size", "Ukuran huruf", "Font size"),
    ("line_height", "Jarak baris", "Line spacing"),
    ("prev_chapter", "Bab sebelumnya", "Previous chapter"),
    ("next_page", "Halaman berikutnya", "Next page"),
    ("prev_page", "Halaman sebelumnya", "Previous page"),
    ("toggle_menu", "Tampilkan atau sembunyikan menu", "Show or hide the menu"),
    ("chapter_end", "Akhir bab", "End of chapter"),
    (
        "book_end",
        "Tamat. Terima kasih sudah membaca sampai halaman terakhir.",
        "The end. Thank you for reading to the last page.",
    ),
    ("recap_title", "Cerita sejauh ini", "The story so far"),
    ("recap_note", "Tanpa bocoran bab berikutnya.", "No spoilers for what lies ahead."),
    ("key_concepts", "Gagasan utama", "Key ideas"),
    ("notable_quotes", "Kutipan penting", "Notable quotes"),
    ("historical_context", "Konteks sejarah", "Historical context"),
    ("key_characters", "Tokoh", "Characters"),
    (
        "not_generated",
        "Belum tersedia untuk bab ini.",
        "Not available for this chapter yet.",
    ),
    (
        "quote_ph",
        "Tulis gagasan atau suasana, bukan kata persisnya…",
        "Describe an idea or a mood, not the exact words…",
    ),
    (
        "quote_empty",
        "Belum ada kutipan yang cocok. Coba suasana lain.",
        "No matching passage yet. Try another mood.",
    ),
    (
        "quote_error",
        "Pencarian kutipan sedang tidak tersedia. Coba lagi sebentar lagi.",
        "Quote search is unavailable right now. Try again shortly.",
    ),
    ("quote_jump", "Buka halaman", "Open the page"),
    ("quote_save", "Simpan", "Save"),
    ("quote_saved", "Tersimpan", "Saved"),
    ("quote_share", "Bagikan kartu", "Share card"),
    ("quote_saved_device", "Tersimpan di perangkat ini", "Saved on this device"),
    (
        "quote_guest_hint",
        "Kutipan tersimpan di perangkat ini. Masuk untuk menyatukannya ke akunmu.",
        "Quotes stay on this device. Sign in to merge them into your account.",
    ),
    ("similarity", "kemiripan", "match"),
    ("shelf_title", "Rak saya", "My shelf"),
    (
        "guest_shelf",
        "Masuk untuk melihat rak, streak, dan kutipan tersimpanmu.",
        "Sign in to see your shelf, streaks, and saved quotes.",
    ),
    ("back_to_catalog", "Kembali ke katalog", "Back to catalog"),
    ("streak_days", "{n} hari berturut-turut", "{n} days in a row"),
    ("streak_days_one", "{n} hari berturut-turut", "{n} day in a row"),
    (
        "streak_hint",
        "Baca lima menit hari ini untuk menjaga streak.",
        "Read five minutes today to keep the streak.",
    ),
    ("badges", "Lencana", "Badges"),
    ("saved_quotes", "Kutipan tersimpan", "Saved quotes"),
    ("offline_books", "Tersimpan di perangkat", "Saved on this device"),
    ("sessions", "Sesi dan keamanan", "Sessions and security"),
    ("current_password", "Kata sandi saat ini", "Current password"),
    ("current_password_wrong", "Kata sandi lamamu tidak cocok.", "Your current password does not match."),
    ("new_password", "Kata sandi baru", "New password"),
    ("revoke_others", "Keluarkan sesi di perangkat lain", "Sign out other devices"),
    ("change_password", "Ganti kata sandi", "Change password"),
    ("password_changed", "Kata sandi diperbarui.", "Password updated."),
    ("revoke_all", "Keluar dari semua perangkat", "Sign out everywhere"),
    ("revoke_others_now", "Keluarkan perangkat lain", "Sign out other devices"),
    ("others_revoked", "Perangkat lain sudah dikeluarkan.", "Other devices signed out."),
    ("devices", "Perangkat yang masuk", "Signed-in devices"),
    ("this_device", "Perangkat ini", "This device"),
    ("unknown_device", "Perangkat tak dikenal", "Unknown device"),
    ("last_seen", "Terakhir aktif {t}", "Last active {t}"),
    ("sign_out_device", "Keluarkan", "Sign out"),
    ("device_revoked", "Perangkat dikeluarkan.", "Device signed out."),
    ("confirm_again", "Ketuk lagi untuk konfirmasi", "Tap again to confirm"),
    ("member_since", "Pembaca sejak", "Reader since"),
    (
        "no_badges",
        "Lencana pertamamu menunggu di menit pertama membaca.",
        "Your first badge awaits after your first minute of reading.",
    ),
    (
        "no_quotes",
        "Belum ada kutipan. Temukan lewat pencarian kutipan di halaman buku.",
        "No quotes yet. Find one with quote search on a book page.",
    ),
    ("admin_title", "Kurasi naskah", "Curate manuscripts"),
    ("admin_upload", "Unggah EPUB domain publik", "Upload a public domain EPUB"),
    ("admin_drop", "Pilih berkas .epub (maks 50 MB)", "Choose an .epub file (max 50 MB)"),
    (
        "admin_too_large",
        "Berkas ini lebih dari 50 MB, batas satu unggahan.",
        "This file is over 50 MB, the limit for one upload.",
    ),
    ("untitled_pending", "Judul menyusul dari EPUB", "Title to come from the EPUB"),
    (
        "admin_title_field",
        "Judul (kosongkan untuk memakai judul EPUB)",
        "Title (leave blank to use the EPUB's)",
    ),
    ("admin_author_field", "Penulis (opsional)", "Author (optional)"),
    ("admin_queue", "Antrekan pemrosesan", "Queue for processing"),
    ("admin_uploading", "Mengunggah…", "Uploading…"),
    ("admin_job", "Pekerjaan", "Job"),
    ("admin_choose_first", "Pilih berkas .epub dahulu.", "Choose an .epub file first."),
    ("job_queued", "Antre", "Queued"),
    ("job_parsing", "Membaca EPUB", "Parsing"),
    ("job_chunking", "Memotong bab", "Chunking"),
    ("job_embedding", "Menyusun indeks", "Embedding"),
    ("job_summarizing", "Merangkum", "Summarizing"),
    ("job_published", "Terbit", "Published"),
    ("job_failed", "Gagal", "Failed"),
    ("job_retrying", "Percobaan {n} gagal, akan dicoba lagi otomatis.", "Attempt {n} failed; it will be retried."),
    ("admin_forbidden", "Halaman ini untuk kurator.", "This page is for curators."),
    ("admin_desk", "Meja kurator", "Curator desk"),
    ("tab_manuscripts", "Naskah", "Manuscripts"),
    ("tab_queue", "Antrean", "Queue"),
    ("tab_retention", "Retensi", "Retention"),
    ("status_draft", "Draf", "Draft"),
    ("status_processing", "Diproses", "Processing"),
    ("status_published", "Terbit", "Published"),
    ("status_archived", "Arsip", "Archived"),
    ("archive", "Arsipkan", "Archive"),
    ("archived_done", "Naskah diarsipkan.", "Manuscript archived."),
    ("chunks_count", "{n} potongan", "{n} chunks"),
    ("chunks_count_one", "{n} potongan", "{n} chunk"),
    (
        "no_manuscripts",
        "Belum ada naskah dengan status ini.",
        "No manuscripts in this status yet.",
    ),
    ("dlq_title", "Pekerjaan gagal", "Failed jobs"),
    (
        "dlq_empty",
        "Tidak ada pekerjaan yang gagal. Antrean bersih.",
        "No failed jobs. The queue is clear.",
    ),
    ("replay", "Ulangi", "Replay"),
    ("dismiss", "Tutup", "Dismiss"),
    ("dismissed", "Entri ditutup dan tidak akan diulang.", "Entry dismissed; it will not run again."),
    ("dlq_book_gone", "Bukunya sudah tidak ada", "The book is gone"),
    ("restore", "Terbitkan lagi", "Publish again"),
    ("restored", "Naskah terbit lagi.", "Manuscript published again."),
    (
        "restore_unfinished",
        "Naskah ini diarsipkan sebelum selesai diproses; unggah ulang EPUB-nya.",
        "This manuscript was archived before ingestion finished; upload the EPUB again.",
    ),
    ("replayed", "Pekerjaan diantrekan ulang.", "Job queued again."),
    (
        "replay_expired",
        "Catatan pekerjaan sudah kedaluwarsa; unggah ulang EPUB-nya.",
        "The job record expired; upload the EPUB again.",
    ),
    ("funnel_pick", "Pilih naskah", "Choose a manuscript"),
    (
        "funnel_hint",
        "Berapa pembaca yang mencapai tiap bab.",
        "How many readers reach each chapter.",
    ),
    ("readers_reached", "{n} pembaca", "{n} readers"),
    ("readers_reached_one", "{n} pembaca", "{n} reader"),
    ("drop_off", "turun {n}%", "{n}% drop"),
    (
        "no_funnel",
        "Belum ada data pembaca untuk naskah ini.",
        "No reader data for this manuscript yet.",
    ),
    (
        "error_generic",
        "Ada yang tidak beres. Coba lagi.",
        "Something went wrong. Try again.",
    ),
    (
        "offline_notice",
        "Kamu sedang luring. Buku yang tersimpan tetap bisa dibaca.",
        "You are offline. Saved books remain readable.",
    ),
    ("not_found", "Naskah tidak ditemukan.", "Manuscript not found."),
    (
        "rate_limited",
        "Terlalu banyak percobaan. Coba lagi dalam {t}.",
        "Too many attempts. Try again in {t}.",
    ),
    (
        "quotes_full",
        "Rak kutipanmu sudah penuh. Kutipan ini belum bisa disimpan.",
        "Your quote shelf is full. This one could not be kept.",
    ),
    (
        "err_bad_request",
        "Permintaan itu tidak bisa diproses. Periksa isianmu lalu coba lagi.",
        "That request could not be processed. Check what you entered and try again.",
    ),
    (
        "err_validation_failed",
        "Ada isian yang belum sesuai. Periksa lagi, ya.",
        "Something you entered does not fit. Check it again.",
    ),
    (
        "err_unauthorized",
        "Sesimu sudah berakhir. Masuk lagi untuk melanjutkan.",
        "Your session has ended. Sign in again to continue.",
    ),
    (
        "err_forbidden",
        "Kamu tidak punya akses ke bagian ini.",
        "You do not have access to this.",
    ),
    (
        "err_not_found",
        "Yang kamu cari tidak ditemukan.",
        "What you are looking for was not found.",
    ),
    (
        "err_method_not_allowed",
        "Permintaan itu tidak didukung.",
        "That request is not supported.",
    ),
    (
        "err_conflict",
        "Ada perubahan yang mendahuluimu. Muat ulang lalu coba lagi.",
        "Something changed before you did. Reload and try again.",
    ),
    (
        "err_quota_exceeded",
        "Batas simpananmu sudah penuh.",
        "You have reached your storage limit.",
    ),
    (
        "err_payload_too_large",
        "Berkas itu terlalu besar.",
        "That file is too large.",
    ),
    (
        "err_rate_limited",
        "Terlalu banyak percobaan. Coba lagi sebentar lagi.",
        "Too many attempts. Try again shortly.",
    ),
    (
        "err_ai_rate_limited",
        "Pencarian kutipan sedang ramai. Coba lagi dalam {t}.",
        "Quote search is busy. Try again in {t}.",
    ),
    (
        "err_service_unavailable",
        "Layanan sedang sibuk. Coba lagi dalam {t}.",
        "The service is busy. Try again in {t}.",
    ),
    (
        "err_external_service_error",
        "Layanan pendukung sedang tidak terjangkau. Coba lagi sebentar lagi.",
        "A service we rely on is unreachable. Try again shortly.",
    ),
    (
        "err_database_error",
        "Ada yang tidak beres di pihak kami. Coba lagi sebentar lagi.",
        "Something went wrong on our side. Try again shortly.",
    ),
    (
        "err_internal_error",
        "Ada yang tidak beres di pihak kami. Coba lagi sebentar lagi.",
        "Something went wrong on our side. Try again shortly.",
    ),
    (
        "err_field_display_name",
        "Nama tampilanmu perlu 2 sampai 100 karakter.",
        "Your display name needs 2 to 100 characters.",
    ),
    (
        "err_field_email",
        "Alamat email itu belum benar. Periksa lagi, ya.",
        "That email address does not look right. Check it again.",
    ),
    (
        "err_field_password",
        "Kata sandimu perlu 8 sampai 128 karakter.",
        "Your password needs 8 to 128 characters.",
    ),
    (
        "err_field_new_password",
        "Kata sandi barumu perlu 8 sampai 128 karakter.",
        "Your new password needs 8 to 128 characters.",
    ),
    (
        "err_field_current_password",
        "Isi kata sandimu yang sekarang.",
        "Enter your current password.",
    ),
    (
        "err_field_otp",
        "Kode verifikasi terdiri dari 6 angka.",
        "The verification code has 6 digits.",
    ),
    (
        "err_field_quote_text",
        "Kutipan perlu 1 sampai 2.000 karakter.",
        "A quote needs 1 to 2,000 characters.",
    ),
    (
        "err_field_query",
        "Tulis 3 sampai 500 karakter untuk mencari kutipan.",
        "Write 3 to 500 characters to search for a quote.",
    ),
    (
        "err_field_q",
        "Tulis 2 sampai 200 karakter untuk mencari.",
        "Write 2 to 200 characters to search.",
    ),
    (
        "err_field_avatar_url",
        "Tautan foto profil itu tidak bisa dipakai.",
        "That profile picture link cannot be used.",
    ),
    (
        "auth_account_exists",
        "Alamat ini sudah punya akun. Masuk saja dengan kata sandimu.",
        "This address already has an account. Sign in with your password.",
    ),
    ("unit_minutes", "menit", "minutes"),
    ("unit_seconds", "detik", "seconds"),
    ("hours", "jam", "h"),
    ("minutes", "mnt", "min"),
];

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

    /// Stored choice first, then the browser language, then English.
    pub fn load() -> Lang {
        let stored = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item(LANG_KEY).ok().flatten());
        match stored.as_deref() {
            Some("ID") => Lang::Id,
            Some("EN") => Lang::En,
            _ => Lang::from_browser(
                web_sys::window()
                    .and_then(|w| w.navigator().language())
                    .as_deref(),
            ),
        }
    }

    pub fn from_browser(language: Option<&str>) -> Lang {
        match language {
            Some(l) if l.to_ascii_lowercase().starts_with("id") => Lang::Id,
            _ => Lang::En,
        }
    }

    pub fn save(self) {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item(LANG_KEY, self.code());
        }
    }

    /// Looks a key up; unknown keys echo themselves so a typo is visible.
    /// True when the dictionary holds `key` (a miss would show the key).
    pub fn has(self, key: &str) -> bool {
        ENTRIES.iter().any(|(k, _, _)| *k == key)
    }

    pub fn text(self, key: &str) -> String {
        ENTRIES
            .iter()
            .find(|(k, _, _)| *k == key)
            .map(|(_, id, en)| match self {
                Lang::Id => (*id).to_string(),
                Lang::En => (*en).to_string(),
            })
            .unwrap_or_else(|| key.to_string())
    }

    /// `{n}` texts for a count. English takes the `_one` form for exactly
    /// one; Indonesian nouns do not inflect, so its `_one` rows repeat the
    /// base text.
    pub fn text_n(self, key: &str, n: i64) -> String {
        let singular = format!("{key}_one");
        let key = if n == 1 && ENTRIES.iter().any(|(k, _, _)| *k == singular) {
            singular.as_str()
        } else {
            key
        };
        self.text_with(key, "n", &n.to_string())
    }

    /// `text` with one `{name}` placeholder substituted.
    pub fn text_with(self, key: &str, name: &str, value: &str) -> String {
        self.text(key).replace(&format!("{{{name}}}"), value)
    }

    pub fn dict(self) -> HashMap<&'static str, String> {
        ENTRIES.iter().map(|(k, _, _)| (*k, self.text(k))).collect()
    }

    /// Reading time in the reader's own words: "2 jam 10 mnt" / "2 h 10 min".
    pub fn duration(self, minutes: i32) -> String {
        let minutes = minutes.max(0);
        let (h, m) = (minutes / 60, minutes % 60);
        match (h, m) {
            (0, m) => format!("~{m} {}", self.text("minutes")),
            (h, 0) => format!("~{h} {}", self.text("hours")),
            (h, m) => format!("~{h} {} {m} {}", self.text("hours"), self.text("minutes")),
        }
    }

    /// A wait in whole minutes when it is a minute or more, else seconds.
    pub fn wait_text(self, seconds: u64) -> String {
        if seconds >= 60 {
            format!("{} {}", seconds.div_ceil(60), self.text("unit_minutes"))
        } else {
            format!("{} {}", seconds.max(1), self.text("unit_seconds"))
        }
    }

    /// Three-letter month name in the interface language (1 = January).
    pub fn month_short(self, month: u32) -> &'static str {
        const ID: [&str; 12] = [
            "Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des",
        ];
        const EN: [&str; 12] = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let index = month.clamp(1, 12) as usize - 1;
        match self {
            Lang::Id => ID[index],
            Lang::En => EN[index],
        }
    }

    /// "Okt 2026" or "Oct 2026".
    pub fn month_year(self, date: chrono::DateTime<chrono::Utc>) -> String {
        use chrono::Datelike;
        format!("{} {}", self.month_short(date.month()), date.year())
    }

    /// "8 Okt 2026" or "8 Oct 2026".
    pub fn date_short(self, date: chrono::DateTime<chrono::Utc>) -> String {
        use chrono::Datelike;
        format!(
            "{} {} {}",
            date.day(),
            self.month_short(date.month()),
            date.year()
        )
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

pub fn entry_count() -> usize {
    ENTRIES.len()
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

#[derive(Clone, Copy)]
struct LangContext(
    leptos::prelude::ReadSignal<Lang>,
    leptos::prelude::WriteSignal<Lang>,
);

/// One language signal for the whole app, provided at the root.
pub fn provide_lang() -> (
    leptos::prelude::ReadSignal<Lang>,
    leptos::prelude::WriteSignal<Lang>,
) {
    use leptos::prelude::*;
    let (l, s) = lang_signal();
    provide_context(LangContext(l, s));
    (l, s)
}

/// The shared language signal, or a detached one outside the app root.
pub fn use_lang() -> (
    leptos::prelude::ReadSignal<Lang>,
    leptos::prelude::WriteSignal<Lang>,
) {
    use leptos::prelude::*;
    match use_context::<LangContext>() {
        Some(ctx) => (ctx.0, ctx.1),
        None => lang_signal(),
    }
}

pub fn apply_lang(set: leptos::prelude::WriteSignal<Lang>, lang: Lang) {
    use leptos::prelude::*;
    set.set(lang);
    lang.save();
    lang.document_lang();
}
