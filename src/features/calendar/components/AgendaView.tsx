import { useMemo } from "react";
import { CalendarClock } from "lucide-react";
import type { AppointmentDetail } from "@/services/appointments";
import {
  enumerateDates,
  formatDayHeader,
  getAppointmentsForDay,
} from "../utils/calendarDate";
import { AppointmentEventCard } from "./AppointmentEventCard";

interface AgendaViewProps {
  rangeStartYmd: string;
  rangeEndYmd: string;
  todayYmd: string;
  appointments: AppointmentDetail[];
  onSelectDay: (dayYmd: string) => void;
}

export function AgendaView({
  rangeStartYmd,
  rangeEndYmd,
  todayYmd,
  appointments,
  onSelectDay,
}: AgendaViewProps) {
  const groupedDays = useMemo(() => {
    const allDays = enumerateDates(rangeStartYmd, rangeEndYmd);
    return allDays
      .map((dayYmd) => ({
        dayYmd,
        appointments: getAppointmentsForDay(appointments, dayYmd),
      }))
      .filter((entry) => entry.appointments.length > 0);
  }, [appointments, rangeStartYmd, rangeEndYmd]);

  if (groupedDays.length === 0) {
    return (
      <section
        aria-label="Agenda schedule"
        className="rounded-xl border border-border bg-card py-12 px-6 text-center"
      >
        <CalendarClock className="mx-auto size-8 text-muted-foreground/60 mb-2" />
        <p className="text-sm font-medium">
          No appointments in this date range
        </p>
        <p className="text-xs text-muted-foreground mt-1">
          Move to another period or schedule a new appointment.
        </p>
      </section>
    );
  }

  return (
    <section aria-label="Agenda schedule" className="space-y-4">
      {groupedDays.map(({ dayYmd, appointments: dayAppts }) => {
        const header = formatDayHeader(dayYmd);
        const isToday = dayYmd === todayYmd;

        return (
          <div
            key={dayYmd}
            className="rounded-xl border border-border bg-card overflow-hidden"
          >
            <div
              className={`flex items-center justify-between border-b border-border px-4 py-2.5 ${
                isToday ? "bg-primary/5" : "bg-muted/30"
              }`}
            >
              <button
                type="button"
                onClick={() => onSelectDay(dayYmd)}
                aria-current={isToday ? "date" : undefined}
                className="flex items-center gap-2 text-left hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring rounded"
              >
                <span className="text-sm font-semibold">
                  {header.weekdayLong}, {header.monthLong} {header.dayNum},{" "}
                  {header.year}
                </span>
                {isToday && (
                  <span className="rounded-full bg-primary/15 px-2 py-0.5 text-[11px] font-medium text-primary">
                    Today
                  </span>
                )}
              </button>

              <span className="text-xs text-muted-foreground">
                {dayAppts.length}{" "}
                {dayAppts.length === 1 ? "session" : "sessions"}
              </span>
            </div>

            <div className="p-3 space-y-2">
              {dayAppts.map((appt) => (
                <AppointmentEventCard
                  key={appt.id}
                  appointment={appt}
                  dayYmd={dayYmd}
                  variant="detailed"
                />
              ))}
            </div>
          </div>
        );
      })}
    </section>
  );
}
