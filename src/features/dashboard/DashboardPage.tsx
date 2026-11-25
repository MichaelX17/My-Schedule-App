import { useState } from "react";
import { Link } from "react-router-dom";
import {
  AlertCircle,
  BarChart3,
  CalendarClock,
  CalendarDays,
  CheckCircle2,
  Clock,
  FolderKanban,
  Plus,
  Users,
  XCircle,
} from "lucide-react";
import { PageLayout } from "@/components/layout/PageLayout";
import { Button } from "@/components/ui/button";
import { AppointmentFormDialog } from "@/features/appointments/components/AppointmentFormDialog";
import {
  useCreateAppointment,
  useSessionPlansList,
} from "@/features/appointments/hooks/useAppointments";
import { useClientsList } from "@/features/clients/hooks/useClients";
import { useDashboardSummary } from "@/features/reports/hooks/useReports";
import { useServicesList } from "@/features/services/hooks/useServices";
import type { AppointmentDetail } from "@/services/appointments";
import { extractErrorMessage } from "@/services/business";
import type { DashboardPeriod } from "@/services/reports";

const PERIOD_OPTIONS: { value: DashboardPeriod; label: string }[] = [
  { value: "today", label: "Today" },
  { value: "this_week", label: "This Week" },
  { value: "this_month", label: "This Month" },
];

function StatusPill({ status }: { status: AppointmentDetail["status"] }) {
  if (status === "completed") {
    return (
      <span className="inline-flex items-center gap-1 rounded-full bg-emerald-500/15 px-2 py-0.5 text-[11px] font-medium text-emerald-700 dark:text-emerald-300">
        <CheckCircle2 className="size-3" aria-hidden="true" />
        Completed
      </span>
    );
  }
  if (status === "cancelled") {
    return (
      <span className="inline-flex items-center gap-1 rounded-full bg-muted px-2 py-0.5 text-[11px] font-medium text-muted-foreground">
        <XCircle className="size-3" aria-hidden="true" />
        Cancelled
      </span>
    );
  }
  return (
    <span className="inline-flex items-center gap-1 rounded-full bg-primary/10 px-2 py-0.5 text-[11px] font-medium text-primary">
      <Clock className="size-3" aria-hidden="true" />
      Scheduled
    </span>
  );
}

function AppointmentRowItem({ appointment }: { appointment: AppointmentDetail }) {
  const servicesLabel = appointment.services
    .map((s) => s.service_name_snapshot)
    .join(", ");

  return (
    <Link
      to={`/appointments/${appointment.id}`}
      className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 px-4 py-3 hover:bg-accent/50 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring"
    >
      <div className="min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-xs font-medium tabular-nums text-muted-foreground">
            {appointment.start_local.replace("T", " ")} –{" "}
            {appointment.end_local.slice(11, 16)}
          </span>
        </div>
        <p className="text-sm font-semibold truncate mt-0.5">
          {appointment.client_name}
        </p>
        {servicesLabel && (
          <p className="text-xs text-muted-foreground truncate">
            {servicesLabel}
          </p>
        )}
      </div>
      <div className="shrink-0">
        <StatusPill status={appointment.status} />
      </div>
    </Link>
  );
}

export function DashboardPage() {
  const [period, setPeriod] = useState<DashboardPeriod>("this_week");
  const [createApptOpen, setCreateApptOpen] = useState(false);

  const { data, isLoading, isError, error } = useDashboardSummary(period);
  const { data: clientsData } = useClientsList({
    filter: "active",
    limit: 200,
    offset: 0,
  });
  const { data: servicesData } = useServicesList({ filter: "active" });
  const { data: plans = [] } = useSessionPlansList({});
  const createAppointment = useCreateAppointment();

  if (isError) {
    return (
      <div className="flex-1 min-h-0 overflow-y-auto">
        <div className="mx-auto w-full max-w-6xl p-6 sm:p-8">
          <div
            role="alert"
            className="flex items-start gap-2.5 rounded-xl border border-destructive/40 bg-card p-4 text-sm text-destructive"
          >
            <AlertCircle className="size-4 mt-0.5 shrink-0" />
            <div>
              <p className="font-medium">Unable to load dashboard</p>
              <p className="text-xs text-muted-foreground mt-0.5">
                {extractErrorMessage(error)}
              </p>
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (isLoading || !data) {
    return (
      <div className="flex-1 min-h-0 overflow-y-auto">
        <div className="mx-auto w-full max-w-6xl p-6 sm:p-8">
          <div className="rounded-xl border border-border bg-card py-16 text-center text-sm text-muted-foreground">
            Loading dashboard summary...
          </div>
        </div>
      </div>
    );
  }

  const periodRangeLabel =
    data.period_start_local === data.period_end_local
      ? data.period_start_local
      : `${data.period_start_local} to ${data.period_end_local}`;

  return (
    <PageLayout
      maxWidth="6xl"
      header={
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-semibold tracking-tight">
              {data.business_name}
            </h1>
            <p className="mt-1 text-sm text-muted-foreground">
              Operational overview for {data.today_local} ({data.timezone}).
            </p>
          </div>

          <div className="flex flex-wrap items-center gap-2">
            <div
              role="group"
              aria-label="Dashboard reporting period"
              className="inline-flex rounded-md border border-border bg-background p-0.5"
            >
              {PERIOD_OPTIONS.map((opt) => {
                const active = period === opt.value;
                return (
                  <button
                    key={opt.value}
                    type="button"
                    aria-pressed={active}
                    onClick={() => setPeriod(opt.value)}
                    className={`rounded px-2.5 py-1 text-xs font-medium transition-colors ${
                      active
                        ? "bg-primary text-primary-foreground"
                        : "text-muted-foreground hover:text-foreground"
                    }`}
                  >
                    {opt.label}
                  </button>
                );
              })}
            </div>

            <Button onClick={() => setCreateApptOpen(true)}>
              <Plus className="size-4" />
              New Appointment
            </Button>
          </div>
        </div>
      }
    >

      {/* KPI Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="rounded-xl border border-border bg-card p-4 space-y-1">
          <p className="text-xs font-medium text-muted-foreground">
            Today&apos;s Appointments
          </p>
          <p className="text-2xl font-semibold tabular-nums">
            {data.today_counts.total}
          </p>
          <p className="text-xs text-muted-foreground">
            {data.today_counts.scheduled} scheduled ·{" "}
            {data.today_counts.completed} completed ·{" "}
            {data.today_counts.cancelled} cancelled
          </p>
        </div>

        <div className="rounded-xl border border-border bg-card p-4 space-y-1">
          <p className="text-xs font-medium text-muted-foreground">
            Upcoming Scheduled
          </p>
          <p className="text-2xl font-semibold tabular-nums">
            {data.upcoming_scheduled_count}
          </p>
          <p className="text-xs text-muted-foreground">
            Active future sessions (excludes cancelled)
          </p>
        </div>

        <div className="rounded-xl border border-border bg-card p-4 space-y-1">
          <p className="text-xs font-medium text-muted-foreground">
            Completed ({PERIOD_OPTIONS.find((p) => p.value === period)?.label})
          </p>
          <p className="text-2xl font-semibold tabular-nums">
            {data.period_counts.completed}
          </p>
          <p className="text-xs text-muted-foreground">
            of {data.period_counts.total} total ({periodRangeLabel})
          </p>
        </div>

        <div className="rounded-xl border border-border bg-card p-4 space-y-1">
          <p className="text-xs font-medium text-muted-foreground">
            Needs Outcome
          </p>
          <p className="text-2xl font-semibold tabular-nums">
            {data.needs_outcome_count}
          </p>
          <p className="text-xs text-muted-foreground">
            Past-end scheduled sessions not yet completed/cancelled
          </p>
        </div>
      </div>

      {/* Compact client & session-plan context strip */}
      <div className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-border bg-muted/30 px-4 py-3 text-xs">
        <div className="flex flex-wrap items-center gap-4 text-muted-foreground">
          <span className="inline-flex items-center gap-1.5">
            <Users className="size-3.5 text-foreground" aria-hidden="true" />
            <strong className="text-foreground tabular-nums">
              {data.active_clients_count}
            </strong>{" "}
            active clients
          </span>
          <span className="inline-flex items-center gap-1.5">
            <FolderKanban
              className="size-3.5 text-foreground"
              aria-hidden="true"
            />
            <strong className="text-foreground tabular-nums">
              {data.active_session_plans_count}
            </strong>{" "}
            active session plans
          </span>
        </div>

        <div className="flex items-center gap-3">
          <Link
            to="/calendar"
            className="inline-flex items-center gap-1 font-medium text-primary hover:underline"
          >
            <CalendarDays className="size-3.5" />
            Calendar
          </Link>
          <Link
            to="/reports"
            className="inline-flex items-center gap-1 font-medium text-primary hover:underline"
          >
            <BarChart3 className="size-3.5" />
            Weekly &amp; Monthly Reports
          </Link>
        </div>
      </div>

      {/* Today's Appointments & Upcoming Appointments */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <section
          aria-label="Today's appointments"
          className="rounded-xl border border-border bg-card overflow-hidden flex flex-col"
        >
          <div className="flex items-center justify-between border-b border-border bg-muted/30 px-4 py-3">
            <h2 className="text-sm font-semibold">Today&apos;s Schedule</h2>
            <Link
              to={`/calendar?view=day&date=${data.today_local}`}
              className="text-xs font-medium text-primary hover:underline"
            >
              Open Day View
            </Link>
          </div>
          {data.today_appointments.length === 0 ? (
            <div className="flex-1 flex flex-col items-center justify-center py-10 px-6 text-center">
              <CalendarClock className="size-7 text-muted-foreground/60 mb-2" />
              <p className="text-sm font-medium">No appointments today</p>
              <p className="text-xs text-muted-foreground mt-1">
                Schedule a session or check upcoming days in Calendar.
              </p>
            </div>
          ) : (
            <div className="divide-y divide-border">
              {data.today_appointments.map((appt) => (
                <AppointmentRowItem key={appt.id} appointment={appt} />
              ))}
            </div>
          )}
        </section>

        <section
          aria-label="Upcoming appointments"
          className="rounded-xl border border-border bg-card overflow-hidden flex flex-col"
        >
          <div className="flex items-center justify-between border-b border-border bg-muted/30 px-4 py-3">
            <h2 className="text-sm font-semibold">Upcoming Scheduled</h2>
            <Link
              to="/appointments"
              className="text-xs font-medium text-primary hover:underline"
            >
              View All
            </Link>
          </div>
          {data.upcoming_appointments.length === 0 ? (
            <div className="flex-1 flex flex-col items-center justify-center py-10 px-6 text-center">
              <CalendarClock className="size-7 text-muted-foreground/60 mb-2" />
              <p className="text-sm font-medium">No upcoming appointments</p>
              <p className="text-xs text-muted-foreground mt-1">
                Scheduled future appointments will appear here.
              </p>
            </div>
          ) : (
            <div className="divide-y divide-border">
              {data.upcoming_appointments.map((appt) => (
                <AppointmentRowItem key={appt.id} appointment={appt} />
              ))}
            </div>
          )}
        </section>
      </div>

      {createApptOpen && (
        <AppointmentFormDialog
          open={createApptOpen}
          mode="create"
          clients={clientsData?.items ?? []}
          services={servicesData?.items ?? []}
          sessionPlans={plans}
          businessTimezone={data.timezone}
          isSubmitting={createAppointment.isPending}
          onClose={() => setCreateApptOpen(false)}
          onCreate={async (input) => {
            await createAppointment.mutateAsync(input);
            setCreateApptOpen(false);
          }}
        />
      )}
    </PageLayout>
  );
}
