-- Plain text of every paragraph, numbered as the reader numbers its <p>
-- elements (`p-N`), so in-book search reads an index instead of splitting
-- and stripping whole chapters of HTML on every request. `folded` is the
-- text case-folded one character at a time, as `shared::fold_for_search`
-- folds the query: a plain LIKE on it costs a sixth of ILIKE's per-row
-- lowercasing. btree_gin lets one GIN index serve the per-book filter and
-- the trigram match together.
CREATE EXTENSION IF NOT EXISTS btree_gin;

CREATE TABLE IF NOT EXISTS chapter_paragraphs (
    book_id UUID NOT NULL,
    chapter_number INTEGER NOT NULL,
    n INTEGER NOT NULL,
    text TEXT NOT NULL,
    folded TEXT NOT NULL,
    PRIMARY KEY (book_id, chapter_number, n),
    FOREIGN KEY (book_id, chapter_number)
        REFERENCES chapters (book_id, chapter_number) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_chapter_paragraphs_folded
    ON chapter_paragraphs USING gin (book_id, folded gin_trgm_ops);
