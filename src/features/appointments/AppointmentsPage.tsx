import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import {
  Archive,
  CalendarClock,
  CheckCircle2,
  Clock,
  FolderKanban,
  Plus,
  Search,
  Trash2,
  XCircle,
} from "lucide-react";
import { PageLayout } from "@/components/layout/PageLayout";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useClientsList } from "@/features/clients/hooks/useClients";
import { useOnboardingStatus } from "@/features/onboarding/hooks/useBusinessProfile";
import { useServicesList } from "@/features/services/hooks/useServices";
import type {
  AppointmentDetail,
  AppointmentStatus,
  SessionPlanSummary,
} from "@/services/appointments";
import { extractErrorMessage } from "@/services/business";
import { AppointmentFormDialog } from "./components/AppointmentFormDialog";
import { SessionPlanFormDialog } from "./components/SessionPlanFormDialog";
import {
  useAppointmentsList,
  useArchiveSessionPlan,
  useCreateAppointment,
  useCreateSessionPlanWithAppointments,
  useDeleteAppointment,
  useDeleteSessionPlan,
  useSessionPlansList,
} from "./hooks/useAppointments";

const STATUS_TABS: { value: AppointmentStatus | "all"; label: string }[] = [
  { value: "scheduled", label: "Scheduled" },
  { value: "completed", label: "Completed" },
  { value: "cancelled", label: "Cancelled" },
  { value: "all", label: "All" },
];

export function AppointmentsPage() {
  const [activeTab, setActiveTab] = useState<"appointments" | "plans">(
    "appointments"
  );
  const [statusFilter, setStatusFilter] = useState<AppointmentStatus | "all">(
    "scheduled"
  );
  const [searchInput, setSearchInput] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [createApptOpen, setCreateApptOpen] = useState(false);
  const [createPlanOpen, setCreatePlanOpen] = useState(false);
  const [confirmDeleteAppt, setConfirmDeleteAppt] =
    useState<AppointmentDetail | null>(null);
  const [confirmDeletePlan, setConfirmDeletePlan] =
    useState<SessionPlanSummary | null>(null);
  const [deletePlanLinkedAppts, setDeletePlanLinkedAppts] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setDebouncedSearch(searchInput.trim());
    }, 200);
    return () => window.clearTimeout(timer);
  }, [searchInput]);

  const { data: onboarding } = useOnboardingStatus();
  const businessTimezone = onboarding?.business?.timezone ?? "UTC";

  const {
    data: appointments = [],
    isLoading: loadingAppts,
    isError: apptError,
    error: apptErrObj,
  } = useAppointmentsList({
    search: debouncedSearch || null,
    status: statusFilter === "all" ? null : statusFilter,
  });

  const { data: plans = [], isLoading: loadingPlans } = useSessionPlansList({});
  const { data: clientsData } = useClientsList({
    filter: "active",
    limit: 200,
    offset: 0,
  });
  const { data: servicesData } = useServicesList({ filter: "active" });

  const createAppointment = useCreateAppointment();
  const createSessionPlanWithAppointments =
    useCreateSessionPlanWithAppointments();
  const archiveSessionPlan = useArchiveSessionPlan();
  const deleteAppointment = useDeleteAppointment();
  const deleteSessionPlan = useDeleteSessionPlan();

  const activeClients = clientsData?.items ?? [];
  const activeServices = servicesData?.items ?? [];

  const handleArchivePlan = async (id: string) => {
    setActionError(null);
    try {
      await archiveSessionPlan.mutateAsync(id);
    } catch (err) {
      setActionError(extractErrorMessage(err));
    }
  };

  const handleConfirmDeleteAppt = async () => {
    if (!confirmDeleteAppt) return;
    setActionError(null);
    try {
      await deleteAppointment.mutateAsync(confirmDeleteAppt.id);
      setConfirmDeleteAppt(null);
    } catch (err) {
      setConfirmDeleteAppt(null);
      setActionError(extractErrorMessage(err));
    }
  };

  const handleConfirmDeletePlan = async () => {
    if (!confirmDeletePlan) return;
    setActionError(null);
    try {
      await deleteSessionPlan.mutateAsync({
        id: confirmDeletePlan.id,
        deleteLinkedAppointments:
          confirmDeletePlan.linked_total > 0 && deletePlanLinkedAppts,
      });
      setConfirmDeletePlan(null);
      setDeletePlanLinkedAppts(false);
    } catch (err) {
      setConfirmDeletePlan(null);
      setActionError(extractErrorMessage(err));
    }
  };

  return (
    <PageLayout
      maxWidth="5xl"
      header={
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-semibold tracking-tight">
              Appointments &amp; Plans
            </h1>
            <p className="mt-1 text-sm text-muted-foreground">
              Schedule individual sessions, link optional multi-session plans,
              and track appointment outcomes ({businessTimezone}).
            </p>
          </div>

          <div className="flex flex-wrap items-center gap-2">
            <Button variant="outline" onClick={() => setCreatePlanOpen(true)}>
              <FolderKanban className="size-4" />
              New Session Plan
            </Button>
            <Button onClick={() => setCreateApptOpen(true)}>
              <Plus className="size-4" />
              New Appointment
            </Button>
          </div>
        </div>
      }
    >

      <div className="flex items-center gap-2 border-b border-border">
        <button
          type="button"
          onClick={() => setActiveTab("appointments")}
          className={`inline-flex items-center gap-2 border-b-2 px-3 py-2 text-sm font-medium transition-colors ${
            activeTab === "appointments"
              ? "border-primary text-foreground"
              : "border-transparent text-muted-foreground hover:text-foreground"
          }`}
        >
          <CalendarClock className="size-4" />
          <span>Appointments</span>
        </button>
        <button
          type="button"
          onClick={() => setActiveTab("plans")}
          className={`inline-flex items-center gap-2 border-b-2 px-3 py-2 text-sm font-medium transition-colors ${
            activeTab === "plans"
              ? "border-primary text-foreground"
              : "border-transparent text-muted-foreground hover:text-foreground"
          }`}
        >
          <FolderKanban className="size-4" />
          <span>Session Plans ({plans.length})</span>
        </button>
      </div>

      {(apptError || actionError) && (
        <div
          role="alert"
          className="rounded-xl border border-destructive/40 bg-destructive/10 p-4 text-sm text-destructive"
        >
          {actionError ?? extractErrorMessage(apptErrObj)}
        </div>
      )}

      {activeTab === "appointments" ? (
        <div className="space-y-4">
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-3">
            <div
              role="tablist"
              aria-label="Appointment status filter"
              className="inline-flex flex-wrap items-center rounded-lg border border-border bg-card p-1 self-start"
            >
              {STATUS_TABS.map((tab) => {
                const selected = statusFilter === tab.value;
                return (
                  <button
                    key={tab.value}
                    type="button"
                    role="tab"
                    aria-selected={selected}
                    onClick={() => setStatusFilter(tab.value)}
                    className={`rounded-md px-3 py-1.5 text-xs font-medium transition-colors ${
                      selected
                        ? "bg-primary text-primary-foreground"
                        : "text-muted-foreground hover:text-foreground"
                    }`}
                  >
                    {tab.label}
                  </button>
                );
              })}
            </div>

            <div className="relative flex-1 md:max-w-sm">
              <Search className="pointer-events-none absolute left-2.5 top-2.5 size-4 text-muted-foreground" />
              <Input
                value={searchInput}
                onChange={(e) => setSearchInput(e.target.value)}
                placeholder="Search client, service, or notes..."
                aria-label="Search appointments"
                className="pl-8"
              />
            </div>
          </div>

          {loadingAppts ? (
            <div className="rounded-xl border border-border bg-card p-8 text-center text-sm text-muted-foreground">
              Loading appointments...
            </div>
          ) : appointments.length === 0 ? (
            <div className="rounded-xl border border-border bg-card p-10 text-center space-y-3">
              <div className="mx-auto size-10 rounded-full bg-secondary flex items-center justify-center text-muted-foreground">
                <CalendarClock className="size-5" />
              </div>
              <h2 className="text-base font-medium">No appointments found</h2>
              <p className="text-sm text-muted-foreground max-w-sm mx-auto">
                {debouncedSearch
                  ? `No appointments matched "${debouncedSearch}".`
                  : "Schedule a new appointment to get started."}
              </p>
              {!debouncedSearch && (
                <Button size="sm" onClick={() => setCreateApptOpen(true)}>
                  <Plus className="size-4" />
                  Schedule Appointment
                </Button>
              )}
            </div>
          ) : (
            <div className="rounded-xl border border-border bg-card overflow-hidden">
              <ul className="divide-y divide-border">
                {appointments.map((appt) => (
                  <li
                    key={appt.id}
                    className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 px-5 py-4 hover:bg-accent/40 transition-colors"
                  >
                    <Link
                      to={`/appointments/${appt.id}`}
                      className="min-w-0 flex-1 space-y-1 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring rounded"
                    >
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="font-medium text-sm">
                          {appt.client_name}
                        </span>
                        <span className="inline-flex items-center gap-1 rounded-full bg-secondary px-2 py-0.5 text-[11px] font-medium text-secondary-foreground capitalize">
                          {appt.status === "completed" ? (
                            <CheckCircle2 className="size-3 text-primary" />
                          ) : appt.status === "cancelled" ? (
                            <XCircle className="size-3 text-muted-foreground" />
                          ) : (
                            <Clock className="size-3" />
                          )}
                          {appt.status}
                        </span>
                        {appt.session_plan_title && (
                          <span className="inline-flex items-center gap-1 rounded-full bg-secondary/70 px-2 py-0.5 text-[11px] text-muted-foreground">
                            <FolderKanban className="size-3" />
                            {appt.session_plan_title}
                          </span>
                        )}
                      </div>
                      <p className="text-xs text-muted-foreground">
                        {appt.start_local.replace("T", " ")} –{" "}
                        {appt.end_local.slice(11)} ({appt.timezone}) ·{" "}
                        {appt.services
                          .map((s) => s.service_name_snapshot)
                          .join(", ")}
                      </p>
                    </Link>

                    <div className="flex items-center gap-2 shrink-0">
                      <Link
                        to={`/appointments/${appt.id}`}
                        className="text-xs font-medium text-muted-foreground hover:text-foreground px-2 py-1 rounded-md hover:bg-accent/60 transition-colors"
                      >
                        Open →
                      </Link>
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        className="size-8 text-muted-foreground hover:text-destructive hover:bg-destructive/10"
                        aria-label={`Delete appointment for ${appt.client_name}`}
                        onClick={() => {
                          setActionError(null);
                          setConfirmDeleteAppt(appt);
                        }}
                      >
                        <Trash2 className="size-4" />
                      </Button>
                    </div>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      ) : (
        <div className="space-y-4">
          {loadingPlans ? (
            <div className="rounded-xl border border-border bg-card p-8 text-center text-sm text-muted-foreground">
              Loading session plans...
            </div>
          ) : plans.length === 0 ? (
            <div className="rounded-xl border border-border bg-card p-10 text-center space-y-3">
              <div className="mx-auto size-10 rounded-full bg-secondary flex items-center justify-center text-muted-foreground">
                <FolderKanban className="size-5" />
              </div>
              <h2 className="text-base font-medium">No session plans yet</h2>
              <p className="text-sm text-muted-foreground max-w-sm mx-auto">
                Session plans are optional groupings for a client&apos;s
                multi-session series. Independent appointments work without a
                plan.
              </p>
              <Button size="sm" onClick={() => setCreatePlanOpen(true)}>
                <Plus className="size-4" />
                Create Session Plan
              </Button>
            </div>
          ) : (
            <div className="rounded-xl border border-border bg-card overflow-hidden">
              <ul className="divide-y divide-border">
                {plans.map((plan) => (
                  <li
                    key={plan.id}
                    className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 px-5 py-4"
                  >
                    <div className="space-y-1 min-w-0">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="font-medium text-sm">
                          {plan.title}
                        </span>
                        <span className="rounded-full bg-secondary px-2 py-0.5 text-[11px] font-medium capitalize text-secondary-foreground">
                          {plan.status}
                        </span>
                      </div>
                      <p className="text-xs text-muted-foreground">
                        Client:{" "}
                        <Link
                          to={`/clients/${plan.client_id}`}
                          className="underline hover:text-foreground"
                        >
                          {plan.client_name}
                        </Link>{" "}
                        · Planned: {plan.planned_sessions} · Linked:{" "}
                        {plan.linked_total} ({plan.linked_completed} completed,{" "}
                        {plan.linked_scheduled} scheduled)
                      </p>
                      {plan.description && (
                        <p className="text-xs text-muted-foreground">
                          {plan.description}
                        </p>
                      )}
                    </div>

                    <div className="flex items-center gap-1.5 shrink-0">
                      {plan.status !== "archived" && (
                        <Button
                          variant="ghost"
                          size="sm"
                          disabled={archiveSessionPlan.isPending}
                          onClick={() => handleArchivePlan(plan.id)}
                        >
                          <Archive className="size-3.5" />
                          Archive
                        </Button>
                      )}
                      <Button
                        variant="ghost"
                        size="sm"
                        className="text-destructive hover:text-destructive hover:bg-destructive/10"
                        disabled={deleteSessionPlan.isPending}
                        onClick={() => {
                          setActionError(null);
                          setDeletePlanLinkedAppts(false);
                          setConfirmDeletePlan(plan);
                        }}
                      >
                        <Trash2 className="size-3.5" />
                        Delete
                      </Button>
                    </div>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}

      {createApptOpen && (
        <AppointmentFormDialog
          open={createApptOpen}
          mode="create"
          clients={activeClients}
          services={activeServices}
          sessionPlans={plans}
          businessTimezone={businessTimezone}
          isSubmitting={createAppointment.isPending}
          onClose={() => setCreateApptOpen(false)}
          onCreate={async (input) => {
            await createAppointment.mutateAsync(input);
          }}
        />
      )}

      <SessionPlanFormDialog
        open={createPlanOpen}
        clients={activeClients}
        services={activeServices}
        businessTimezone={businessTimezone}
        isSubmitting={createSessionPlanWithAppointments.isPending}
        onClose={() => setCreatePlanOpen(false)}
        onSubmit={async (values) => {
          await createSessionPlanWithAppointments.mutateAsync(values);
        }}
      />

      {confirmDeleteAppt && (
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="delete-list-appt-title"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-4"
        >
          <div className="w-full max-w-md rounded-xl border border-border bg-card p-6 text-card-foreground shadow-lg space-y-4">
            <h2 id="delete-list-appt-title" className="text-lg font-semibold">
              Delete appointment for {confirmDeleteAppt.client_name}?
            </h2>
            <p className="text-sm text-muted-foreground leading-relaxed">
              {confirmDeleteAppt.session_plan_title ? (
                <>
                  This appointment is linked to session plan{" "}
                  <span className="font-medium text-foreground">
                    &ldquo;{confirmDeleteAppt.session_plan_title}&rdquo;
                  </span>
                  . Deleting it will permanently remove this appointment and
                  update the session plan to remove this session.
                </>
              ) : (
                <>
                  Scheduled for{" "}
                  <span className="font-medium text-foreground">
                    {confirmDeleteAppt.start_local.replace("T", " ")}
                  </span>
                  . This action cannot be undone.
                </>
              )}
            </p>
            <div className="flex justify-end gap-2 pt-2">
              <Button
                variant="outline"
                onClick={() => setConfirmDeleteAppt(null)}
                disabled={deleteAppointment.isPending}
              >
                Cancel
              </Button>
              <Button
                variant="destructive"
                onClick={handleConfirmDeleteAppt}
                disabled={deleteAppointment.isPending}
              >
                {deleteAppointment.isPending
                  ? "Deleting..."
                  : "Delete Appointment"}
              </Button>
            </div>
          </div>
        </div>
      )}

      {confirmDeletePlan && (
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="delete-plan-title"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-4"
        >
          <div className="w-full max-w-md rounded-xl border border-border bg-card p-6 text-card-foreground shadow-lg space-y-4">
            <h2 id="delete-plan-title" className="text-lg font-semibold">
              Delete session plan &ldquo;{confirmDeletePlan.title}&rdquo;?
            </h2>

            {confirmDeletePlan.linked_total > 0 ? (
              <div className="space-y-3 text-sm">
                <p className="text-muted-foreground leading-relaxed">
                  This session plan for{" "}
                  <span className="font-medium text-foreground">
                    {confirmDeletePlan.client_name}
                  </span>{" "}
                  has{" "}
                  <span className="font-medium text-foreground">
                    {confirmDeletePlan.linked_total}{" "}
                    {confirmDeletePlan.linked_total === 1
                      ? "linked appointment"
                      : "linked appointments"}
                  </span>
                  . Choose how to handle them:
                </p>

                <div className="space-y-2">
                  <label className="flex items-start gap-2.5 rounded-lg bg-secondary/60 p-3 cursor-pointer">
                    <input
                      type="radio"
                      name="delete-plan-mode"
                      checked={!deletePlanLinkedAppts}
                      onChange={() => setDeletePlanLinkedAppts(false)}
                      className="mt-0.5"
                    />
                    <span className="text-xs leading-relaxed">
                      <span className="block font-medium text-foreground">
                        Delete plan only (keep appointments)
                      </span>
                      <span className="text-muted-foreground">
                        Removes the session plan and keeps all{" "}
                        {confirmDeletePlan.linked_total} appointments as
                        independent sessions.
                      </span>
                    </span>
                  </label>

                  <label className="flex items-start gap-2.5 rounded-lg bg-destructive/10 p-3 cursor-pointer">
                    <input
                      type="radio"
                      name="delete-plan-mode"
                      checked={deletePlanLinkedAppts}
                      onChange={() => setDeletePlanLinkedAppts(true)}
                      className="mt-0.5"
                    />
                    <span className="text-xs leading-relaxed">
                      <span className="block font-medium text-destructive">
                        Delete plan and all {confirmDeletePlan.linked_total}{" "}
                        linked{" "}
                        {confirmDeletePlan.linked_total === 1
                          ? "appointment"
                          : "appointments"}
                      </span>
                      <span className="text-muted-foreground">
                        Permanently removes the session plan, its appointments,
                        and their reminders.
                      </span>
                    </span>
                  </label>
                </div>
              </div>
            ) : (
              <p className="text-sm text-muted-foreground leading-relaxed">
                This session plan has no linked appointments. Deleting it cannot
                be undone.
              </p>
            )}

            <div className="flex justify-end gap-2 pt-2">
              <Button
                variant="outline"
                onClick={() => {
                  setConfirmDeletePlan(null);
                  setDeletePlanLinkedAppts(false);
                }}
                disabled={deleteSessionPlan.isPending}
              >
                Cancel
              </Button>
              <Button
                variant="destructive"
                onClick={handleConfirmDeletePlan}
                disabled={deleteSessionPlan.isPending}
              >
                {deleteSessionPlan.isPending
                  ? "Deleting..."
                  : confirmDeletePlan.linked_total > 0 && deletePlanLinkedAppts
                    ? "Delete Plan & Appointments"
                    : "Delete Session Plan"}
              </Button>
            </div>
          </div>
        </div>
      )}
    </PageLayout>
  );
}
