-- Add migration script here
CREATE TABLE IF NOT EXISTS url (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    original_url TEXT NOT NULL,
    short_url TEXT NOT NULL UNIQUE,
    click_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_url_short_url ON url(short_url);
CREATE INDEX IF NOT EXISTS idx_url_created_at ON url(created_at);
-- function to update the updated_at field
CREATE TRIGGER IF NOT EXISTS update_url_updated_at
AFTER UPDATE ON url
BEGIN
    UPDATE url SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;