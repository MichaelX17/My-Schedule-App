import type { AppointmentDetail } from "@/services/appointments";
import { invokeCommand } from "@/services/tauri";

export type DashboardPeriod = "today" | "this_week" | "this_month";
export type ReportPeriodType = "weekly" | "monthly";

export interface StatusCounts {
  total: number;
  scheduled: number;
  completed: number;
  cancelled: number;
}

export interface ServiceUsageItem {
  service_name: string;
  appointment_count: number;
  total_minutes: number;
}

export interface DailyBreakdownPoint {
  date_local: string;
  scheduled: number;
  completed: number;
  cancelled: number;
  total: number;
}

export interface DashboardSummary {
  business_name: string;
  timezone: string;
  today_local: string;
  period: DashboardPeriod;
  period_start_local: string;
  period_end_local: string;
  today_appointments: AppointmentDetail[];
  upcoming_appointments: AppointmentDetail[];
  today_counts: StatusCounts;
  period_counts: StatusCounts;
  upcoming_scheduled_count: number;
  needs_outcome_count: number;
  active_clients_count: number;
  active_session_plans_count: number;
  unread_notifications_count: number;
}

export interface OperationalReportInput {
  period_type: ReportPeriodType;
  anchor_date?: string | null;
}

export interface OperationalReport {
  period_type: ReportPeriodType;
  timezone: string;
  anchor_date: string;
  period_start_local: string;
  period_end_local: string;
  period_start_utc: string;
  period_end_utc_exclusive: string;
  counts: StatusCounts;
  completion_rate_percent: number | null;
  total_active_minutes: number;
  unique_clients_served: number;
  daily_breakdown: DailyBreakdownPoint[];
  top_services: ServiceUsageItem[];
}

export async function getDashboardSummary(
  period: DashboardPeriod = "this_week"
): Promise<DashboardSummary> {
  return invokeCommand<DashboardSummary>("get_dashboard_summary", { period });
}

export async function getOperationalReport(
  input: OperationalReportInput
): Promise<OperationalReport> {
  return invokeCommand<OperationalReport>("get_operational_report", { input });
}
