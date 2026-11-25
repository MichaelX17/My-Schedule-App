import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type {
  CalendarRangeInput,
  CalendarRangeResponse,
} from "@/services/calendar";
import { CalendarPage } from "../CalendarPage";

const mockGetCalendarRange = vi.fn<
  (input: CalendarRangeInput) => Promise<CalendarRangeResponse>
>();

vi.mock("@/services/tauri", () => ({
  isTauriRuntime: () => true,
  invokeCommand: vi.fn(),
}));

vi.mock("@/services/calendar", async () => {
  const actual = await vi.importActual<typeof import("@/services/calendar")>(
    "@/services/calendar"
  );
  return {
    ...actual,
    getCalendarRange: (input: CalendarRangeInput) =>
      mockGetCalendarRange(input),
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

vi.mock("@/features/clients/hooks/useClients", () => ({
  useClientsList: () => ({ data: { items: [], total: 0 } }),
}));

vi.mock("@/features/services/hooks/useServices", () => ({
  useServicesList: () => ({ data: { items: [], total: 0 } }),
}));

vi.mock("@/features/appointments/hooks/useAppointments", () => ({
  useSessionPlansList: () => ({ data: [] }),
  useCreateAppointment: () => ({ mutateAsync: vi.fn(), isPending: false }),
}));

function buildMockResponse(input: CalendarRangeInput): CalendarRangeResponse {
  const anchor = input.anchor_date ?? "2026-06-17";
  const rangeByView: Record<CalendarRangeInput["view"], [string, string]> = {
    day: [anchor, anchor],
    week: ["2026-06-15", "2026-06-21"],
    month: ["2026-06-01", "2026-07-05"],
    agenda: [anchor, "2026-06-30"],
  };
  const [rangeStart, rangeEnd] = rangeByView[input.view];

  return {
    view: input.view,
    timezone: "Europe/Madrid",
    today_local: "2026-06-17",
    anchor_date: anchor,
    range_start_local: rangeStart,
    range_end_local: rangeEnd,
    range_start_utc: "2026-06-14T22:00:00.000Z",
    range_end_utc_exclusive: "2026-06-21T22:00:00.000Z",
    appointments: [
      {
        id: "appt-123",
        client_id: "c1",
        client_name: "Elena Vance",
        client_is_archived: false,
        session_plan_id: null,
        session_plan_title: null,
        start_utc: "2026-06-17T08:00:00.000Z",
        end_utc: "2026-06-17T09:00:00.000Z",
        start_local: "2026-06-17T10:00",
        end_local: "2026-06-17T11:00",
        timezone: "Europe/Madrid",
        status: "scheduled",
        notes: null,
        services: [
          {
            id: "as1",
            appointment_id: "appt-123",
            service_id: "s1",
            service_name_snapshot: "Initial Consultation",
            service_description_snapshot: null,
            duration_minutes_snapshot: 60,
            sort_order: 0,
          },
        ],
        created_at: "2026-06-01T00:00:00.000Z",
        updated_at: "2026-06-01T00:00:00.000Z",
      },
    ],
  };
}

function renderCalendar(initialEntry = "/calendar?view=week&date=2026-06-17") {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[initialEntry]}>
        <Routes>
          <Route path="/calendar" element={<CalendarPage />} />
          <Route
            path="/appointments/:appointmentId"
            element={<div>Appointment Detail Target</div>}
          />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>
  );
}

describe("CalendarPage", () => {
  beforeEach(() => {
    mockGetCalendarRange.mockReset();
    mockGetCalendarRange.mockImplementation(async (input) =>
      buildMockResponse(input)
    );
  });

  it("switches across week, day, month, and agenda views and navigates periods", async () => {
    const user = userEvent.setup();
    renderCalendar();

    // Initial week view loads appointment with accessible client/service/status text
    expect(await screen.findByText("Elena Vance")).toBeInTheDocument();
    expect(screen.getByText("Initial Consultation")).toBeInTheDocument();
    expect(screen.getByText("Jun 15 – 21, 2026")).toBeInTheDocument();

    // Switch to Day view
    await user.click(screen.getByRole("button", { name: "Day" }));
    await waitFor(() => {
      expect(mockGetCalendarRange).toHaveBeenCalledWith(
        expect.objectContaining({ view: "day", anchor_date: "2026-06-17" })
      );
    });
    expect(screen.getByText("Wednesday, June 17, 2026")).toBeInTheDocument();

    // Navigate Next day -> 2026-06-18
    await user.click(screen.getByRole("button", { name: "Next period" }));
    await waitFor(() => {
      expect(mockGetCalendarRange).toHaveBeenCalledWith(
        expect.objectContaining({ view: "day", anchor_date: "2026-06-18" })
      );
    });

    // Click Today -> returns to 2026-06-17
    await user.click(screen.getByRole("button", { name: "Today" }));
    await waitFor(() => {
      expect(mockGetCalendarRange).toHaveBeenCalledWith(
        expect.objectContaining({ view: "day", anchor_date: "2026-06-17" })
      );
    });

    // Switch to Month view
    await user.click(screen.getByRole("button", { name: "Month" }));
    await waitFor(() => {
      expect(mockGetCalendarRange).toHaveBeenCalledWith(
        expect.objectContaining({ view: "month" })
      );
    });
    expect(screen.getByText("June 2026")).toBeInTheDocument();

    // Switch to Agenda view
    await user.click(screen.getByRole("button", { name: "Agenda" }));
    await waitFor(() => {
      expect(mockGetCalendarRange).toHaveBeenCalledWith(
        expect.objectContaining({ view: "agenda" })
      );
    });
  });

  it("opens appointment detail workflow when clicking an appointment", async () => {
    const user = userEvent.setup();
    renderCalendar("/calendar?view=agenda&date=2026-06-17");

    const apptLink = await screen.findByRole("link", {
      name: /10:00–11:00, Elena Vance, Initial Consultation, status Scheduled/i,
    });
    await user.click(apptLink);

    expect(
      await screen.findByText("Appointment Detail Target")
    ).toBeInTheDocument();
  });
});
