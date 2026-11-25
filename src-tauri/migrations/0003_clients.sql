-- Clients table for My Schedule App.
-- Stores practical contact/profile information and optional general notes only.
-- Excludes sensitive medical, financial, identity-document, or regulated fields.
-- Uses archive semantics (`is_archived`, `archived_at`) to preserve appointment history.
CREATE TABLE IF NOT EXISTS clients (
    id TEXT PRIMARY KEY NOT NULL,
    full_name TEXT NOT NULL CHECK (length(trim(full_name)) > 0),
    phone TEXT NULL,
    email TEXT NULL,
    address TEXT NULL,
    notes TEXT NULL,
    is_archived INTEGER NOT NULL DEFAULT 0 CHECK (is_archived IN (0, 1)),
    archived_at TEXT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_clients_archived_name
    ON clients (is_archived, full_name COLLATE NOCASE);

CREATE INDEX IF NOT EXISTS idx_clients_updated_at
    ON clients (updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_clients_email
    ON clients (email COLLATE NOCASE);
