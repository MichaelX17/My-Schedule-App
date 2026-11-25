use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Timed reminder types stored in the `reminders` table.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReminderType {
    OneDayBefore,
    OneHourBefore,
}

impl ReminderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneDayBefore => "one_day_before",
            Self::OneHourBefore => "one_hour_before",
        }
    }
}

/// Lifecycle state of a scheduled reminder.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReminderStatus {
    Pending,
    Sent,
    Cancelled,
    Expired,
}

impl ReminderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Sent => "sent",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
        }
    }
}

/// Notification category persisted in the `notifications` table.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationKind {
    OneDayBefore,
    OneHourBefore,
    DayOfLaunch,
}

impl NotificationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneDayBefore => "one_day_before",
            Self::OneHourBefore => "one_hour_before",
            Self::DayOfLaunch => "day_of_launch",
        }
    }
}

/// Raw database row for a reminder joined with appointment and client context.
#[derive(Debug, Clone, FromRow)]
pub struct DueReminderRow {
    pub id: String,
    pub appointment_id: String,
    pub reminder_type: String,
    pub scheduled_for_utc: String,
    pub expires_at_utc: String,
    pub status: String,
    pub idempotency_key: String,
    pub appointment_start_utc: String,
    pub appointment_end_utc: String,
    pub appointment_status: String,
    pub client_name: String,
}

/// Upcoming reminder DTO returned to the UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReminderSummary {
    pub id: String,
    pub appointment_id: String,
    pub reminder_type: String,
    pub scheduled_for_utc: String,
    pub scheduled_for_local: String,
    pub appointment_start_local: String,
    pub timezone: String,
    pub status: String,
    pub client_name: String,
    pub services_summary: String,
}

/// Persisted in-app notification item returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct NotificationItem {
    pub id: String,
    pub reminder_id: Option<String>,
    pub appointment_id: Option<String>,
    pub kind: String,
    pub idempotency_key: String,
    pub title: String,
    pub body: String,
    pub is_read: bool,
    pub is_dismissed: bool,
    pub os_delivery_status: String,
    pub os_delivery_error: Option<String>,
    pub created_at: String,
    pub read_at: Option<String>,
    pub dismissed_at: Option<String>,
}

/// Operational summary of a scheduler catch-up or tick run.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReminderRunSummary {
    pub expired_count: u64,
    pub timed_sent_count: u64,
    pub day_of_sent_count: u64,
    pub os_delivered_count: u64,
    pub os_failed_count: u64,
}

impl ReminderRunSummary {
    pub fn total_new_notifications(&self) -> u64 {
        self.timed_sent_count + self.day_of_sent_count
    }
}
