import { useQuery } from "@tanstack/react-query";
import {
  getDashboardSummary,
  getOperationalReport,
  type DashboardPeriod,
  type OperationalReportInput,
} from "@/services/reports";
import { isTauriRuntime } from "@/services/tauri";

export const REPORT_QUERY_KEYS = {
  all: ["reports"] as const,
  dashboard: (period: DashboardPeriod) =>
    ["reports", "dashboard", period] as const,
  operational: (input: OperationalReportInput) =>
    ["reports", "operational", input] as const,
};

export function useDashboardSummary(period: DashboardPeriod) {
  return useQuery({
    queryKey: REPORT_QUERY_KEYS.dashboard(period),
    queryFn: () => getDashboardSummary(period),
    enabled: isTauriRuntime(),
    placeholderData: (prev) => prev,
  });
}

export function useOperationalReport(input: OperationalReportInput) {
  return useQuery({
    queryKey: REPORT_QUERY_KEYS.operational(input),
    queryFn: () => getOperationalReport(input),
    enabled: isTauriRuntime(),
    placeholderData: (prev) => prev,
  });
}
