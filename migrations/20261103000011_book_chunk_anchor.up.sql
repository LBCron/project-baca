-- The reader anchor (`p-N`, the same form as `reading_progress.last_anchor_cfi`)
-- of the paragraph each chunk starts in, so a quote search hit opens there.
-- Nullable: chunks stored before this column carry none until backfilled.
ALTER TABLE book_chunks ADD COLUMN IF NOT EXISTS cfi_range VARCHAR(32);
