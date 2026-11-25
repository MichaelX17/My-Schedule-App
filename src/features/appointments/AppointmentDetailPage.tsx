import { useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  ArrowLeft,
  CalendarClock,
  CheckCircle2,
  Clock,
  FolderKanban,
  Pencil,
  User,
  XCircle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useClientsList } from "@/features/clients/hooks/useClients";
import { useServicesList } from "@/features/services/hooks/useServices";
import { formatDuration } from "@/features/services/validation";
import { extractErrorMessage } from "@/services/business";
import { AppointmentFormDialog } from "./components/AppointmentFormDialog";
import {
  useAppointmentDetail,
  useCancelAppointment,
  useCompleteAppointment,
  useRescheduleAppointment,
  useSessionPlansList,
  useUpdateAppointment,
} from "./hooks/useAppointments";

export function AppointmentDetailPage() {
  const { appointmentId } = useParams<{ appointmentId: string }>();
  const {
    data: appointment,
    isLoading,
    isError,
    error,
  } = useAppointmentDetail(appointmentId);
  const { data: clientsData } = useClientsList({ filter: "active", limit: 200 });
  const { data: servicesData } = useServicesList({ filter: "all" });
  const { data: plans = [] } = useSessionPlansList({});

  const updateAppointment = useUpdateAppointment();
  const rescheduleAppointment = useRescheduleAppointment();
  const completeAppointment = useCompleteAppointment();
  const cancelAppointment = useCancelAppointment();

  const [editOpen, setEditOpen] = useState(false);
  const [rescheduleOpen, setRescheduleOpen] = useState(false);
  const [rescheduleStart, setRescheduleStart] = useState("");
  const [rescheduleEnd, setRescheduleEnd] = useState("");
  const [confirmCancelOpen, setConfirmCancelOpen] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  if (isLoading) {
    return (
      <div className="mx-auto w-full max-w-4xl p-6 sm:p-8">
        <p className="text-sm text-muted-foreground">Loading appointment...</p>
      </div>
    );
  }

  if (isError || !appointment) {
    return (
      <div className="mx-auto w-full max-w-4xl p-6 sm:p-8 space-y-4">
        <Link
          to="/appointments"
          className="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft className="size-4" />
          Back to Appointments
        </Link>
        <div
          role="alert"
          className="rounded-xl border border-destructive/40 bg-destructive/10 p-4 text-sm text-destructive"
        >
          {extractErrorMessage(error)}
        </div>
      </div>
    );
  }

  const isScheduled = appointment.status === "scheduled";

  const openReschedule = () => {
    setActionError(null);
    setRescheduleStart(appointment.start_local);
    setRescheduleEnd(appointment.end_local);
    setRescheduleOpen(true);
  };

  const handleRescheduleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setActionError(null);
    try {
      await rescheduleAppointment.mutateAsync({
        id: appointment.id,
        start_local: rescheduleStart,
        end_local: rescheduleEnd,
      });
      setRescheduleOpen(false);
    } catch (err) {
      setActionError(extractErrorMessage(err));
    }
  };

  const handleComplete = async () => {
    setActionError(null);
    try {
      await completeAppointment.mutateAsync(appointment.id);
    } catch (err) {
      setActionError(extractErrorMessage(err));
    }
  };

  const handleCancel = async () => {
    setActionError(null);
    try {
      await cancelAppointment.mutateAsync(appointment.id);
      setConfirmCancelOpen(false);
    } catch (err) {
      setActionError(extractErrorMessage(err));
    }
  };

  // Preserve any currently attached inactive services in the edit selector
  const existingServiceIds = new Set(
    appointment.services.map((s) => s.service_id)
  );
  const selectableServices = (servicesData?.items ?? []).filter(
    (s) => (s.is_active && !s.is_archived) || existingServiceIds.has(s.id)
  );

  return (
    <div className="mx-auto w-full max-w-4xl p-6 sm:p-8 space-y-6">
      <Link
        to="/appointments"
        className="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-4" />
        Back to Appointments
      </Link>

      <div className="flex flex-col sm:flex-row sm:items-start justify-between gap-4">
        <div className="space-y-1.5">
          <div className="flex flex-wrap items-center gap-2.5">
            <h1 className="text-2xl font-semibold tracking-tight">
              {appointment.client_name}
            </h1>
            <span className="inline-flex items-center gap-1 rounded-full bg-secondary px-2.5 py-0.5 text-xs font-medium capitalize text-secondary-foreground">
              {appointment.status === "completed" ? (
                <CheckCircle2 className="size-3.5 text-primary" />
              ) : appointment.status === "cancelled" ? (
                <XCircle className="size-3.5 text-muted-foreground" />
              ) : (
                <Clock className="size-3.5" />
              )}
              {appointment.status}
            </span>
          </div>
          <p className="text-sm text-muted-foreground">
            {appointment.start_local.replace("T", " ")} –{" "}
            {appointment.end_local.slice(11)} ({appointment.timezone})
          </p>
        </div>

        {isScheduled && (
          <div className="flex flex-wrap items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setEditOpen(true)}
            >
              <Pencil className="size-3.5" />
              Edit
            </Button>
            <Button variant="outline" size="sm" onClick={openReschedule}>
              <CalendarClock className="size-3.5" />
              Reschedule
            </Button>
            <Button
              size="sm"
              disabled={completeAppointment.isPending}
              onClick={handleComplete}
            >
              <CheckCircle2 className="size-3.5" />
              Mark Completed
            </Button>
            <Button
              variant="destructive"
              size="sm"
              onClick={() => setConfirmCancelOpen(true)}
            >
              <XCircle className="size-3.5" />
              Cancel
            </Button>
          </div>
        )}
      </div>

      {actionError && (
        <div
          role="alert"
          className="rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive"
        >
          {actionError}
        </div>
      )}

      <section className="rounded-xl border border-border bg-card p-6 space-y-4">
        <h2 className="text-base font-semibold">Appointment Details</h2>

        <dl className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm">
          <div className="space-y-1">
            <dt className="text-xs font-medium text-muted-foreground inline-flex items-center gap-1.5">
              <User className="size-3.5" />
              Client
            </dt>
            <dd>
              <Link
                to={`/clients/${appointment.client_id}`}
                className="underline hover:text-primary"
              >
                {appointment.client_name}
              </Link>
              {appointment.client_is_archived && (
                <span className="ml-2 text-xs text-muted-foreground">
                  (Archived)
                </span>
              )}
            </dd>
          </div>

          <div className="space-y-1">
            <dt className="text-xs font-medium text-muted-foreground inline-flex items-center gap-1.5">
              <FolderKanban className="size-3.5" />
              Session Plan
            </dt>
            <dd>
              {appointment.session_plan_title ?? "Independent appointment"}
            </dd>
          </div>

          <div className="sm:col-span-2 space-y-1">
            <dt className="text-xs font-medium text-muted-foreground">
              Booked Services (Historical Snapshots)
            </dt>
            <dd>
              <ul className="mt-1 space-y-1.5">
                {appointment.services.map((svc) => (
                  <li
                    key={svc.id}
                    className="flex items-center justify-between rounded-md border border-border bg-muted/20 px-3 py-2 text-sm"
                  >
                    <span className="font-medium">
                      {svc.service_name_snapshot}
                    </span>
                    <span className="text-xs text-muted-foreground">
                      {formatDuration(svc.duration_minutes_snapshot)}
                    </span>
                  </li>
                ))}
              </ul>
            </dd>
          </div>

          <div className="sm:col-span-2 space-y-1">
            <dt className="text-xs font-medium text-muted-foreground">Notes</dt>
            <dd className="whitespace-pre-wrap text-muted-foreground">
              {appointment.notes ?? "No notes recorded."}
            </dd>
          </div>
        </dl>
      </section>

      {editOpen && (
        <AppointmentFormDialog
          open={editOpen}
          mode="edit"
          initialAppointment={appointment}
          clients={clientsData?.items ?? []}
          services={selectableServices}
          sessionPlans={plans}
          businessTimezone={appointment.timezone}
          isSubmitting={updateAppointment.isPending}
          onClose={() => setEditOpen(false)}
          onUpdate={async (input) => {
            await updateAppointment.mutateAsync(input);
          }}
        />
      )}

      {rescheduleOpen && (
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="reschedule-title"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-4"
        >
          <form
            onSubmit={handleRescheduleSubmit}
            className="w-full max-w-md rounded-xl border border-border bg-card p-6 text-card-foreground shadow-lg space-y-4"
          >
            <h2 id="reschedule-title" className="text-lg font-semibold">
              Reschedule Appointment
            </h2>
            <div className="space-y-3">
              <div className="space-y-1.5">
                <Label htmlFor="reschedule-start">
                  New Start ({appointment.timezone})
                </Label>
                <Input
                  id="reschedule-start"
                  type="datetime-local"
                  value={rescheduleStart}
                  onChange={(e) => setRescheduleStart(e.target.value)}
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="reschedule-end">
                  New End ({appointment.timezone})
                </Label>
                <Input
                  id="reschedule-end"
                  type="datetime-local"
                  value={rescheduleEnd}
                  onChange={(e) => setRescheduleEnd(e.target.value)}
                />
              </div>
            </div>
            <div className="flex justify-end gap-2 pt-2">
              <Button
                type="button"
                variant="outline"
                onClick={() => setRescheduleOpen(false)}
                disabled={rescheduleAppointment.isPending}
              >
                Cancel
              </Button>
              <Button type="submit" disabled={rescheduleAppointment.isPending}>
                {rescheduleAppointment.isPending ? "Saving..." : "Reschedule"}
              </Button>
            </div>
          </form>
        </div>
      )}

      {confirmCancelOpen && (
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="cancel-appt-title"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-4"
        >
          <div className="w-full max-w-md rounded-xl border border-border bg-card p-6 text-card-foreground shadow-lg space-y-4">
            <h2 id="cancel-appt-title" className="text-lg font-semibold">
              Cancel this appointment?
            </h2>
            <p className="text-sm text-muted-foreground">
              Cancelled appointments remain in history and cannot be
              rescheduled or marked completed.
            </p>
            <div className="flex justify-end gap-2 pt-2">
              <Button
                variant="outline"
                onClick={() => setConfirmCancelOpen(false)}
                disabled={cancelAppointment.isPending}
              >
                Keep Scheduled
              </Button>
              <Button
                variant="destructive"
                onClick={handleCancel}
                disabled={cancelAppointment.isPending}
              >
                {cancelAppointment.isPending
                  ? "Cancelling..."
                  : "Cancel Appointment"}
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
