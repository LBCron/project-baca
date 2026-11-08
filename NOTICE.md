# NOTICE — Copyright & Legal Licenses

Public domain status of the texts, attribution of sources and fonts, third-party licenses.

## 1. Public Domain Copyright Status

All manuscripts curated, ingested, and presented in Project Baca are legally in the **Public Domain**:

1. **Indonesian Law (Law No. 28/2014 on Copyright):**
   - Under Article 58(1), copyright on written works lasts for the life of the author plus 70 years after their death, calculated from January 1 of the following year.
   - After this period, works enter the public domain and may be read, reproduced, and adapted freely without royalty obligations.
2. **Berne Convention & International Terms:**
   - All classic literature included has passed the term of copyright protection under international rule of the shorter term.

## 2. Manuscript & Typography Attribution

Raw texts and typography are sourced from open literary preservation initiatives:

* **Project Gutenberg:** [www.gutenberg.org](https://www.gutenberg.org) — Public domain text archive.
* **Standard Ebooks:** [standardebooks.org](https://standardebooks.org) — Clean typography and modern EPUB3 editions.
* **Typography:** Fonts (*EB Garamond*, *Newsreader*, *Plus Jakarta Sans*) licensed under [SIL Open Font License 1.1](http://scripts.sil.org/OFL), self-hosted from `crates/web/fonts/` (woff2 subsets from Google Fonts) with each family's `OFL-*.txt` alongside.

## 3. Third-Party Open Source Software

* **Rust Ecosystem:** Axum (MIT), Leptos (MIT/Apache-2.0), SeaORM (MIT/Apache-2.0), Tokio (MIT), lettre (MIT), utoipa (MIT/Apache-2.0).
* **Database & Infrastructure:** PostgreSQL (PostgreSQL License), pgvector (PostgreSQL License), Redis (RSALv2/SSPLv1/BSD), MinIO (AGPLv3), Mailpit (MIT).
* **Common password list:** SecLists `10k-most-common.txt` (MIT, Daniel Miessler), entries of 8 characters or more in `crates/shared/src/common_passwords.txt`.
* **AI & Embeddings:** text-embeddings-inference 1.9 (Apache-2.0, Hugging Face) with ONNX Runtime (MIT), serving the multilingual-e5-base model (MIT, intfloat) at a pinned revision; the `anthropic` Python SDK 2.6.0 (Apache-2.0).
## 4. Project License

The Project Baca source code is proprietary, all rights reserved; see [LICENSE](LICENSE). Versions 0.1.1 and earlier were released under MIT OR Apache-2.0 and stay under those terms for whoever obtained them. Book texts stay in the public domain and fonts under the SIL Open Font License 1.1 regardless of the code license.

## 5. Disclaimer of Warranty

Project Baca software is provided "AS IS", without warranties of any kind, express or implied, including fitness for a particular purpose or merchantability.
