import { Link } from "react-router-dom";
import { Bell, Check, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  useDismissNotification,
  useMarkNotificationRead,
  useNotificationsList,
} from "../hooks/useReminders";

export function NotificationToastBanner() {
  const { data: notifications = [] } = useNotificationsList(false);
  const markRead = useMarkNotificationRead();
  const dismiss = useDismissNotification();

  const unread = notifications.filter((n) => !n.is_read && !n.is_dismissed);
  const latest = unread[0];

  if (!latest) return null;

  return (
    <div
      role="status"
      aria-live="polite"
      className="border-b border-border bg-primary/5 px-4 py-2.5"
    >
      <div className="mx-auto max-w-6xl flex flex-col sm:flex-row sm:items-center justify-between gap-2">
        <div className="flex items-start sm:items-center gap-2.5 min-w-0">
          <Bell className="size-4 text-primary shrink-0 mt-0.5 sm:mt-0" aria-hidden="true" />
          <div className="min-w-0 text-sm">
            <span className="font-semibold">{latest.title}:</span>{" "}
            <span className="text-muted-foreground">{latest.body}</span>
            {unread.length > 1 && (
              <Link
                to="/reminders"
                className="ml-2 text-xs font-medium text-primary hover:underline"
              >
                +{unread.length - 1} more
              </Link>
            )}
          </div>
        </div>

        <div className="flex items-center gap-1.5 shrink-0">
          {latest.appointment_id && (
            <Link
              to={`/appointments/${latest.appointment_id}`}
              className="inline-flex h-7 items-center justify-center rounded-md border border-border bg-background px-2.5 text-xs font-medium hover:bg-accent hover:text-accent-foreground transition-colors"
            >
              View
            </Link>
          )}
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="h-7 text-xs"
            disabled={markRead.isPending}
            onClick={() => void markRead.mutateAsync(latest.id)}
          >
            <Check className="size-3.5" />
            Read
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="size-7"
            aria-label="Dismiss notification"
            disabled={dismiss.isPending}
            onClick={() => void dismiss.mutateAsync(latest.id)}
          >
            <X className="size-3.5" />
          </Button>
        </div>
      </div>
    </div>
  );
}
