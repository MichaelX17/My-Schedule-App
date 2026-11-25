import {
  Calendar as CalendarIcon,
  ChevronLeft,
  ChevronRight,
  Plus,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import type { AppointmentStatus } from "@/services/appointments";
import type { CalendarView } from "@/services/calendar";

const VIEW_OPTIONS: { value: CalendarView; label: string }[] = [
  { value: "day", label: "Day" },
  { value: "week", label: "Week" },
  { value: "month", label: "Month" },
  { value: "agenda", label: "Agenda" },
];

const STATUS_OPTIONS: { value: AppointmentStatus | "all"; label: string }[] = [
  { value: "all", label: "All statuses" },
  { value: "scheduled", label: "Scheduled" },
  { value: "completed", label: "Completed" },
  { value: "cancelled", label: "Cancelled" },
];

interface CalendarToolbarProps {
  view: CalendarView;
  anchorDate: string;
  rangeHeading: string;
  timezone: string;
  statusFilter: AppointmentStatus | "all";
  onViewChange: (view: CalendarView) => void;
  onPrevious: () => void;
  onNext: () => void;
  onToday: () => void;
  onDateChange: (ymd: string) => void;
  onStatusFilterChange: (status: AppointmentStatus | "all") => void;
  onNewAppointment: () => void;
}

export function CalendarToolbar({
  view,
  anchorDate,
  rangeHeading,
  timezone,
  statusFilter,
  onViewChange,
  onPrevious,
  onNext,
  onToday,
  onDateChange,
  onStatusFilterChange,
  onNewAppointment,
}: CalendarToolbarProps) {
  return (
    <div className="space-y-3">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Calendar</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Day, week, month, and agenda views in business timezone ({timezone}).
          </p>
        </div>

        <Button onClick={onNewAppointment}>
          <Plus className="size-4" />
          New Appointment
        </Button>
      </div>

      <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-3 rounded-xl border border-border bg-card p-3">
        <div className="flex flex-wrap items-center gap-2">
          <div className="inline-flex items-center rounded-md border border-border bg-background p-0.5">
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="size-8"
              onClick={onPrevious}
              aria-label="Previous period"
            >
              <ChevronLeft className="size-4" />
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="h-8 px-2.5 text-xs font-medium"
              onClick={onToday}
            >
              Today
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="size-8"
              onClick={onNext}
              aria-label="Next period"
            >
              <ChevronRight className="size-4" />
            </Button>
          </div>

          <h2
            className="text-base font-semibold tracking-tight px-1"
            aria-live="polite"
          >
            {rangeHeading}
          </h2>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <label className="relative inline-flex items-center">
            <CalendarIcon
              className="pointer-events-none size-3.5 text-muted-foreground absolute left-2.5"
              aria-hidden="true"
            />
            <input
              type="date"
              aria-label="Select date"
              value={anchorDate}
              onChange={(e) => {
                if (e.target.value) {
                  onDateChange(e.target.value);
                }
              }}
              className="h-8 rounded-md border border-input bg-background pl-8 pr-2 text-xs text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            />
          </label>

          <select
            aria-label="Filter by status"
            value={statusFilter}
            onChange={(e) =>
              onStatusFilterChange(e.target.value as AppointmentStatus | "all")
            }
            className="h-8 rounded-md border border-input bg-background px-2.5 text-xs text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            {STATUS_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>

          <div
            role="group"
            aria-label="Calendar view"
            className="inline-flex items-center rounded-md border border-border bg-background p-0.5"
          >
            {VIEW_OPTIONS.map((opt) => {
              const active = view === opt.value;
              return (
                <button
                  key={opt.value}
                  type="button"
                  aria-pressed={active}
                  onClick={() => onViewChange(opt.value)}
                  className={`rounded px-2.5 py-1 text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring ${
                    active
                      ? "bg-primary text-primary-foreground shadow-xs"
                      : "text-muted-foreground hover:text-foreground hover:bg-accent/60"
                  }`}
                >
                  {opt.label}
                </button>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}
