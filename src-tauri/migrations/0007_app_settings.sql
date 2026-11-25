-- Singleton application settings table for local operator preferences.
-- Persists appearance (theme + accent), reminder preferences, automatic backup
-- retention, and default appointment duration.

CREATE TABLE IF NOT EXISTS app_settings (
    singleton_key INTEGER PRIMARY KEY NOT NULL DEFAULT 1 CHECK (singleton_key = 1),
    theme_mode TEXT NOT NULL DEFAULT 'system' CHECK (theme_mode IN ('light', 'dark', 'system')),
    accent_color TEXT NOT NULL DEFAULT 'slate' CHECK (accent_color IN ('slate', 'blue', 'emerald', 'violet', 'amber', 'rose')),
    reminders_enabled INTEGER NOT NULL DEFAULT 1 CHECK (reminders_enabled IN (0, 1)),
    reminder_one_day_before INTEGER NOT NULL DEFAULT 1 CHECK (reminder_one_day_before IN (0, 1)),
    reminder_one_hour_before INTEGER NOT NULL DEFAULT 1 CHECK (reminder_one_hour_before IN (0, 1)),
    reminder_day_of_launch INTEGER NOT NULL DEFAULT 1 CHECK (reminder_day_of_launch IN (0, 1)),
    os_notifications_enabled INTEGER NOT NULL DEFAULT 1 CHECK (os_notifications_enabled IN (0, 1)),
    auto_backup_enabled INTEGER NOT NULL DEFAULT 1 CHECK (auto_backup_enabled IN (0, 1)),
    auto_backup_retention INTEGER NOT NULL DEFAULT 30 CHECK (auto_backup_retention >= 1 AND auto_backup_retention <= 365),
    default_appointment_duration_minutes INTEGER NOT NULL DEFAULT 60 CHECK (default_appointment_duration_minutes >= 5 AND default_appointment_duration_minutes <= 1440),
    updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO app_settings (singleton_key, updated_at)
VALUES (1, CURRENT_TIMESTAMP);
