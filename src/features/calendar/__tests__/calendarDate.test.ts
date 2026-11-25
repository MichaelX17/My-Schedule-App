import { describe, expect, it } from "vitest";
import type { AppointmentDetail } from "@/services/appointments";
import {
  addDays,
  addMonths,
  enumerateDates,
  formatAppointmentDuration,
  formatAppointmentTimeForDay,
  formatRangeHeading,
  getAppointmentsForDay,
  isValidYmd,
  stepAnchorDate,
} from "../utils/calendarDate";

function makeAppt(
  id: string,
  start_local: string,
  end_local: string
): AppointmentDetail {
  return {
    id,
    client_id: "c1",
    client_name: "Test Client",
    client_is_archived: false,
    session_plan_id: null,
    session_plan_title: null,
    start_utc: "2026-06-15T10:00:00.000Z",
    end_utc: "2026-06-15T11:00:00.000Z",
    start_local,
    end_local,
    timezone: "Europe/Madrid",
    status: "scheduled",
    notes: null,
    services: [
      {
        id: "as1",
        appointment_id: id,
        service_id: "s1",
        service_name_snapshot: "Consultation",
        service_description_snapshot: null,
        duration_minutes_snapshot: 60,
        sort_order: 0,
      },
    ],
    created_at: "2026-06-01T00:00:00.000Z",
    updated_at: "2026-06-01T00:00:00.000Z",
  };
}

describe("calendarDate utilities", () => {
  it("validates YYYY-MM-DD calendar dates strictly", () => {
    expect(isValidYmd("2026-06-15")).toBe(true);
    expect(isValidYmd("2024-02-29")).toBe(true);
    expect(isValidYmd("2026-02-29")).toBe(false);
    expect(isValidYmd("2026-13-01")).toBe(false);
    expect(isValidYmd("not-a-date")).toBe(false);
    expect(isValidYmd(null)).toBe(false);
  });

  it("steps anchor dates cleanly across day, week, month, and agenda views", () => {
    expect(stepAnchorDate("day", "2026-06-15", 1)).toBe("2026-06-16");
    expect(stepAnchorDate("day", "2026-06-15", -1)).toBe("2026-06-14");
    expect(stepAnchorDate("week", "2026-06-15", 1)).toBe("2026-06-22");
    expect(stepAnchorDate("week", "2026-06-15", -1)).toBe("2026-06-08");
    // Month clamping: Jan 31 + 1 month -> Feb 28 (non-leap year 2026)
    expect(addMonths("2026-01-31", 1)).toBe("2026-02-28");
    expect(stepAnchorDate("month", "2026-12-15", 1)).toBe("2027-01-15");
    expect(stepAnchorDate("agenda", "2026-06-15", 1)).toBe("2026-06-29");
    expect(addDays("2026-06-30", 1)).toBe("2026-07-01");
  });

  it("enumerates inclusive date ranges and formats headings", () => {
    const week = enumerateDates("2026-06-15", "2026-06-21");
    expect(week).toHaveLength(7);
    expect(week[0]).toBe("2026-06-15");
    expect(week[6]).toBe("2026-06-21");

    expect(
      formatRangeHeading("day", "2026-06-15", "2026-06-15", "2026-06-15")
    ).toBe("Monday, June 15, 2026");
    expect(
      formatRangeHeading("month", "2026-06-15", "2026-06-01", "2026-07-05")
    ).toBe("June 2026");
    expect(
      formatRangeHeading("week", "2026-06-15", "2026-06-15", "2026-06-21")
    ).toBe("Jun 15 – 21, 2026");
    expect(
      formatRangeHeading("agenda", "2026-06-25", "2026-06-25", "2026-07-08")
    ).toBe("Agenda: Jun 25 – Jul 8, 2026");
  });

  it("assigns boundary and midnight-crossing appointments to the exact right days", () => {
    const endsAtMidnight = makeAppt("a1", "2026-06-14T23:00", "2026-06-15T00:00");
    const startsAtMidnight = makeAppt("a2", "2026-06-15T00:00", "2026-06-15T01:00");
    const crossesMidnight = makeAppt("a3", "2026-06-15T23:30", "2026-06-16T00:30");
    const all = [endsAtMidnight, startsAtMidnight, crossesMidnight];

    expect(getAppointmentsForDay(all, "2026-06-14").map((a) => a.id)).toEqual([
      "a1",
    ]);
    expect(getAppointmentsForDay(all, "2026-06-15").map((a) => a.id)).toEqual([
      "a2",
      "a3",
    ]);
    expect(getAppointmentsForDay(all, "2026-06-16").map((a) => a.id)).toEqual([
      "a3",
    ]);

    expect(formatAppointmentTimeForDay(startsAtMidnight, "2026-06-15")).toBe(
      "00:00–01:00"
    );
    expect(formatAppointmentTimeForDay(crossesMidnight, "2026-06-15")).toBe(
      "23:30–24:00 (cont.)"
    );
    expect(formatAppointmentTimeForDay(crossesMidnight, "2026-06-16")).toBe(
      "00:00–00:30 (cont.)"
    );
  });

  it("formats appointment durations concisely", () => {
    const oneHour = makeAppt("d1", "2026-06-15T10:00", "2026-06-15T11:00");
    expect(formatAppointmentDuration(oneHour)).toBe("1h");

    const fortyFiveMins = {
      ...oneHour,
      start_utc: "2026-06-15T10:00:00.000Z",
      end_utc: "2026-06-15T10:45:00.000Z",
    };
    expect(formatAppointmentDuration(fortyFiveMins)).toBe("45m");

    const ninetyMins = {
      ...oneHour,
      start_utc: "2026-06-15T10:00:00.000Z",
      end_utc: "2026-06-15T11:30:00.000Z",
    };
    expect(formatAppointmentDuration(ninetyMins)).toBe("1h 30m");
  });
});
