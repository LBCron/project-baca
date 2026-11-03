-- Saved quotes are listed newest first, one keyset page at a time, and the
-- per-account quota counts them; both read this index instead of sorting
-- every quote the account keeps.
CREATE INDEX IF NOT EXISTS idx_saved_quotes_user_recent ON saved_quotes
USING btree (user_id, created_at DESC, id DESC);
