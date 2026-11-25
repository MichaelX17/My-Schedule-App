import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type {
  AppointmentDetail,
  SessionPlanSummary,
} from "@/services/appointments";
import { AppointmentsPage } from "../AppointmentsPage";

const mockDeleteAppointment = vi.fn<(id: string) => Promise<void>>();
const mockDeleteSessionPlan =
  vi.fn<(id: string, deleteLinked?: boolean) => Promise<void>>();

const MOCK_APPT: AppointmentDetail = {
  id: "appt-plan-1",
  client_id: "c1",
  client_name: "Lucia Caminos",
  client_is_archived: false,
  session_plan_id: "plan-1",
  session_plan_title: "8-Session Massage Pack",
  start_utc: "2026-12-01T09:00:00.000Z",
  end_utc: "2026-12-01T10:00:00.000Z",
  start_local: "2026-12-01T10:00",
  end_local: "2026-12-01T11:00",
  timezone: "Europe/Madrid",
  status: "scheduled",
  notes: "Session 1 of 8",
  services: [
    {
      id: "as1",
      appointment_id: "appt-plan-1",
      service_id: "s1",
      service_name_snapshot: "Masaje Normal",
      service_description_snapshot: null,
      duration_minutes_snapshot: 60,
      sort_order: 0,
    },
  ],
  created_at: "2026-11-25T00:00:00.000Z",
  updated_at: "2026-11-25T00:00:00.000Z",
};

const MOCK_PLAN: SessionPlanSummary = {
  id: "plan-1",
  client_id: "c1",
  client_name: "Lucia Caminos",
  title: "8-Session Massage Pack",
  description: null,
  planned_sessions: 8,
  status: "active",
  linked_total: 8,
  linked_scheduled: 8,
  linked_completed: 0,
  linked_cancelled: 0,
  created_at: "2026-11-25T00:00:00.000Z",
  updated_at: "2026-11-25T00:00:00.000Z",
};

vi.mock("@/services/tauri", () => ({
  isTauriRuntime: () => true,
  invokeCommand: vi.fn(),
}));

vi.mock("@/services/appointments", async () => {
  const actual = await vi.importActual<typeof import("@/services/appointments")>(
    "@/services/appointments"
  );
  return {
    ...actual,
    listAppointments: vi.fn(async () => [MOCK_APPT]),
    listSessionPlans: vi.fn(async () => [MOCK_PLAN]),
    deleteAppointment: (id: string) => mockDeleteAppointment(id),
    deleteSessionPlan: (id: string, deleteLinked?: boolean) =>
      mockDeleteSessionPlan(id, deleteLinked),
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

function renderPage() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>
        <AppointmentsPage />
      </MemoryRouter>
    </QueryClientProvider>
  );
}

describe("AppointmentsPage deletion flows", () => {
  beforeEach(() => {
    mockDeleteAppointment.mockReset().mockResolvedValue(undefined);
    mockDeleteSessionPlan.mockReset().mockResolvedValue(undefined);
  });

  it("confirms and deletes a session-plan appointment", async () => {
    const user = userEvent.setup();
    renderPage();

    const deleteBtn = await screen.findByRole("button", {
      name: /Delete appointment for Lucia Caminos/i,
    });
    await user.click(deleteBtn);

    expect(
      screen.getByText(/Delete appointment for Lucia Caminos\?/i)
    ).toBeInTheDocument();
    expect(
      screen.getByText(/update the session plan to remove this session/i)
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: "Delete Appointment" })
    );

    await waitFor(() => {
      expect(mockDeleteAppointment).toHaveBeenCalledWith("appt-plan-1");
    });
  });

  it("confirms and deletes a session plan with optional linked-appointment cascade", async () => {
    const user = userEvent.setup();
    renderPage();

    await user.click(
      await screen.findByRole("button", { name: /Session Plans \(1\)/i })
    );

    await user.click(screen.getByRole("button", { name: "Delete" }));

    expect(
      screen.getByRole("heading", { name: /Delete session plan/i })
    ).toBeInTheDocument();

    // Select cascade deletion of linked appointments
    await user.click(
      screen.getByLabelText(/Delete plan and all 8 linked appointments/i)
    );

    await user.click(
      screen.getByRole("button", { name: "Delete Plan & Appointments" })
    );

    await waitFor(() => {
      expect(mockDeleteSessionPlan).toHaveBeenCalledWith("plan-1", true);
    });
  });
});
