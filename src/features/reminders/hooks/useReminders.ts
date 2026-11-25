import { useEffect } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import {
  dismissNotification,
  getUnreadNotificationCount,
  listNotifications,
  listUpcomingReminders,
  markAllNotificationsRead,
  markNotificationRead,
  runReminderCheckNow,
} from "@/services/reminders";
import { isTauriRuntime } from "@/services/tauri";

export const REMINDER_QUERY_KEYS = {
  all: ["reminders"] as const,
  notifications: (includeDismissed: boolean) =>
    ["reminders", "notifications", includeDismissed] as const,
  unreadCount: ["reminders", "unread-count"] as const,
  upcoming: ["reminders", "upcoming"] as const,
};

/** Subscribes to backend `"reminders-updated"` events so the UI refreshes immediately. */
export function useReminderEventsSubscription() {
  const queryClient = useQueryClient();

  useEffect(() => {
    if (!isTauriRuntime()) return;

    let active = true;
    let unlistenFn: (() => void) | null = null;

    listen("reminders-updated", () => {
      void queryClient.invalidateQueries({ queryKey: REMINDER_QUERY_KEYS.all });
    })
      .then((unlisten) => {
        if (!active) {
          unlisten();
        } else {
          unlistenFn = unlisten;
        }
      })
      .catch(() => {
        // Ignore listener setup errors outside full desktop webview contexts.
      });

    return () => {
      active = false;
      unlistenFn?.();
    };
  }, [queryClient]);
}

export function useNotificationsList(includeDismissed = false) {
  return useQuery({
    queryKey: REMINDER_QUERY_KEYS.notifications(includeDismissed),
    queryFn: () => listNotifications(includeDismissed),
    enabled: isTauriRuntime(),
    refetchInterval: 60_000,
  });
}

export function useUnreadNotificationCount() {
  return useQuery({
    queryKey: REMINDER_QUERY_KEYS.unreadCount,
    queryFn: () => getUnreadNotificationCount(),
    enabled: isTauriRuntime(),
    refetchInterval: 60_000,
  });
}

export function useUpcomingReminders() {
  return useQuery({
    queryKey: REMINDER_QUERY_KEYS.upcoming,
    queryFn: () => listUpcomingReminders(),
    enabled: isTauriRuntime(),
  });
}

export function useMarkNotificationRead() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => markNotificationRead(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: REMINDER_QUERY_KEYS.all,
      });
    },
  });
}

export function useMarkAllNotificationsRead() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => markAllNotificationsRead(),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: REMINDER_QUERY_KEYS.all,
      });
    },
  });
}

export function useDismissNotification() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => dismissNotification(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: REMINDER_QUERY_KEYS.all,
      });
    },
  });
}

export function useRunReminderCheckNow() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => runReminderCheckNow(),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: REMINDER_QUERY_KEYS.all,
      });
    },
  });
}
