-- Persistent reminder scheduling, in-app/OS notification queue, and daily launch marker.
-- Unique idempotency keys prevent duplicate reminders/notifications across restarts,
-- clock adjustments, and repeated scheduler ticks.

CREATE TABLE IF NOT EXISTS reminders (
    id TEXT PRIMARY KEY NOT NULL,
    appointment_id TEXT NOT NULL REFERENCES appointments(id) ON DELETE CASCADE,
    reminder_type TEXT NOT NULL CHECK (reminder_type IN ('one_day_before', 'one_hour_before')),
    scheduled_for_utc TEXT NOT NULL,
    expires_at_utc TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'sent', 'cancelled', 'expired')),
    idempotency_key TEXT NOT NULL UNIQUE,
    sent_at TEXT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY NOT NULL,
    reminder_id TEXT NULL REFERENCES reminders(id) ON DELETE SET NULL,
    appointment_id TEXT NULL REFERENCES appointments(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('one_day_before', 'one_hour_before', 'day_of_launch')),
    idempotency_key TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    body TEXT NOT NULL CHECK (length(trim(body)) > 0),
    is_read INTEGER NOT NULL DEFAULT 0 CHECK (is_read IN (0, 1)),
    is_dismissed INTEGER NOT NULL DEFAULT 0 CHECK (is_dismissed IN (0, 1)),
    os_delivery_status TEXT NOT NULL DEFAULT 'pending' CHECK (os_delivery_status IN ('pending', 'delivered', 'failed', 'skipped')),
    os_delivery_error TEXT NULL,
    created_at TEXT NOT NULL,
    read_at TEXT NULL,
    dismissed_at TEXT NULL
);

CREATE TABLE IF NOT EXISTS reminder_daily_runs (
    business_local_date TEXT PRIMARY KEY NOT NULL,
    processed_at TEXT NOT NULL,
    appointments_notified INTEGER NOT NULL DEFAULT 0 CHECK (appointments_notified >= 0)
);

CREATE INDEX IF NOT EXISTS idx_reminders_status_due
    ON reminders (status, scheduled_for_utc ASC);

CREATE INDEX IF NOT EXISTS idx_reminders_appointment_status
    ON reminders (appointment_id, status);

CREATE INDEX IF NOT EXISTS idx_notifications_active
    ON notifications (is_dismissed, is_read, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_notifications_appointment
    ON notifications (appointment_id);
