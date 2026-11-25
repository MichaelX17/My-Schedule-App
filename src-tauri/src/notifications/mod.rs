use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// Outcome of attempting to deliver an OS-level desktop notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationDeliveryOutcome {
    Delivered,
    Failed(String),
    Skipped(String),
}

impl NotificationDeliveryOutcome {
    pub fn status_str(&self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Failed(_) => "failed",
            Self::Skipped(_) => "skipped",
        }
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Delivered => None,
            Self::Failed(msg) | Self::Skipped(msg) => Some(msg.as_str()),
        }
    }
}

/// Abstraction over OS notification delivery so the scheduler can be tested
/// deterministically without requiring a live desktop notification daemon.
pub trait NotificationSender: Send + Sync {
    fn send(&self, title: &str, body: &str) -> NotificationDeliveryOutcome;
}

/// Production desktop notification sender backed by `tauri-plugin-notification`
/// (Windows toast notifications on Windows, D-Bus/libnotify on Linux, UNUserNotificationCenter on macOS).
pub struct TauriNotificationSender {
    app_handle: AppHandle,
}

impl TauriNotificationSender {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl NotificationSender for TauriNotificationSender {
    fn send(&self, title: &str, body: &str) -> NotificationDeliveryOutcome {
        match self
            .app_handle
            .notification()
            .builder()
            .title(title)
            .body(body)
            .show()
        {
            Ok(()) => NotificationDeliveryOutcome::Delivered,
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    "OS notification delivery failed; retaining in-app notification"
                );
                NotificationDeliveryOutcome::Failed(
                    "System notification could not be delivered (check OS notification permissions).".into(),
                )
            }
        }
    }
}

/// No-op notification sender used in automated tests or headless contexts.
#[derive(Debug, Default)]
pub struct NoopNotificationSender;

impl NotificationSender for NoopNotificationSender {
    fn send(&self, _title: &str, _body: &str) -> NotificationDeliveryOutcome {
        NotificationDeliveryOutcome::Skipped(
            "System notifications are disabled in headless/test mode.".into(),
        )
    }
}
