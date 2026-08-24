CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY NOT NULL,
    data TEXT NOT NULL,
    expiry_date INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS sessions_expiry_date ON sessions (expiry_date);
