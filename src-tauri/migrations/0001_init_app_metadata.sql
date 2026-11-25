-- Foundational metadata table for My Schedule App.
-- Business domain tables are introduced in subsequent steps.
CREATE TABLE IF NOT EXISTS app_metadata (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO app_metadata (key, value, updated_at)
VALUES (
    'schema_foundation_version',
    '1',
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
);
