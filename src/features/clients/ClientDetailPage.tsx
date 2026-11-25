import { useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import {
  Archive,
  ArchiveRestore,
  ArrowLeft,
  Calendar,
  CheckCircle2,
  Mail,
  MapPin,
  Pencil,
  Phone,
  StickyNote,
  Trash2,
} from "lucide-react";
import { PageLayout } from "@/components/layout/PageLayout";
import { Button } from "@/components/ui/button";
import { useAppointmentsList } from "@/features/appointments/hooks/useAppointments";
import { extractErrorMessage } from "@/services/business";
import { ClientFormDialog } from "./components/ClientFormDialog";
import {
  useArchiveClient,
  useClientDetail,
  useDeleteClient,
  useRestoreClient,
  useUpdateClient,
} from "./hooks/useClients";
import type { normalizeClientFormInput } from "./validation";

export function ClientDetailPage() {
  const { clientId } = useParams<{ clientId: string }>();
  const navigate = useNavigate();
  const { data: client, isLoading, isError, error } = useClientDetail(clientId);
  const { data: clientAppointments = [], isLoading: loadingHistory } =
    useAppointmentsList({ client_id: clientId ?? null });
  const updateClient = useUpdateClient();
  const archiveClient = useArchiveClient();
  const restoreClient = useRestoreClient();
  const deleteClient = useDeleteClient();

  const [editOpen, setEditOpen] = useState(false);
  const [confirmArchiveAction, setConfirmArchiveAction] = useState<
    "archive" | "restore" | null
  >(null);
  const [confirmDeleteOpen, setConfirmDeleteOpen] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const handleUpdateSubmit = async (
    values: ReturnType<typeof normalizeClientFormInput>
  ) => {
    if (!client) return;
    await updateClient.mutateAsync({
      id: client.id,
      ...values,
    });
  };

  const handleConfirmArchiveToggle = async () => {
    if (!client || !confirmArchiveAction) return;
    setActionError(null);
    try {
      if (confirmArchiveAction === "archive") {
        await archiveClient.mutateAsync(client.id);
      } else {
        await restoreClient.mutateAsync(client.id);
      }
      setConfirmArchiveAction(null);
    } catch (err) {
      setActionError(extractErrorMessage(err));
    }
  };

  const handleConfirmDelete = async () => {
    if (!client) return;
    setActionError(null);
    try {
      await deleteClient.mutateAsync(client.id);
      setConfirmDeleteOpen(false);
      navigate("/clients");
    } catch (err) {
      setConfirmDeleteOpen(false);
      setActionError(extractErrorMessage(err));
    }
  };

  if (isLoading) {
    return (
      <div className="flex-1 min-h-0 overflow-y-auto">
        <div className="mx-auto w-full max-w-4xl p-6 sm:p-8">
          <p className="text-sm text-muted-foreground">Loading client...</p>
        </div>
      </div>
    );
  }

  if (isError || !client) {
    return (
      <div className="flex-1 min-h-0 overflow-y-auto">
        <div className="mx-auto w-full max-w-4xl p-6 sm:p-8 space-y-4">
          <Link
            to="/clients"
            className="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground"
          >
            <ArrowLeft className="size-4" />
            Back to Clients
          </Link>
          <div
            role="alert"
            className="rounded-xl border border-destructive/40 bg-destructive/10 p-4 text-sm text-destructive"
          >
            {extractErrorMessage(error)}
          </div>
        </div>
      </div>
    );
  }

  const isTogglingArchive = archiveClient.isPending || restoreClient.isPending;

  return (
    <PageLayout
      maxWidth="4xl"
      header={
        <div className="space-y-3">
          <Link
            to="/clients"
            className="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground"
          >
            <ArrowLeft className="size-4" />
            Back to Clients
          </Link>

          <div className="flex flex-col sm:flex-row sm:items-start justify-between gap-4">
            <div className="space-y-1.5">
              <div className="flex flex-wrap items-center gap-2.5">
                <h1 className="text-2xl font-semibold tracking-tight">
                  {client.full_name}
                </h1>
                {client.is_archived ? (
                  <span className="inline-flex items-center gap-1 rounded-full bg-secondary px-2.5 py-0.5 text-xs font-medium text-secondary-foreground">
                    <Archive className="size-3.5" />
                    Archived
                  </span>
                ) : (
                  <span className="inline-flex items-center gap-1 rounded-full bg-secondary px-2.5 py-0.5 text-xs font-medium text-secondary-foreground">
                    <CheckCircle2 className="size-3.5 text-primary" />
                    Active
                  </span>
                )}
              </div>
              <p className="text-xs text-muted-foreground">
                Created {new Date(client.created_at).toLocaleDateString()} ·
                Last updated {new Date(client.updated_at).toLocaleDateString()}
              </p>
            </div>

            <div className="flex flex-wrap items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setEditOpen(true)}
              >
                <Pencil className="size-3.5" />
                Edit
              </Button>
              {client.is_archived ? (
                <>
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={() => {
                      setActionError(null);
                      setConfirmArchiveAction("restore");
                    }}
                  >
                    <ArchiveRestore className="size-3.5" />
                    Restore Client
                  </Button>
                  <Button
                    variant="destructive"
                    size="sm"
                    onClick={() => {
                      setActionError(null);
                      setConfirmDeleteOpen(true);
                    }}
                  >
                    <Trash2 className="size-3.5" />
                    Delete Permanently
                  </Button>
                </>
              ) : (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => {
                    setActionError(null);
                    setConfirmArchiveAction("archive");
                  }}
                >
                  <Archive className="size-3.5" />
                  Archive Client
                </Button>
              )}
            </div>
          </div>
        </div>
      }
    >

      {actionError && (
        <div
          role="alert"
          className="rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive"
        >
          {actionError}
        </div>
      )}

      <section className="rounded-xl border border-border bg-card p-6 space-y-4">
        <h2 className="text-base font-semibold">Contact &amp; Notes</h2>

        <dl className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm">
          <div className="space-y-1">
            <dt className="text-xs font-medium text-muted-foreground inline-flex items-center gap-1.5">
              <Phone className="size-3.5" />
              Phone
            </dt>
            <dd>{client.phone ?? "Not provided"}</dd>
          </div>

          <div className="space-y-1">
            <dt className="text-xs font-medium text-muted-foreground inline-flex items-center gap-1.5">
              <Mail className="size-3.5" />
              Email
            </dt>
            <dd>{client.email ?? "Not provided"}</dd>
          </div>

          <div className="sm:col-span-2 space-y-1">
            <dt className="text-xs font-medium text-muted-foreground inline-flex items-center gap-1.5">
              <MapPin className="size-3.5" />
              Address
            </dt>
            <dd>{client.address ?? "Not provided"}</dd>
          </div>

          <div className="sm:col-span-2 space-y-1">
            <dt className="text-xs font-medium text-muted-foreground inline-flex items-center gap-1.5">
              <StickyNote className="size-3.5" />
              General Notes
            </dt>
            <dd className="whitespace-pre-wrap text-muted-foreground">
              {client.notes ?? "No notes recorded."}
            </dd>
          </div>
        </dl>
      </section>

      <section className="rounded-xl border border-border bg-card p-6 space-y-3">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <Calendar className="size-4 text-muted-foreground" />
            <h2 className="text-base font-semibold">
              Appointment History ({clientAppointments.length})
            </h2>
          </div>
          <Link to="/appointments">
            <Button variant="outline" size="sm">
              Manage Appointments
            </Button>
          </Link>
        </div>

        {loadingHistory ? (
          <p className="text-xs text-muted-foreground">Loading history...</p>
        ) : clientAppointments.length === 0 ? (
          <div className="rounded-lg border border-dashed border-border bg-muted/20 p-6 text-center space-y-1">
            <p className="text-sm font-medium">No appointments recorded yet</p>
            <p className="text-xs text-muted-foreground">
              Appointments scheduled for this client will appear here.
            </p>
          </div>
        ) : (
          <ul className="divide-y divide-border rounded-lg border border-border overflow-hidden">
            {clientAppointments.map((appt) => (
              <li key={appt.id}>
                <Link
                  to={`/appointments/${appt.id}`}
                  className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 px-4 py-3 hover:bg-accent/40 transition-colors"
                >
                  <div className="space-y-0.5">
                    <p className="text-sm font-medium">
                      {appt.start_local.replace("T", " ")} –{" "}
                      {appt.end_local.slice(11)} ({appt.timezone})
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {appt.services
                        .map((s) => s.service_name_snapshot)
                        .join(", ")}
                    </p>
                  </div>
                  <span className="rounded-full bg-secondary px-2.5 py-0.5 text-xs font-medium capitalize text-secondary-foreground self-start sm:self-center">
                    {appt.status}
                  </span>
                </Link>
              </li>
            ))}
          </ul>
        )}
      </section>

      <ClientFormDialog
        open={editOpen}
        mode="edit"
        initialClient={client}
        isSubmitting={updateClient.isPending}
        onClose={() => setEditOpen(false)}
        onSubmit={handleUpdateSubmit}
      />

      {confirmArchiveAction && (
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="archive-confirm-title"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-4"
        >
          <div className="w-full max-w-md rounded-xl border border-border bg-card p-6 text-card-foreground shadow-lg space-y-4">
            <h2 id="archive-confirm-title" className="text-lg font-semibold">
              {confirmArchiveAction === "archive"
                ? "Archive this client?"
                : "Restore this client?"}
            </h2>
            <p className="text-sm text-muted-foreground leading-relaxed">
              {confirmArchiveAction === "archive"
                ? "Archiving hides this client from the default active list and future new-appointment pickers, while keeping all historical records intact. You can restore this client at any time."
                : "Restoring moves this client back to the active list so they can be selected for new appointments again."}
            </p>
            <div className="flex items-center justify-end gap-2 pt-2">
              <Button
                variant="outline"
                onClick={() => setConfirmArchiveAction(null)}
                disabled={isTogglingArchive}
              >
                Cancel
              </Button>
              <Button
                onClick={handleConfirmArchiveToggle}
                disabled={isTogglingArchive}
              >
                {isTogglingArchive
                  ? "Updating..."
                  : confirmArchiveAction === "archive"
                    ? "Archive Client"
                    : "Restore Client"}
              </Button>
            </div>
          </div>
        </div>
      )}
      {confirmDeleteOpen && (
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="delete-client-title"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-4"
        >
          <div className="w-full max-w-md rounded-xl border border-border bg-card p-6 text-card-foreground shadow-lg space-y-4">
            <h2 id="delete-client-title" className="text-lg font-semibold">
              Permanently delete &ldquo;{client.full_name}&rdquo;?
            </h2>
            <p className="text-sm text-muted-foreground leading-relaxed">
              This action cannot be undone. Archived clients with existing
              appointments or session plans cannot be deleted in order to
              preserve schedule history.
            </p>
            <div className="flex items-center justify-end gap-2 pt-2">
              <Button
                variant="outline"
                onClick={() => setConfirmDeleteOpen(false)}
                disabled={deleteClient.isPending}
              >
                Cancel
              </Button>
              <Button
                variant="destructive"
                onClick={handleConfirmDelete}
                disabled={deleteClient.isPending}
              >
                {deleteClient.isPending ? "Deleting..." : "Delete Permanently"}
              </Button>
            </div>
          </div>
        </div>
      )}
    </PageLayout>
  );
}
