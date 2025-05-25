-- Add migration script here
-- sqlite3 migration script
CREATE TABLE IF NOT EXISTS todo (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    completed BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_todo_completed ON todo(completed);
CREATE INDEX IF NOT EXISTS idx_todo_created_at ON todo(created_at);
-- function to update the updated_at field
CREATE TRIGGER IF NOT EXISTS update_todo_updated_at
AFTER UPDATE ON todo
BEGIN
    UPDATE todo SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;
-- function to delete completed todos
CREATE TRIGGER IF NOT EXISTS delete_completed_todos
AFTER DELETE ON todo
WHEN OLD.completed = 1
BEGIN
    DELETE FROM todo WHERE id = OLD.id;
END;