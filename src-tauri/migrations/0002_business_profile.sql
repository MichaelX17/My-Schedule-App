-- Single-business profile for My Schedule App.
-- Enforces the one-business-per-installation invariant via `singleton_key = 1`.
-- Excludes payment, tax, medical, and regulated fields by product design.
CREATE TABLE IF NOT EXISTS business_profile (
    id TEXT PRIMARY KEY NOT NULL,
    singleton_key INTEGER NOT NULL UNIQUE DEFAULT 1 CHECK (singleton_key = 1),
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    logo_filename TEXT NULL,
    address TEXT NULL,
    city TEXT NULL,
    region TEXT NULL,
    postal_code TEXT NULL,
    country TEXT NULL,
    phone TEXT NULL,
    email TEXT NULL,
    website TEXT NULL,
    timezone TEXT NOT NULL CHECK (length(trim(timezone)) > 0),
    onboarding_completed INTEGER NOT NULL DEFAULT 1 CHECK (onboarding_completed IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
