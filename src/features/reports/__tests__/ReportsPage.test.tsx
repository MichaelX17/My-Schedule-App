import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type {
  OperationalReport,
  OperationalReportInput,
} from "@/services/reports";
import { ReportsPage } from "../ReportsPage";

const mockGetOperationalReport = vi.fn<
  (input: OperationalReportInput) => Promise<OperationalReport>
>();

vi.mock("@/services/tauri", () => ({
  isTauriRuntime: () => true,
  invokeCommand: vi.fn(),
}));

vi.mock("@/services/reports", async () => {
  const actual = await vi.importActual<typeof import("@/services/reports")>(
    "@/services/reports"
  );
  return {
    ...actual,
    getOperationalReport: (input: OperationalReportInput) =>
      mockGetOperationalReport(input),
  };
});

function buildReport(input: OperationalReportInput): OperationalReport {
  const isMonthly = input.period_type === "monthly";
  return {
    period_type: input.period_type,
    timezone: "Europe/Madrid",
    anchor_date: input.anchor_date ?? "2026-06-17",
    period_start_local: isMonthly ? "2026-06-01" : "2026-06-15",
    period_end_local: isMonthly ? "2026-06-30" : "2026-06-21",
    period_start_utc: "2026-06-14T22:00:00.000Z",
    period_end_utc_exclusive: "2026-06-21T22:00:00.000Z",
    counts: { total: 4, scheduled: 1, completed: 2, cancelled: 1 },
    completion_rate_percent: 50.0,
    total_active_minutes: 180,
    unique_clients_served: 2,
    daily_breakdown: [
      {
        date_local: "2026-06-15",
        scheduled: 1,
        completed: 1,
        cancelled: 0,
        total: 2,
      },
      {
        date_local: "2026-06-16",
        scheduled: 0,
        completed: 1,
        cancelled: 1,
        total: 2,
      },
    ],
    top_services: [
      {
        service_name: "Initial Consultation",
        appointment_count: 3,
        total_minutes: 180,
      },
    ],
  };
}

function renderReports() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <ReportsPage />
    </QueryClientProvider>
  );
}

describe("ReportsPage", () => {
  beforeEach(() => {
    mockGetOperationalReport.mockReset();
    mockGetOperationalReport.mockImplementation(async (input) =>
      buildReport(input)
    );
  });

  it("renders weekly and monthly operational summaries with accessible chart and tabular breakdown", async () => {
    const user = userEvent.setup();
    renderReports();

    // Weekly report loads with timezone, KPI counts, accessible chart, and table
    expect(
      await screen.findByText(/Week of Jun 15 – Jun 21, 2026/i)
    ).toBeInTheDocument();
    expect(screen.getByText("50% of total")).toBeInTheDocument();
    expect(
      screen.getByRole("img", {
        name: /Stacked bar chart showing daily weekly appointment counts by status/i,
      })
    ).toBeInTheDocument();
    expect(screen.getByText("Daily Breakdown Table")).toBeInTheDocument();
    expect(screen.getByText("Initial Consultation")).toBeInTheDocument();

    // Switch to Monthly report
    await user.click(screen.getByRole("button", { name: "Monthly" }));
    await waitFor(() => {
      expect(mockGetOperationalReport).toHaveBeenCalledWith(
        expect.objectContaining({ period_type: "monthly" })
      );
    });
    expect(
      await screen.findByText(/June 2026 \(2026-06-01 to 2026-06-30\)/i)
    ).toBeInTheDocument();
  });
});
