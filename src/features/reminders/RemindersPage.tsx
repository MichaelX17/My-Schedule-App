import { useState } from "react";
import { Link } from "react-router-dom";
import {
  AlertTriangle,
  Bell,
  BellOff,
  Check,
  CheckCheck,
  Clock,
  Info,
  RefreshCw,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { useOnboardingStatus } from "@/features/onboarding/hooks/useBusinessProfile";
import { extractErrorMessage } from "@/services/business";
import type { NotificationItem, NotificationKind } from "@/services/reminders";
import {
  useDismissNotification,
  useMarkAllNotificationsRead,
  useMarkNotificationRead,
  useNotificationsList,
  useRunReminderCheckNow,
  useUpcomingReminders,
} from "./hooks/useReminders";

function kindLabel(kind: NotificationKind | "one_day_before" | "one_hour_before"): string {
  switch (kind) {
    case "one_day_before":
      return "1 day before";
    case "one_hour_before":
      return "1 hour before";
    case "day_of_launch":
      return "Day-of launch";
  }
}

function OsDeliveryBadge({ item }: { item: NotificationItem }) {
  if (item.os_delivery_status === "delivered") {
    return (
      <span className="inline-flex items-center gap-1 rounded-full bg-emerald-500/15 px-2 py-0.5 text-[11px] font-medium text-emerald-700 dark:text-emerald-300">
        System notified
      </span>
    );
  }
  if (item.os_delivery_status === "failed") {
    return (
      <span
        className="inline-flex items-center gap-1 rounded-full bg-amber-500/15 px-2 py-0.5 text-[11px] font-medium text-amber-800 dark:text-amber-300"
        title={item.os_delivery_error ?? "System notification failed"}
      >
        <AlertTriangle className="size-3" aria-hidden="true" />
        In-app only (OS failed)
      </span>
    );
  }
  return (
    <span className="inline-flex items-center gap-1 rounded-full bg-muted px-2 py-0.5 text-[11px] font-medium text-muted-foreground">
      In-app
    </span>
  );
}

export function RemindersPage() {
  const [showDismissed, setShowDismissed] = useState(false);
  const [actionMessage, setActionMessage] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const { data: onboarding } = useOnboardingStatus();
  const timezone = onboarding?.business?.timezone ?? "UTC";

  const {
    data: notifications = [],
    isLoading: loadingNotifications,
    isError: notifError,
    error: notifErrObj,
  } = useNotificationsList(showDismissed);

  const { data: upcoming = [], isLoading: loadingUpcoming } =
    useUpcomingReminders();

  const markRead = useMarkNotificationRead();
  const markAllRead = useMarkAllNotificationsRead();
  const dismiss = useDismissNotification();
  const runCheck = useRunReminderCheckNow();

  const unreadCount = notifications.filter(
    (n) => !n.is_read && !n.is_dismissed
  ).length;

  const handleCheckNow = async () => {
    setActionError(null);
    setActionMessage(null);
    try {
      const summary = await runCheck.mutateAsync();
      const total = summary.timed_sent_count + summary.day_of_sent_count;
      setActionMessage(
        total > 0
          ? `Checked reminders: ${total} new notification(s) delivered.`
          : "Checked reminders: everything is up to date."
      );
    } catch (err) {
      setActionError(extractErrorMessage(err));
    }
  };

  const handleMarkAllRead = async () => {
    setActionError(null);
    try {
      await markAllRead.mutateAsync();
    } catch (err) {
      setActionError(extractErrorMessage(err));
    }
  };

  return (
    <div className="mx-auto w-full max-w-5xl p-6 sm:p-8 space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">
            Reminders &amp; Notifications
          </h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Automatic 1-day-before, 1-hour-before, and appointment-day launch
            reminders in business timezone ({timezone}).
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <Button
            type="button"
            variant="outline"
            onClick={handleCheckNow}
            disabled={runCheck.isPending}
          >
            <RefreshCw
              className={`size-4 ${runCheck.isPending ? "animate-spin" : ""}`}
            />
            Check Now
          </Button>
          {unreadCount > 0 && (
            <Button
              type="button"
              variant="secondary"
              onClick={handleMarkAllRead}
              disabled={markAllRead.isPending}
            >
              <CheckCheck className="size-4" />
              Mark All Read
            </Button>
          )}
        </div>
      </div>

      {/* Closed-app limitation disclosure (required by Step 07) */}
      <div
        role="note"
        className="flex items-start gap-3 rounded-xl border border-border bg-muted/40 p-4 text-sm"
      >
        <Info className="size-4 text-primary shrink-0 mt-0.5" aria-hidden="true" />
        <div className="space-y-1">
          <p className="font-medium text-foreground">
            How desktop reminders work
          </p>
          <p className="text-xs text-muted-foreground leading-relaxed">
            My Schedule App checks and delivers reminders while the application
            is running. When the application is completely closed, reminders
            cannot be delivered in the background unless a separate
            operating-system scheduled task is configured. Any due (non-expired)
            reminders and once-per-day appointment-day alerts are automatically
            caught up when you next open the app.
          </p>
        </div>
      </div>

      {actionMessage && (
        <div
          role="status"
          className="rounded-lg border border-border bg-card px-4 py-2.5 text-sm text-foreground"
        >
          {actionMessage}
        </div>
      )}

      {actionError && (
        <div
          role="alert"
          className="rounded-lg border border-destructive/40 bg-card px-4 py-2.5 text-sm text-destructive"
        >
          {actionError}
        </div>
      )}

      {/* Notification Center */}
      <section aria-label="Notification center" className="space-y-3">
        <div className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-2">
            <h2 className="text-base font-semibold">Notification Center</h2>
            {unreadCount > 0 && (
              <span className="rounded-full bg-primary/15 px-2 py-0.5 text-xs font-medium text-primary">
                {unreadCount} unread
              </span>
            )}
          </div>

          <div
            role="group"
            aria-label="Filter notifications"
            className="inline-flex rounded-md border border-border bg-background p-0.5"
          >
            <button
              type="button"
              aria-pressed={!showDismissed}
              onClick={() => setShowDismissed(false)}
              className={`rounded px-2.5 py-1 text-xs font-medium transition-colors ${
                !showDismissed
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:text-foreground"
              }`}
            >
              Active
            </button>
            <button
              type="button"
              aria-pressed={showDismissed}
              onClick={() => setShowDismissed(true)}
              className={`rounded px-2.5 py-1 text-xs font-medium transition-colors ${
                showDismissed
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:text-foreground"
              }`}
            >
              Include Dismissed
            </button>
          </div>
        </div>

        {notifError ? (
          <div
            role="alert"
            className="rounded-xl border border-destructive/40 bg-card p-4 text-sm text-destructive"
          >
            Failed to load notifications: {extractErrorMessage(notifErrObj)}
          </div>
        ) : loadingNotifications ? (
          <div className="rounded-xl border border-border bg-card p-8 text-center text-sm text-muted-foreground">
            Loading notifications...
          </div>
        ) : notifications.length === 0 ? (
          <div className="rounded-xl border border-border bg-card p-10 text-center space-y-2">
            <BellOff className="mx-auto size-8 text-muted-foreground/60" />
            <p className="text-sm font-medium">No notifications yet</p>
            <p className="text-xs text-muted-foreground">
              Upcoming appointment reminders will appear here automatically.
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {notifications.map((item) => (
              <article
                key={item.id}
                className={`rounded-xl border p-4 transition-colors ${
                  item.is_dismissed
                    ? "border-border bg-muted/30 opacity-75"
                    : item.is_read
                      ? "border-border bg-card"
                      : "border-primary/40 bg-primary/5"
                }`}
              >
                <div className="flex flex-col sm:flex-row sm:items-start justify-between gap-3">
                  <div className="space-y-1 min-w-0">
                    <div className="flex flex-wrap items-center gap-2">
                      <Bell
                        className={`size-4 shrink-0 ${
                          item.is_read ? "text-muted-foreground" : "text-primary"
                        }`}
                        aria-hidden="true"
                      />
                      <h3 className="text-sm font-semibold">{item.title}</h3>
                      <span className="rounded-full bg-secondary px-2 py-0.5 text-[11px] font-medium text-secondary-foreground">
                        {kindLabel(item.kind)}
                      </span>
                      <OsDeliveryBadge item={item} />
                      {!item.is_read && !item.is_dismissed && (
                        <span className="rounded-full bg-primary px-2 py-0.5 text-[10px] font-semibold text-primary-foreground">
                          Unread
                        </span>
                      )}
                    </div>

                    <p className="text-sm text-foreground">{item.body}</p>

                    {item.os_delivery_status === "failed" &&
                      item.os_delivery_error && (
                        <p className="text-xs text-amber-700 dark:text-amber-300">
                          {item.os_delivery_error}
                        </p>
                      )}
                  </div>

                  <div className="flex items-center gap-1.5 shrink-0">
                    {item.appointment_id && (
                      <Link
                        to={`/appointments/${item.appointment_id}`}
                        className="inline-flex h-8 items-center justify-center rounded-md border border-border bg-background px-3 text-xs font-medium hover:bg-accent hover:text-accent-foreground transition-colors"
                      >
                        View Appointment
                      </Link>
                    )}
                    {!item.is_read && !item.is_dismissed && (
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        disabled={markRead.isPending}
                        onClick={() => void markRead.mutateAsync(item.id)}
                      >
                        <Check className="size-4" />
                        Mark Read
                      </Button>
                    )}
                    {!item.is_dismissed && (
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        aria-label={`Dismiss ${item.title}`}
                        disabled={dismiss.isPending}
                        onClick={() => void dismiss.mutateAsync(item.id)}
                      >
                        <X className="size-4" />
                      </Button>
                    )}
                  </div>
                </div>
              </article>
            ))}
          </div>
        )}
      </section>

      {/* Upcoming Scheduled Reminders */}
      <section aria-label="Upcoming scheduled reminders" className="space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="text-base font-semibold">
            Upcoming Scheduled Reminders
          </h2>
          <span className="text-xs text-muted-foreground">
            {upcoming.length} pending
          </span>
        </div>

        {loadingUpcoming ? (
          <div className="rounded-xl border border-border bg-card p-6 text-center text-sm text-muted-foreground">
            Loading scheduled reminders...
          </div>
        ) : upcoming.length === 0 ? (
          <div className="rounded-xl border border-border bg-card p-8 text-center text-sm text-muted-foreground">
            No pending reminders queued. Reminders are created automatically
            when you schedule an appointment.
          </div>
        ) : (
          <div className="rounded-xl border border-border bg-card divide-y divide-border overflow-hidden">
            {upcoming.map((rem) => (
              <div
                key={rem.id}
                className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 px-4 py-3"
              >
                <div className="min-w-0">
                  <div className="flex flex-wrap items-center gap-2">
                    <Clock
                      className="size-3.5 text-muted-foreground shrink-0"
                      aria-hidden="true"
                    />
                    <span className="text-sm font-medium">
                      {rem.client_name}
                    </span>
                    <span className="rounded-full bg-secondary px-2 py-0.5 text-[11px] font-medium text-secondary-foreground">
                      {kindLabel(rem.reminder_type)}
                    </span>
                  </div>
                  <p className="text-xs text-muted-foreground mt-0.5 truncate">
                    {rem.services_summary} · Appointment{" "}
                    {rem.appointment_start_local.replace("T", " ")}
                  </p>
                </div>

                <div className="flex items-center gap-3 shrink-0">
                  <span className="text-xs font-medium tabular-nums text-muted-foreground">
                    Due {rem.scheduled_for_local.replace("T", " ")}
                  </span>
                  <Link
                    to={`/appointments/${rem.appointment_id}`}
                    className="inline-flex h-8 items-center justify-center rounded-md px-3 text-xs font-medium hover:bg-accent hover:text-accent-foreground transition-colors"
                  >
                    Open
                  </Link>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
