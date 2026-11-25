-- Appointments, appointment-service snapshots, and optional session plans.
-- Appointments are individual scheduled events stored with UTC instants and
-- the business IANA timezone context. Session plans optionally group a client's
-- appointments without replacing individual appointment records.

CREATE TABLE IF NOT EXISTS session_plans (
    id TEXT PRIMARY KEY NOT NULL,
    client_id TEXT NOT NULL REFERENCES clients(id) ON DELETE RESTRICT,
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    description TEXT NULL,
    planned_sessions INTEGER NOT NULL CHECK (planned_sessions >= 1 AND planned_sessions <= 500),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed', 'archived')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS appointments (
    id TEXT PRIMARY KEY NOT NULL,
    client_id TEXT NOT NULL REFERENCES clients(id) ON DELETE RESTRICT,
    session_plan_id TEXT NULL REFERENCES session_plans(id) ON DELETE SET NULL,
    start_utc TEXT NOT NULL,
    end_utc TEXT NOT NULL,
    timezone TEXT NOT NULL CHECK (length(trim(timezone)) > 0),
    status TEXT NOT NULL DEFAULT 'scheduled' CHECK (status IN ('scheduled', 'completed', 'cancelled')),
    notes TEXT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CHECK (end_utc > start_utc)
);

CREATE TABLE IF NOT EXISTS appointment_services (
    id TEXT PRIMARY KEY NOT NULL,
    appointment_id TEXT NOT NULL REFERENCES appointments(id) ON DELETE CASCADE,
    service_id TEXT NOT NULL REFERENCES services(id) ON DELETE RESTRICT,
    service_name_snapshot TEXT NOT NULL CHECK (length(trim(service_name_snapshot)) > 0),
    service_description_snapshot TEXT NULL,
    duration_minutes_snapshot INTEGER NOT NULL CHECK (duration_minutes_snapshot >= 5 AND duration_minutes_snapshot <= 1440),
    sort_order INTEGER NOT NULL DEFAULT 0,
    UNIQUE (appointment_id, service_id)
);

CREATE INDEX IF NOT EXISTS idx_session_plans_client_status
    ON session_plans (client_id, status, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_appointments_start_utc
    ON appointments (start_utc ASC);

CREATE INDEX IF NOT EXISTS idx_appointments_client_start
    ON appointments (client_id, start_utc DESC);

CREATE INDEX IF NOT EXISTS idx_appointments_status_start
    ON appointments (status, start_utc ASC);

CREATE INDEX IF NOT EXISTS idx_appointments_plan_start
    ON appointments (session_plan_id, start_utc ASC);

CREATE INDEX IF NOT EXISTS idx_appointment_services_appointment
    ON appointment_services (appointment_id, sort_order ASC);
