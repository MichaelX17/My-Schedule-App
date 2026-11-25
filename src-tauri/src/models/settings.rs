use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Persisted application preferences (`app_settings` singleton).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct AppSettings {
    pub theme_mode: String,
    pub accent_color: String,
    pub reminders_enabled: bool,
    pub reminder_one_day_before: bool,
    pub reminder_one_hour_before: bool,
    pub reminder_day_of_launch: bool,
    pub os_notifications_enabled: bool,
    pub auto_backup_enabled: bool,
    pub auto_backup_retention: i64,
    pub default_appointment_duration_minutes: i64,
    pub updated_at: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_mode: "system".to_string(),
            accent_color: "slate".to_string(),
            reminders_enabled: true,
            reminder_one_day_before: true,
            reminder_one_hour_before: true,
            reminder_day_of_launch: true,
            os_notifications_enabled: true,
            auto_backup_enabled: true,
            auto_backup_retention: 30,
            default_appointment_duration_minutes: 60,
            updated_at: String::new(),
        }
    }
}

/// Partial or full update payload for application preferences.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateAppSettingsInput {
    pub theme_mode: Option<String>,
    pub accent_color: Option<String>,
    pub reminders_enabled: Option<bool>,
    pub reminder_one_day_before: Option<bool>,
    pub reminder_one_hour_before: Option<bool>,
    pub reminder_day_of_launch: Option<bool>,
    pub os_notifications_enabled: Option<bool>,
    pub auto_backup_enabled: Option<bool>,
    pub auto_backup_retention: Option<i64>,
    pub default_appointment_duration_minutes: Option<i64>,
}
