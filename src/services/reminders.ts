import { invokeCommand } from "@/services/tauri";

export type NotificationKind =
  | "one_day_before"
  | "one_hour_before"
  | "day_of_launch";

export type OsDeliveryStatus = "pending" | "delivered" | "failed" | "skipped";

export interface NotificationItem {
  id: string;
  reminder_id: string | null;
  appointment_id: string | null;
  kind: NotificationKind;
  idempotency_key: string;
  title: string;
  body: string;
  is_read: boolean;
  is_dismissed: boolean;
  os_delivery_status: OsDeliveryStatus;
  os_delivery_error: string | null;
  created_at: string;
  read_at: string | null;
  dismissed_at: string | null;
}

export interface ReminderSummary {
  id: string;
  appointment_id: string;
  reminder_type: "one_day_before" | "one_hour_before";
  scheduled_for_utc: string;
  scheduled_for_local: string;
  appointment_start_local: string;
  timezone: string;
  status: "pending" | "sent" | "cancelled" | "expired";
  client_name: string;
  services_summary: string;
}

export interface ReminderRunSummary {
  expired_count: number;
  timed_sent_count: number;
  day_of_sent_count: number;
  os_delivered_count: number;
  os_failed_count: number;
}

export async function listNotifications(
  includeDismissed = false
): Promise<NotificationItem[]> {
  return invokeCommand<NotificationItem[]>("list_notifications", {
    includeDismissed,
  });
}

export async function getUnreadNotificationCount(): Promise<number> {
  return invokeCommand<number>("get_unread_notification_count");
}

export async function markNotificationRead(
  id: string
): Promise<NotificationItem> {
  return invokeCommand<NotificationItem>("mark_notification_read", { id });
}

export async function markAllNotificationsRead(): Promise<number> {
  return invokeCommand<number>("mark_all_notifications_read");
}

export async function dismissNotification(
  id: string
): Promise<NotificationItem> {
  return invokeCommand<NotificationItem>("dismiss_notification", { id });
}

export async function listUpcomingReminders(): Promise<ReminderSummary[]> {
  return invokeCommand<ReminderSummary[]>("list_upcoming_reminders");
}

export async function runReminderCheckNow(): Promise<ReminderRunSummary> {
  return invokeCommand<ReminderRunSummary>("run_reminder_check_now");
}
