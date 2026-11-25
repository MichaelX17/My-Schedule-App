import type { AppointmentDetail } from "@/services/appointments";
import type { CalendarView } from "@/services/calendar";

const WEEKDAY_SHORT = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"] as const;
const WEEKDAY_LONG = [
  "Sunday",
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday",
] as const;
const MONTH_SHORT = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
] as const;
const MONTH_LONG = [
  "January",
  "February",
  "March",
  "April",
  "May",
  "June",
  "July",
  "August",
  "September",
  "October",
  "November",
  "December",
] as const;

export const AGENDA_WINDOW_DAYS = 14;

/** Parses a YYYY-MM-DD string into numeric components without local-TZ drift. */
export function parseYmd(ymd: string): { year: number; month: number; day: number } {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(ymd.trim());
  if (!match) {
    throw new Error(`Invalid YYYY-MM-DD date: ${ymd}`);
  }
  return {
    year: Number(match[1]),
    month: Number(match[2]),
    day: Number(match[3]),
  };
}

/** Formats numeric calendar parts into YYYY-MM-DD. */
export function formatYmd(year: number, month: number, day: number): string {
  return `${String(year).padStart(4, "0")}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
}

/** Checks whether a string is a valid YYYY-MM-DD calendar date. */
export function isValidYmd(value: string | null | undefined): value is string {
  if (!value) return false;
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value.trim());
  if (!match) return false;
  const y = Number(match[1]);
  const m = Number(match[2]);
  const d = Number(match[3]);
  const dt = new Date(Date.UTC(y, m - 1, d));
  return (
    dt.getUTCFullYear() === y &&
    dt.getUTCMonth() === m - 1 &&
    dt.getUTCDate() === d
  );
}

/** Adds `days` (positive or negative) to a YYYY-MM-DD date using pure UTC calendar math. */
export function addDays(ymd: string, days: number): string {
  const { year, month, day } = parseYmd(ymd);
  const dt = new Date(Date.UTC(year, month - 1, day + days));
  return formatYmd(dt.getUTCFullYear(), dt.getUTCMonth() + 1, dt.getUTCDate());
}

/** Shifts a YYYY-MM-DD date by `months`, clamping the day to the target month's length. */
export function addMonths(ymd: string, months: number): string {
  const { year, month, day } = parseYmd(ymd);
  const firstOfTarget = new Date(Date.UTC(year, month - 1 + months, 1));
  const targetYear = firstOfTarget.getUTCFullYear();
  const targetMonth = firstOfTarget.getUTCMonth() + 1;
  const daysInTargetMonth = new Date(
    Date.UTC(targetYear, targetMonth, 0)
  ).getUTCDate();
  return formatYmd(targetYear, targetMonth, Math.min(day, daysInTargetMonth));
}

/** Steps the anchor date forward (+1) or backward (-1) for the given calendar view. */
export function stepAnchorDate(
  view: CalendarView,
  anchorYmd: string,
  direction: -1 | 1
): string {
  switch (view) {
    case "day":
      return addDays(anchorYmd, direction);
    case "week":
      return addDays(anchorYmd, direction * 7);
    case "month":
      return addMonths(anchorYmd, direction);
    case "agenda":
      return addDays(anchorYmd, direction * AGENDA_WINDOW_DAYS);
  }
}

/** Enumerates every YYYY-MM-DD date from `startYmd` through `endYmd` (inclusive). */
export function enumerateDates(startYmd: string, endYmd: string): string[] {
  const dates: string[] = [];
  let cursor = startYmd;
  // Safety guard against accidental inverted or extreme ranges.
  let guard = 0;
  while (cursor <= endYmd && guard < 90) {
    dates.push(cursor);
    cursor = addDays(cursor, 1);
    guard += 1;
  }
  return dates;
}

/** Formats weekday/day/month metadata for a YYYY-MM-DD string using UTC calendar math. */
export function formatDayHeader(ymd: string): {
  weekdayShort: string;
  weekdayLong: string;
  monthShort: string;
  monthLong: string;
  dayNum: number;
  year: number;
} {
  const { year, month, day } = parseYmd(ymd);
  const dt = new Date(Date.UTC(year, month - 1, day));
  const dow = dt.getUTCDay();
  return {
    weekdayShort: WEEKDAY_SHORT[dow] ?? "",
    weekdayLong: WEEKDAY_LONG[dow] ?? "",
    monthShort: MONTH_SHORT[month - 1] ?? "",
    monthLong: MONTH_LONG[month - 1] ?? "",
    dayNum: day,
    year,
  };
}

/** Formats the clear human-readable heading for the active calendar range. */
export function formatRangeHeading(
  view: CalendarView,
  anchorYmd: string,
  rangeStartYmd: string,
  rangeEndYmd: string
): string {
  if (view === "day") {
    const d = formatDayHeader(anchorYmd);
    return `${d.weekdayLong}, ${d.monthLong} ${d.dayNum}, ${d.year}`;
  }
  if (view === "month") {
    const d = formatDayHeader(anchorYmd);
    return `${d.monthLong} ${d.year}`;
  }
  const s = formatDayHeader(rangeStartYmd);
  const e = formatDayHeader(rangeEndYmd);
  const prefix = view === "agenda" ? "Agenda: " : "";
  if (s.year === e.year && s.monthShort === e.monthShort) {
    return `${prefix}${s.monthShort} ${s.dayNum} – ${e.dayNum}, ${s.year}`;
  }
  if (s.year === e.year) {
    return `${prefix}${s.monthShort} ${s.dayNum} – ${e.monthShort} ${e.dayNum}, ${s.year}`;
  }
  return `${prefix}${s.monthShort} ${s.dayNum}, ${s.year} – ${e.monthShort} ${e.dayNum}, ${e.year}`;
}

/** Extracts `YYYY-MM-DD` from a backend-formatted business-local timestamp `YYYY-MM-DDTHH:MM`. */
export function extractLocalDate(localDateTime: string): string {
  return localDateTime.slice(0, 10);
}

/** Extracts `HH:MM` from a backend-formatted business-local timestamp `YYYY-MM-DDTHH:MM`. */
export function extractLocalTime(localDateTime: string): string {
  return localDateTime.slice(11, 16);
}

/** Converts `YYYY-MM-DDTHH:MM` time-of-day to minutes since 00:00 (0..1440). */
export function extractLocalMinutes(localDateTime: string): number {
  const time = extractLocalTime(localDateTime);
  const parts = time.split(":");
  const h = Number(parts[0]);
  const m = Number(parts[1]);
  if (Number.isNaN(h) || Number.isNaN(m)) return 0;
  return h * 60 + m;
}

/**
 * Returns appointments overlapping a specific business-local date `dayYmd`.
 * Handles midnight-crossing appointments accurately:
 * - An appointment ending at `dayYmdT00:00` belongs to the previous day, not `dayYmd`.
 * - An appointment starting before `dayYmd` and ending after `dayYmdT00:00` appears on `dayYmd`.
 */
export function getAppointmentsForDay(
  appointments: AppointmentDetail[],
  dayYmd: string
): AppointmentDetail[] {
  return appointments.filter((appt) => {
    const startDate = extractLocalDate(appt.start_local);
    const endDate = extractLocalDate(appt.end_local);
    const endTime = extractLocalTime(appt.end_local);

    if (startDate === dayYmd) {
      return true;
    }
    if (startDate < dayYmd) {
      return endDate > dayYmd || (endDate === dayYmd && endTime > "00:00");
    }
    return false;
  });
}

/** Formats the display time label of an appointment within a specific local day. */
export function formatAppointmentTimeForDay(
  appt: AppointmentDetail,
  dayYmd: string
): string {
  const startDate = extractLocalDate(appt.start_local);
  const endDate = extractLocalDate(appt.end_local);
  const startTime = extractLocalTime(appt.start_local);
  const endTime = extractLocalTime(appt.end_local);

  if (startDate === dayYmd && endDate === dayYmd) {
    return `${startTime}–${endTime}`;
  }
  if (startDate === dayYmd && endDate > dayYmd) {
    return `${startTime}–24:00 (cont.)`;
  }
  if (startDate < dayYmd && endDate === dayYmd) {
    return `00:00–${endTime} (cont.)`;
  }
  return "All day (cont.)";
}

/** Formats total appointment duration concisely (e.g. "45m", "1h", "1h 30m"). */
export function formatAppointmentDuration(appt: AppointmentDetail): string {
  const startMs = Date.parse(appt.start_utc);
  const endMs = Date.parse(appt.end_utc);
  let totalMinutes = 0;

  if (!Number.isNaN(startMs) && !Number.isNaN(endMs) && endMs > startMs) {
    totalMinutes = Math.round((endMs - startMs) / 60_000);
  } else {
    const startMins = extractLocalMinutes(appt.start_local);
    const endMins = extractLocalMinutes(appt.end_local);
    if (endMins > startMins) {
      totalMinutes = endMins - startMins;
    }
  }

  if (totalMinutes <= 0) return "";
  const hours = Math.floor(totalMinutes / 60);
  const mins = totalMinutes % 60;
  if (hours === 0) return `${mins}m`;
  if (mins === 0) return `${hours}h`;
  return `${hours}h ${mins}m`;
}
