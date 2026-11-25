-- Offered services catalog for My Schedule App.
-- Intentionally excludes pricing, currency, tax, invoice, or payment fields.
-- Supports active/inactive toggle and archive semantics so historical appointments
-- retain snapshots and references without losing past records.
CREATE TABLE IF NOT EXISTS services (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    description TEXT NULL,
    default_duration_minutes INTEGER NOT NULL CHECK (default_duration_minutes >= 5 AND default_duration_minutes <= 1440),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    is_archived INTEGER NOT NULL DEFAULT 0 CHECK (is_archived IN (0, 1)),
    archived_at TEXT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_services_state_name
    ON services (is_archived, is_active, name COLLATE NOCASE);

CREATE INDEX IF NOT EXISTS idx_services_updated_at
    ON services (updated_at DESC);
