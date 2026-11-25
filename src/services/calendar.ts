import type {
  AppointmentDetail,
  AppointmentStatus,
} from "@/services/appointments";
import { invokeCommand } from "@/services/tauri";

export type CalendarView = "day" | "week" | "month" | "agenda";

export interface CalendarRangeInput {
  view: CalendarView;
  anchor_date?: string | null;
  status?: AppointmentStatus | null;
}

export interface CalendarRangeResponse {
  view: CalendarView;
  timezone: string;
  today_local: string;
  anchor_date: string;
  range_start_local: string;
  range_end_local: string;
  range_start_utc: string;
  range_end_utc_exclusive: string;
  appointments: AppointmentDetail[];
}

export async function getCalendarRange(
  input: CalendarRangeInput
): Promise<CalendarRangeResponse> {
  return invokeCommand<CalendarRangeResponse>("get_calendar_range", { input });
}
