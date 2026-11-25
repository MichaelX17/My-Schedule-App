import { useState } from "react";
import {
  AlertCircle,
  CheckCircle2,
  ChevronLeft,
  ChevronRight,
  Clock,
  Users,
  XCircle,
} from "lucide-react";
import { PageLayout } from "@/components/layout/PageLayout";
import { Button } from "@/components/ui/button";
import {
  addDays,
  addMonths,
  formatDayHeader,
} from "@/features/calendar/utils/calendarDate";
import { formatDuration } from "@/features/services/validation";
import { extractErrorMessage } from "@/services/business";
import type { ReportPeriodType } from "@/services/reports";
import { OperationalBreakdownChart } from "./components/OperationalBreakdownChart";
import { useOperationalReport } from "./hooks/useReports";

function formatReportRangeHeading(
  periodType: ReportPeriodType,
  startYmd: string,
  endYmd: string
): string {
  const s = formatDayHeader(startYmd);
  const e = formatDayHeader(endYmd);
  if (periodType === "monthly") {
    return `${s.monthLong} ${s.year} (${startYmd} to ${endYmd})`;
  }
  return `Week of ${s.monthShort} ${s.dayNum} – ${e.monthShort} ${e.dayNum}, ${e.year} (${startYmd} to ${endYmd})`;
}

export function ReportsPage() {
  const [periodType, setPeriodType] = useState<ReportPeriodType>("weekly");
  const [anchorDate, setAnchorDate] = useState<string | null>(null);

  const { data, isLoading, isFetching, isError, error } = useOperationalReport({
    period_type: periodType,
    anchor_date: anchorDate,
  });

  const activeAnchor = data?.anchor_date ?? anchorDate ?? "";

  const handleStep = (direction: -1 | 1) => {
    if (!activeAnchor) return;
    if (periodType === "weekly") {
      setAnchorDate(addDays(activeAnchor, direction * 7));
    } else {
      setAnchorDate(addMonths(activeAnchor, direction));
    }
  };

  const handleCurrentPeriod = () => {
    setAnchorDate(null);
  };

  return (
    <PageLayout
      maxWidth="6xl"
      header={
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-semibold tracking-tight">
              Operational Reports
            </h1>
            <p className="mt-1 text-sm text-muted-foreground">
              Weekly and monthly non-financial summaries calculated in business
              timezone ({data?.timezone ?? "..."}).
            </p>
          </div>

          <div className="flex flex-wrap items-center gap-2">
            <div
              role="group"
              aria-label="Report cadence"
              className="inline-flex rounded-md border border-border bg-background p-0.5"
            >
              <button
                type="button"
                aria-pressed={periodType === "weekly"}
                onClick={() => setPeriodType("weekly")}
                className={`rounded px-3 py-1 text-xs font-medium transition-colors ${
                  periodType === "weekly"
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:text-foreground"
                }`}
              >
                Weekly
              </button>
              <button
                type="button"
                aria-pressed={periodType === "monthly"}
                onClick={() => setPeriodType("monthly")}
                className={`rounded px-3 py-1 text-xs font-medium transition-colors ${
                  periodType === "monthly"
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:text-foreground"
                }`}
              >
                Monthly
              </button>
            </div>

            <div className="inline-flex items-center rounded-md border border-border bg-background p-0.5">
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="size-8"
                aria-label="Previous report period"
                onClick={() => handleStep(-1)}
              >
                <ChevronLeft className="size-4" />
              </Button>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="h-8 px-2.5 text-xs"
                onClick={handleCurrentPeriod}
              >
                Current
              </Button>
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="size-8"
                aria-label="Next report period"
                onClick={() => handleStep(1)}
              >
                <ChevronRight className="size-4" />
              </Button>
            </div>
          </div>
        </div>
      }
    >

      {isError ? (
        <div
          role="alert"
          className="flex items-start gap-2.5 rounded-xl border border-destructive/40 bg-card p-4 text-sm text-destructive"
        >
          <AlertCircle className="size-4 mt-0.5 shrink-0" />
          <div>
            <p className="font-medium">Unable to load operational report</p>
            <p className="text-xs text-muted-foreground mt-0.5">
              {extractErrorMessage(error)}
            </p>
          </div>
        </div>
      ) : isLoading && !data ? (
        <div className="rounded-xl border border-border bg-card py-16 text-center text-sm text-muted-foreground">
          Loading operational report...
        </div>
      ) : data ? (
        <div className={`space-y-6 ${isFetching ? "opacity-80" : ""}`}>
          <div className="rounded-xl border border-border bg-muted/30 px-4 py-3 flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <h2 className="text-sm font-semibold" aria-live="polite">
              {formatReportRangeHeading(
                data.period_type,
                data.period_start_local,
                data.period_end_local
              )}
            </h2>
            <span className="text-xs text-muted-foreground">
              Timezone: <span className="font-mono">{data.timezone}</span>
            </span>
          </div>

          {/* Status & Operational KPI Grid */}
          <div className="grid grid-cols-2 lg:grid-cols-6 gap-3">
            <div className="rounded-xl border border-border bg-card p-4">
              <p className="text-xs text-muted-foreground">Total</p>
              <p className="text-2xl font-semibold tabular-nums mt-1">
                {data.counts.total}
              </p>
              <p className="text-[11px] text-muted-foreground mt-0.5">
                All statuses
              </p>
            </div>

            <div className="rounded-xl border border-border bg-card p-4">
              <div className="flex items-center gap-1 text-xs text-primary">
                <Clock className="size-3.5" aria-hidden="true" />
                <span>Scheduled</span>
              </div>
              <p className="text-2xl font-semibold tabular-nums mt-1">
                {data.counts.scheduled}
              </p>
              <p className="text-[11px] text-muted-foreground mt-0.5">
                Booked / active
              </p>
            </div>

            <div className="rounded-xl border border-border bg-card p-4">
              <div className="flex items-center gap-1 text-xs text-emerald-700 dark:text-emerald-300">
                <CheckCircle2 className="size-3.5" aria-hidden="true" />
                <span>Completed</span>
              </div>
              <p className="text-2xl font-semibold tabular-nums mt-1">
                {data.counts.completed}
              </p>
              <p className="text-[11px] text-muted-foreground mt-0.5">
                {data.completion_rate_percent !== null
                  ? `${data.completion_rate_percent}% of total`
                  : "No sessions"}
              </p>
            </div>

            <div className="rounded-xl border border-border bg-card p-4">
              <div className="flex items-center gap-1 text-xs text-muted-foreground">
                <XCircle className="size-3.5" aria-hidden="true" />
                <span>Cancelled</span>
              </div>
              <p className="text-2xl font-semibold tabular-nums mt-1">
                {data.counts.cancelled}
              </p>
              <p className="text-[11px] text-muted-foreground mt-0.5">
                Excluded from active time
              </p>
            </div>

            <div className="rounded-xl border border-border bg-card p-4">
              <p className="text-xs text-muted-foreground">Active Time</p>
              <p className="text-2xl font-semibold tabular-nums mt-1">
                {data.total_active_minutes > 0
                  ? formatDuration(data.total_active_minutes)
                  : "0m"}
              </p>
              <p className="text-[11px] text-muted-foreground mt-0.5">
                Scheduled + completed
              </p>
            </div>

            <div className="rounded-xl border border-border bg-card p-4">
              <div className="flex items-center gap-1 text-xs text-muted-foreground">
                <Users className="size-3.5" aria-hidden="true" />
                <span>Clients Served</span>
              </div>
              <p className="text-2xl font-semibold tabular-nums mt-1">
                {data.unique_clients_served}
              </p>
              <p className="text-[11px] text-muted-foreground mt-0.5">
                Unique non-cancelled
              </p>
            </div>
          </div>

          {/* Daily Chart + Accessible Table */}
          <OperationalBreakdownChart
            periodType={data.period_type}
            points={data.daily_breakdown}
          />

          {/* Top Services Table */}
          <section
            aria-label="Service usage breakdown"
            className="rounded-xl border border-border bg-card overflow-hidden"
          >
            <div className="border-b border-border bg-muted/30 px-4 py-2.5 flex items-center justify-between">
              <h3 className="text-sm font-semibold">
                Services Breakdown (Non-Cancelled Sessions)
              </h3>
              <span className="text-xs text-muted-foreground">
                Based on historical service snapshots
              </span>
            </div>

            {data.top_services.length === 0 ? (
              <p className="py-8 text-center text-sm text-muted-foreground">
                No non-cancelled services recorded in this period.
              </p>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead className="border-b border-border bg-muted/20 text-xs text-muted-foreground">
                    <tr>
                      <th className="px-4 py-2 text-left font-medium">
                        Service
                      </th>
                      <th className="px-4 py-2 text-right font-medium">
                        Sessions
                      </th>
                      <th className="px-4 py-2 text-right font-medium">
                        Total Duration
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-border">
                    {data.top_services.map((svc) => (
                      <tr key={svc.service_name}>
                        <td className="px-4 py-2 font-medium">
                          {svc.service_name}
                        </td>
                        <td className="px-4 py-2 text-right tabular-nums">
                          {svc.appointment_count}
                        </td>
                        <td className="px-4 py-2 text-right tabular-nums text-muted-foreground">
                          {formatDuration(svc.total_minutes)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </section>
        </div>
      ) : null}
    </PageLayout>
  );
}
