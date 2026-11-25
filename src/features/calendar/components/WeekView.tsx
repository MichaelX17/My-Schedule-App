import { useMemo } from "react";
import type { AppointmentDetail } from "@/services/appointments";
import {
  enumerateDates,
  formatDayHeader,
  getAppointmentsForDay,
} from "../utils/calendarDate";
import { AppointmentEventCard } from "./AppointmentEventCard";

interface WeekViewProps {
  rangeStartYmd: string;
  rangeEndYmd: string;
  todayYmd: string;
  appointments: AppointmentDetail[];
  onSelectDay: (dayYmd: string) => void;
}

export function WeekView({
  rangeStartYmd,
  rangeEndYmd,
  todayYmd,
  appointments,
  onSelectDay,
}: WeekViewProps) {
  const days = useMemo(
    () => enumerateDates(rangeStartYmd, rangeEndYmd),
    [rangeStartYmd, rangeEndYmd]
  );

  return (
    <section
      aria-label="Week schedule"
      className="rounded-xl border border-border bg-card overflow-hidden"
    >
      <div className="grid grid-cols-1 md:grid-cols-7 divide-y md:divide-y-0 md:divide-x divide-border">
        {days.map((dayYmd) => {
          const header = formatDayHeader(dayYmd);
          const isToday = dayYmd === todayYmd;
          const dayAppts = getAppointmentsForDay(appointments, dayYmd);

          return (
            <div
              key={dayYmd}
              className="flex flex-col min-w-0 md:min-h-[420px]"
            >
              <button
                type="button"
                onClick={() => onSelectDay(dayYmd)}
                aria-current={isToday ? "date" : undefined}
                aria-label={`Open day view for ${header.weekdayLong}, ${header.monthLong} ${header.dayNum}`}
                className={`flex items-center md:flex-col md:items-start justify-between gap-1 border-b border-border px-3 py-2.5 text-left transition-colors hover:bg-accent/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring ${
                  isToday ? "bg-primary/5" : "bg-muted/25"
                }`}
              >
                <div className="flex items-center gap-1.5">
                  <span className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                    {header.weekdayShort}
                  </span>
                  {isToday && (
                    <span className="rounded-full bg-primary/15 px-1.5 py-0.5 text-[10px] font-medium text-primary">
                      Today
                    </span>
                  )}
                </div>
                <div className="flex items-baseline gap-1">
                  <span
                    className={`text-base font-semibold tabular-nums ${
                      isToday ? "text-primary" : "text-foreground"
                    }`}
                  >
                    {header.dayNum}
                  </span>
                  <span className="text-[11px] text-muted-foreground">
                    {header.monthShort}
                  </span>
                </div>
              </button>

              <div className="flex-1 p-2 space-y-2">
                {dayAppts.length === 0 ? (
                  <p className="py-3 text-center text-xs text-muted-foreground/70">
                    No sessions
                  </p>
                ) : (
                  dayAppts.map((appt) => (
                    <AppointmentEventCard
                      key={appt.id}
                      appointment={appt}
                      dayYmd={dayYmd}
                      variant="standard"
                    />
                  ))
                )}
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
