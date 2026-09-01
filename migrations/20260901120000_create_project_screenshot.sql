CREATE TABLE IF NOT EXISTS project_screenshot (
    screenshot_id TEXT NOT NULL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project (id) ON DELETE CASCADE,
    media_type TEXT NOT NULL,
    image BLOB NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    alt_text TEXT NOT NULL,
    caption TEXT,
    sort_order INTEGER NOT NULL,
    UNIQUE (project_id, sort_order)
);
