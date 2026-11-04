UPDATE books SET status = 'draft' WHERE status = 'processing';
ALTER TABLE books DROP CONSTRAINT IF EXISTS books_status_check;
ALTER TABLE books ADD CONSTRAINT books_status_check
    CHECK (status IN ('draft', 'published', 'archived'));
