import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type {
  NotificationItem,
  ReminderRunSummary,
  ReminderSummary,
} from "@/services/reminders";
import { RemindersPage } from "../RemindersPage";

const mockListNotifications = vi.fn<
  (includeDismissed?: boolean) => Promise<NotificationItem[]>
>();
const mockListUpcoming = vi.fn<() => Promise<ReminderSummary[]>>();
const mockMarkRead = vi.fn<(id: string) => Promise<NotificationItem>>();
const mockDismiss = vi.fn<(id: string) => Promise<NotificationItem>>();
const mockRunCheck = vi.fn<() => Promise<ReminderRunSummary>>();

vi.mock("@/services/tauri", () => ({
  isTauriRuntime: () => true,
  invokeCommand: vi.fn(),
}));

vi.mock("@/services/reminders", async () => {
  const actual = await vi.importActual<typeof import("@/services/reminders")>(
    "@/services/reminders"
  );
  return {
    ...actual,
    listNotifications: (includeDismissed?: boolean) =>
      mockListNotifications(includeDismissed),
    listUpcomingReminders: () => mockListUpcoming(),
    markNotificationRead: (id: string) => mockMarkRead(id),
    dismissNotification: (id: string) => mockDismiss(id),
    runReminderCheckNow: () => mockRunCheck(),
  };
});

vi.mock("@/features/onboarding/hooks/useBusinessProfile", () => ({
  useOnboardingStatus: () => ({
    data: {
      completed: true,
      detected_timezone: "Europe/Madrid",
      business: { timezone: "Europe/Madrid", name: "Studio Test" },
    },
  }),
}));

const sampleNotification: NotificationItem = {
  id: "notif-1",
  reminder_id: "rem-1",
  appointment_id: "appt-1",
  kind: "one_hour_before",
  idempotency_key: "notif:one_hour_before:appt-1:2026-06-15T09:00:00.000Z",
  title: "Appointment in 1 Hour",
  body: "Elena Vance — today at 10:00 (Consultation)",
  is_read: false,
  is_dismissed: false,
  os_delivery_status: "failed",
  os_delivery_error: "System notification could not be delivered.",
  created_at: "2026-06-15T09:00:00.000Z",
  read_at: null,
  dismissed_at: null,
};

const sampleUpcoming: ReminderSummary = {
  id: "rem-2",
  appointment_id: "appt-2",
  reminder_type: "one_day_before",
  scheduled_for_utc: "2026-06-18T08:00:00.000Z",
  scheduled_for_local: "2026-06-18T10:00",
  appointment_start_local: "2026-06-19T10:00",
  timezone: "Europe/Madrid",
  status: "pending",
  client_name: "Marcus Holloway",
  services_summary: "Strategy Review",
};

function renderPage() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={["/reminders"]}>
        <Routes>
          <Route path="/reminders" element={<RemindersPage />} />
          <Route
            path="/appointments/:appointmentId"
            element={<div>Appointment Target</div>}
          />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>
  );
}

describe("RemindersPage", () => {
  beforeEach(() => {
    mockListNotifications.mockReset();
    mockListUpcoming.mockReset();
    mockMarkRead.mockReset();
    mockDismiss.mockReset();
    mockRunCheck.mockReset();

    mockListNotifications.mockResolvedValue([sampleNotification]);
    mockListUpcoming.mockResolvedValue([sampleUpcoming]);
    mockMarkRead.mockResolvedValue({ ...sampleNotification, is_read: true });
    mockDismiss.mockResolvedValue({
      ...sampleNotification,
      is_read: true,
      is_dismissed: true,
    });
    mockRunCheck.mockResolvedValue({
      expired_count: 0,
      timed_sent_count: 0,
      day_of_sent_count: 0,
      os_delivered_count: 0,
      os_failed_count: 0,
    });
  });

  it("renders closed-app disclosure, notifications with OS failure fallback, and upcoming reminders", async () => {
    const user = userEvent.setup();
    renderPage();

    // Closed-app limitation disclosure is visible
    expect(
      await screen.findByText(/When the application is completely closed/i)
    ).toBeInTheDocument();

    // Active notification with OS delivery fallback badge
    expect(
      await screen.findByText("Appointment in 1 Hour")
    ).toBeInTheDocument();
    expect(screen.getByText(/In-app only \(OS failed\)/i)).toBeInTheDocument();
    expect(await screen.findByText("Marcus Holloway")).toBeInTheDocument();

    // Mark notification as read
    await user.click(screen.getByRole("button", { name: /Mark Read/i }));
    await waitFor(() => {
      expect(mockMarkRead).toHaveBeenCalledWith("notif-1");
    });

    // Trigger manual check
    await user.click(screen.getByRole("button", { name: /Check Now/i }));
    await waitFor(() => {
      expect(mockRunCheck).toHaveBeenCalledTimes(1);
    });
    expect(
      await screen.findByText(/everything is up to date/i)
    ).toBeInTheDocument();
  });
});
