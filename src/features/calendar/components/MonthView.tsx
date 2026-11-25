import { useMemo } from "react";
import type { AppointmentDetail } from "@/services/appointments";
import {
  enumerateDates,
  formatDayHeader,
  getAppointmentsForDay,
  parseYmd,
} from "../utils/calendarDate";
import { AppointmentEventCard } from "./AppointmentEventCard";

const WEEKDAY_COLUMNS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] as const;
const MAX_VISIBLE_CHIPS = 3;

interface MonthViewProps {
  anchorDate: string;
  rangeStartYmd: string;
  rangeEndYmd: string;
  todayYmd: string;
  appointments: AppointmentDetail[];
  onSelectDay: (dayYmd: string) => void;
}

export function MonthView({
  anchorDate,
  rangeStartYmd,
  rangeEndYmd,
  todayYmd,
  appointments,
  onSelectDay,
}: MonthViewProps) {
  const { month: anchorMonth } = parseYmd(anchorDate);
  const days = useMemo(
    () => enumerateDates(rangeStartYmd, rangeEndYmd),
    [rangeStartYmd, rangeEndYmd]
  );

  return (
    <section
      aria-label="Month schedule"
      className="rounded-xl border border-border bg-card overflow-hidden"
    >
      <div className="grid grid-cols-7 border-b border-border bg-muted/30">
        {WEEKDAY_COLUMNS.map((label) => (
          <div
            key={label}
            className="px-2 py-2 text-center text-xs font-medium uppercase tracking-wide text-muted-foreground"
          >
            {label}
          </div>
        ))}
      </div>

      <div className="grid grid-cols-7 auto-rows-fr">
        {days.map((dayYmd) => {
          const { month } = parseYmd(dayYmd);
          const header = formatDayHeader(dayYmd);
          const inCurrentMonth = month === anchorMonth;
          const isToday = dayYmd === todayYmd;
          const dayAppts = getAppointmentsForDay(appointments, dayYmd);
          const visible = dayAppts.slice(0, MAX_VISIBLE_CHIPS);
          const overflowCount = dayAppts.length - visible.length;

          return (
            <div
              key={dayYmd}
              className={`min-h-28 border-b border-r border-border p-1.5 flex flex-col gap-1 min-w-0 ${
                inCurrentMonth ? "bg-card" : "bg-muted/25 text-muted-foreground"
              } ${isToday ? "ring-1 ring-inset ring-primary/50 bg-primary/5" : ""}`}
            >
              <div className="flex items-center justify-between gap-1">
                <button
                  type="button"
                  onClick={() => onSelectDay(dayYmd)}
                  aria-current={isToday ? "date" : undefined}
                  aria-label={`Open day view for ${header.weekdayLong}, ${header.monthLong} ${header.dayNum}`}
                  className={`inline-flex items-center justify-center rounded-md px-1.5 py-0.5 text-xs font-semibold tabular-nums transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring ${
                    isToday
                      ? "bg-primary text-primary-foreground hover:bg-primary/90"
                      : inCurrentMonth
                        ? "text-foreground"
                        : "text-muted-foreground"
                  }`}
                >
                  {header.dayNum}
                </button>

                {dayAppts.length > 0 && (
                  <span className="text-[10px] text-muted-foreground tabular-nums">
                    {dayAppts.length}
                  </span>
                )}
              </div>

              <div className="space-y-1 flex-1 min-w-0">
                {visible.map((appt) => (
                  <AppointmentEventCard
                    key={appt.id}
                    appointment={appt}
                    dayYmd={dayYmd}
                    variant="compact"
                  />
                ))}
                {overflowCount > 0 && (
                  <button
                    type="button"
                    onClick={() => onSelectDay(dayYmd)}
                    className="w-full rounded px-1.5 py-0.5 text-left text-[11px] font-medium text-primary hover:bg-accent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  >
                    +{overflowCount} more
                  </button>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
