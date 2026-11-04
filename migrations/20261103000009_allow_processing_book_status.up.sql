-- The worker marks a book as processing while its ingestion job runs, so the
-- curator desk can filter on it; the status existed in the domain but the
-- column refused it.
ALTER TABLE books DROP CONSTRAINT IF EXISTS books_status_check;
ALTER TABLE books ADD CONSTRAINT books_status_check
    CHECK (status IN ('draft', 'processing', 'published', 'archived'));
