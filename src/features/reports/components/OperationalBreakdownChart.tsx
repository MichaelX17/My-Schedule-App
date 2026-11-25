import { formatDayHeader } from "@/features/calendar/utils/calendarDate";
import type { DailyBreakdownPoint, ReportPeriodType } from "@/services/reports";

interface OperationalBreakdownChartProps {
  periodType: ReportPeriodType;
  points: DailyBreakdownPoint[];
}

export function OperationalBreakdownChart({
  periodType,
  points,
}: OperationalBreakdownChartProps) {
  const maxTotal = Math.max(1, ...points.map((p) => p.total));
  const hasAnyAppointments = points.some((p) => p.total > 0);

  return (
    <div className="space-y-4">
      <div className="rounded-xl border border-border bg-card p-4 space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h3 className="text-sm font-semibold">
            Daily Appointments by Status
          </h3>
          <div className="flex flex-wrap items-center gap-3 text-xs text-muted-foreground">
            <span className="inline-flex items-center gap-1.5">
              <span
                className="size-2.5 rounded-xs bg-emerald-600 dark:bg-emerald-400"
                aria-hidden="true"
              />
              Completed
            </span>
            <span className="inline-flex items-center gap-1.5">
              <span
                className="size-2.5 rounded-xs bg-primary"
                aria-hidden="true"
              />
              Scheduled
            </span>
            <span className="inline-flex items-center gap-1.5">
              <span
                className="size-2.5 rounded-xs bg-muted-foreground/50"
                aria-hidden="true"
              />
              Cancelled
            </span>
          </div>
        </div>

        {!hasAnyAppointments ? (
          <p className="py-8 text-center text-sm text-muted-foreground">
            No appointments recorded during this reporting period.
          </p>
        ) : (
          <div
            role="img"
            aria-label={`Stacked bar chart showing daily ${periodType} appointment counts by status`}
            className="overflow-x-auto pb-1"
          >
            <div
              className={`grid items-end gap-1.5 h-44 min-w-[320px] ${
                periodType === "weekly" ? "grid-cols-7" : ""
              }`}
              style={
                periodType === "monthly"
                  ? {
                      gridTemplateColumns: `repeat(${points.length}, minmax(16px, 1fr))`,
                    }
                  : undefined
              }
            >
              {points.map((point) => {
                const header = formatDayHeader(point.date_local);
                const completedPct = (point.completed / maxTotal) * 100;
                const scheduledPct = (point.scheduled / maxTotal) * 100;
                const cancelledPct = (point.cancelled / maxTotal) * 100;
                const label =
                  periodType === "weekly"
                    ? `${header.weekdayShort} ${header.dayNum}`
                    : String(header.dayNum);

                return (
                  <div
                    key={point.date_local}
                    className="flex flex-col items-center h-full justify-end gap-1"
                    title={`${point.date_local}: ${point.total} total (${point.completed} completed, ${point.scheduled} scheduled, ${point.cancelled} cancelled)`}
                  >
                    <span className="text-[10px] tabular-nums text-muted-foreground">
                      {point.total > 0 ? point.total : ""}
                    </span>
                    <div className="w-full max-w-8 flex-1 flex flex-col justify-end rounded-t bg-muted/30 overflow-hidden">
                      {point.cancelled > 0 && (
                        <div
                          className="w-full bg-muted-foreground/50"
                          style={{ height: `${cancelledPct}%` }}
                        />
                      )}
                      {point.scheduled > 0 && (
                        <div
                          className="w-full bg-primary"
                          style={{ height: `${scheduledPct}%` }}
                        />
                      )}
                      {point.completed > 0 && (
                        <div
                          className="w-full bg-emerald-600 dark:bg-emerald-400"
                          style={{ height: `${completedPct}%` }}
                        />
                      )}
                    </div>
                    <span className="text-[10px] tabular-nums text-muted-foreground truncate max-w-full">
                      {label}
                    </span>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>

      {/* Accessible tabular alternative */}
      <div className="rounded-xl border border-border bg-card overflow-hidden">
        <div className="border-b border-border bg-muted/30 px-4 py-2.5">
          <h3 className="text-sm font-semibold">Daily Breakdown Table</h3>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead className="border-b border-border bg-muted/20 text-xs text-muted-foreground">
              <tr>
                <th className="px-4 py-2 text-left font-medium">Date</th>
                <th className="px-4 py-2 text-right font-medium">Scheduled</th>
                <th className="px-4 py-2 text-right font-medium">Completed</th>
                <th className="px-4 py-2 text-right font-medium">Cancelled</th>
                <th className="px-4 py-2 text-right font-medium">Total</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {points.map((point) => {
                const header = formatDayHeader(point.date_local);
                return (
                  <tr
                    key={point.date_local}
                    className={
                      point.total === 0 ? "text-muted-foreground/70" : ""
                    }
                  >
                    <td className="px-4 py-2 tabular-nums">
                      {point.date_local} ({header.weekdayShort})
                    </td>
                    <td className="px-4 py-2 text-right tabular-nums">
                      {point.scheduled}
                    </td>
                    <td className="px-4 py-2 text-right tabular-nums">
                      {point.completed}
                    </td>
                    <td className="px-4 py-2 text-right tabular-nums">
                      {point.cancelled}
                    </td>
                    <td className="px-4 py-2 text-right font-medium tabular-nums">
                      {point.total}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
