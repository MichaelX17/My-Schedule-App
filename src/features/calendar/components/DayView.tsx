import { useMemo } from "react";
import { CalendarClock } from "lucide-react";
import type { AppointmentDetail } from "@/services/appointments";
import {
  extractLocalDate,
  extractLocalMinutes,
  getAppointmentsForDay,
} from "../utils/calendarDate";
import { AppointmentEventCard } from "./AppointmentEventCard";

interface DayViewProps {
  dayYmd: string;
  todayYmd: string;
  appointments: AppointmentDetail[];
}

export function DayView({ dayYmd, todayYmd, appointments }: DayViewProps) {
  const dayAppointments = useMemo(
    () => getAppointmentsForDay(appointments, dayYmd),
    [appointments, dayYmd]
  );

  // Determine hour range: default 07:00..21:00, expanding if any appointment is earlier/later.
  const { startHour, endHour, byHour } = useMemo(() => {
    let minH = 7;
    let maxH = 21;
    const grouped = new Map<number, AppointmentDetail[]>();

    for (const appt of dayAppointments) {
      const apptStartDate = extractLocalDate(appt.start_local);
      const startMinutes =
        apptStartDate < dayYmd ? 0 : extractLocalMinutes(appt.start_local);
      const hour = Math.floor(startMinutes / 60);
      if (hour < minH) minH = hour;
      if (hour > maxH) maxH = hour;
      const list = grouped.get(hour) ?? [];
      list.push(appt);
      grouped.set(hour, list);
    }

    return { startHour: minH, endHour: maxH, byHour: grouped };
  }, [dayAppointments, dayYmd]);

  const hours = useMemo(() => {
    const result: number[] = [];
    for (let h = startHour; h <= endHour; h += 1) {
      result.push(h);
    }
    return result;
  }, [startHour, endHour]);

  const isToday = dayYmd === todayYmd;

  return (
    <section
      aria-label="Day schedule"
      className="rounded-xl border border-border bg-card overflow-hidden"
    >
      <div className="flex items-center justify-between border-b border-border bg-muted/30 px-4 py-2.5">
        <div className="flex items-center gap-2">
          <span className="text-sm font-semibold">{dayYmd}</span>
          {isToday && (
            <span className="rounded-full bg-primary/15 px-2 py-0.5 text-[11px] font-medium text-primary">
              Today
            </span>
          )}
        </div>
        <span className="text-xs text-muted-foreground">
          {dayAppointments.length}{" "}
          {dayAppointments.length === 1 ? "appointment" : "appointments"}
        </span>
      </div>

      {dayAppointments.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-12 px-6 text-center">
          <CalendarClock className="size-8 text-muted-foreground/60 mb-2" />
          <p className="text-sm font-medium">No appointments on this day</p>
          <p className="text-xs text-muted-foreground mt-1">
            Use &ldquo;New Appointment&rdquo; above or navigate to another date.
          </p>
        </div>
      ) : (
        <div className="divide-y divide-border">
          {hours.map((hour) => {
            const slotItems = byHour.get(hour) ?? [];
            const hourLabel = `${String(hour).padStart(2, "0")}:00`;
            return (
              <div
                key={hour}
                className="grid grid-cols-[64px_1fr] min-h-12"
              >
                <div className="border-r border-border px-3 py-2 text-right text-xs font-medium tabular-nums text-muted-foreground bg-muted/15">
                  {hourLabel}
                </div>
                <div className="p-2">
                  {slotItems.length > 0 && (
                    <div
                      className={`grid gap-2 ${
                        slotItems.length > 1
                          ? "grid-cols-1 md:grid-cols-2"
                          : "grid-cols-1"
                      }`}
                    >
                      {slotItems.map((appt) => (
                        <AppointmentEventCard
                          key={appt.id}
                          appointment={appt}
                          dayYmd={dayYmd}
                          variant="detailed"
                        />
                      ))}
                    </div>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </section>
  );
}
