import { useEffect, useMemo, useState } from "react";
import { AlertTriangle, UserPlus, X } from "lucide-react";
import { DateTimeLocalPicker } from "@/components/shared/DateTimeLocalPicker";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { formatDuration } from "@/features/services/validation";
import {
  checkAppointmentOverlaps,
  type AppointmentDetail,
  type CreateAppointmentInput,
  type DstDisambiguation,
  type OverlapWarning,
  type SessionPlanSummary,
  type UpdateAppointmentInput,
} from "@/services/appointments";
import { extractErrorMessage } from "@/services/business";
import type { Client } from "@/services/clients";
import type { Service } from "@/services/serviceCatalog";
import { isTauriRuntime } from "@/services/tauri";

function defaultLocalSlot(): { start: string; end: string } {
  const now = new Date();
  now.setMinutes(0, 0, 0);
  now.setHours(now.getHours() + 1);
  const end = new Date(now.getTime() + 60 * 60 * 1000);
  const fmt = (d: Date) => {
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
  };
  return { start: fmt(now), end: fmt(end) };
}

function addMinutesToLocal(startLocal: string, minutes: number): string {
  const match = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})$/.exec(startLocal);
  if (!match) return startLocal;
  const [, y, mo, d, h, mi] = match;
  const dt = new Date(
    Number(y),
    Number(mo) - 1,
    Number(d),
    Number(h),
    Number(mi) + minutes
  );
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${dt.getFullYear()}-${pad(dt.getMonth() + 1)}-${pad(dt.getDate())}T${pad(dt.getHours())}:${pad(dt.getMinutes())}`;
}

interface AppointmentFormDialogProps {
  open: boolean;
  mode: "create" | "edit";
  initialAppointment?: AppointmentDetail | null;
  clients: Client[];
  services: Service[];
  sessionPlans: SessionPlanSummary[];
  businessTimezone: string;
  isSubmitting?: boolean;
  onClose: () => void;
  onCreate?: (input: CreateAppointmentInput) => Promise<void>;
  onUpdate?: (input: UpdateAppointmentInput) => Promise<void>;
}

export function AppointmentFormDialog({
  open,
  mode,
  initialAppointment,
  clients,
  services,
  sessionPlans,
  businessTimezone,
  isSubmitting = false,
  onClose,
  onCreate,
  onUpdate,
}: AppointmentFormDialogProps) {
  const defaults = useMemo(() => defaultLocalSlot(), []);
  const [clientMode, setClientMode] = useState<"existing" | "inline">("existing");
  const [clientId, setClientId] = useState(
    initialAppointment?.client_id ?? clients[0]?.id ?? ""
  );
  const [inlineName, setInlineName] = useState("");
  const [inlinePhone, setInlinePhone] = useState("");
  const [inlineEmail, setInlineEmail] = useState("");
  const [sessionPlanId, setSessionPlanId] = useState(
    initialAppointment?.session_plan_id ?? ""
  );
  const [selectedServiceIds, setSelectedServiceIds] = useState<string[]>(
    initialAppointment?.services.map((s) => s.service_id) ?? []
  );
  const [startLocal, setStartLocal] = useState(
    initialAppointment?.start_local ?? defaults.start
  );
  const [endLocal, setEndLocal] = useState(
    initialAppointment?.end_local ?? defaults.end
  );
  const [dstChoice, setDstChoice] = useState<DstDisambiguation | "">("");
  const [notes, setNotes] = useState(initialAppointment?.notes ?? "");
  const [overlaps, setOverlaps] = useState<OverlapWarning[]>([]);
  const [submitError, setSubmitError] = useState<string | null>(null);

  const effectiveClientId =
    mode === "edit"
      ? initialAppointment?.client_id ?? ""
      : clientId || clients[0]?.id || "";

  const clientPlans = useMemo(
    () =>
      sessionPlans.filter(
        (p) => p.client_id === effectiveClientId && p.status !== "archived"
      ),
    [sessionPlans, effectiveClientId]
  );

  // Live non-blocking overlap check
  useEffect(() => {
    if (!open || !isTauriRuntime() || !startLocal || !endLocal) return;
    if (endLocal <= startLocal) return;

    let cancelled = false;
    const timer = window.setTimeout(async () => {
      try {
        const result = await checkAppointmentOverlaps({
          start_local: startLocal,
          end_local: endLocal,
          dst_disambiguation: dstChoice || null,
          exclude_appointment_id:
            mode === "edit" ? initialAppointment?.id ?? null : null,
        });
        if (!cancelled) {
          setOverlaps(result);
        }
      } catch {
        if (!cancelled) {
          setOverlaps([]);
        }
      }
    }, 250);

    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [open, startLocal, endLocal, dstChoice, mode, initialAppointment?.id]);

  useEffect(() => {
    if (!open) return;
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !isSubmitting) {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [open, isSubmitting, onClose]);

  if (!open) return null;

  const toggleService = (id: string) => {
    setSelectedServiceIds((prev) => {
      const next = prev.includes(id)
        ? prev.filter((item) => item !== id)
        : [...prev, id];
      if (next.length > 0 && startLocal) {
        const totalMins = next.reduce((sum, svcId) => {
          const svc = services.find((s) => s.id === svcId);
          return sum + (svc?.default_duration_minutes ?? 0);
        }, 0);
        if (totalMins >= 5) {
          setEndLocal(addMinutesToLocal(startLocal, totalMins));
        }
      }
      return next;
    });
  };

  const handleStartChange = (nextStart: string) => {
    setStartLocal(nextStart);
    if (selectedServiceIds.length > 0) {
      const totalMins = selectedServiceIds.reduce((sum, svcId) => {
        const svc = services.find((s) => s.id === svcId);
        return sum + (svc?.default_duration_minutes ?? 0);
      }, 0);
      if (totalMins >= 5) {
        setEndLocal(addMinutesToLocal(nextStart, totalMins));
      }
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitError(null);

    if (selectedServiceIds.length === 0) {
      setSubmitError("Please select at least one service.");
      return;
    }
    if (!startLocal || !endLocal) {
      setSubmitError("Start and end times are required.");
      return;
    }
    if (endLocal <= startLocal) {
      setSubmitError("End time must be after start time.");
      return;
    }

    try {
      if (mode === "create" && onCreate) {
        if (clientMode === "inline") {
          if (!inlineName.trim()) {
            setSubmitError("New client name is required.");
            return;
          }
          await onCreate({
            client_id: null,
            inline_client: {
              full_name: inlineName.trim(),
              phone: inlinePhone.trim() || null,
              email: inlineEmail.trim() || null,
              address: null,
              notes: null,
            },
            session_plan_id: null,
            start_local: startLocal,
            end_local: endLocal,
            dst_disambiguation: dstChoice || null,
            service_ids: selectedServiceIds,
            notes: notes.trim() || null,
          });
        } else {
          if (!effectiveClientId) {
            setSubmitError(
              "Please select a client or switch to creating one inline."
            );
            return;
          }
          await onCreate({
            client_id: effectiveClientId,
            inline_client: null,
            session_plan_id: sessionPlanId || null,
            start_local: startLocal,
            end_local: endLocal,
            dst_disambiguation: dstChoice || null,
            service_ids: selectedServiceIds,
            notes: notes.trim() || null,
          });
        }
      } else if (mode === "edit" && onUpdate && initialAppointment) {
        await onUpdate({
          id: initialAppointment.id,
          session_plan_id: sessionPlanId || null,
          start_local: startLocal,
          end_local: endLocal,
          dst_disambiguation: dstChoice || null,
          service_ids: selectedServiceIds,
          notes: notes.trim() || null,
        });
      }
      onClose();
    } catch (err) {
      setSubmitError(extractErrorMessage(err));
    }
  };

  const showDstDisambiguation =
    dstChoice !== "" ||
    (submitError?.toLowerCase().includes("occurs twice") ?? false);

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="appointment-dialog-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-4 overflow-y-auto"
    >
      <div className="w-full max-w-2xl rounded-xl border border-border bg-card text-card-foreground shadow-lg overflow-hidden my-auto">
        <div className="flex items-center justify-between border-b border-border px-6 py-4">
          <div>
            <h2 id="appointment-dialog-title" className="text-lg font-semibold">
              {mode === "create" ? "New Appointment" : "Edit Appointment"}
            </h2>
            <p className="text-xs text-muted-foreground">
              Times use business timezone{" "}
              <span className="font-mono text-foreground">
                {businessTimezone}
              </span>
              .
            </p>
          </div>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            onClick={onClose}
            disabled={isSubmitting}
            aria-label="Close dialog"
          >
            <X className="size-4" />
          </Button>
        </div>

        <form
          onSubmit={handleSubmit}
          className="p-6 space-y-5 max-h-[80vh] overflow-y-auto"
        >
          {mode === "create" ? (
            <div className="space-y-3">
              <div className="flex items-center justify-between gap-2">
                <Label htmlFor="appt-client">
                  Client <span className="text-destructive">*</span>
                </Label>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => {
                    setClientMode((prev) =>
                      prev === "existing" ? "inline" : "existing"
                    );
                    setSessionPlanId("");
                  }}
                >
                  <UserPlus className="size-3.5" />
                  {clientMode === "existing"
                    ? "Create New Client Inline"
                    : "Select Existing Client"}
                </Button>
              </div>

              {clientMode === "existing" ? (
                <select
                  id="appt-client"
                  value={effectiveClientId}
                  onChange={(e) => {
                    setClientId(e.target.value);
                    setSessionPlanId("");
                  }}
                  className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm text-foreground shadow-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  {clients.length === 0 ? (
                    <option value="">
                      No active clients yet — click &ldquo;Create New Client
                      Inline&rdquo;
                    </option>
                  ) : (
                    clients.map((c) => (
                      <option key={c.id} value={c.id}>
                        {c.full_name}
                        {c.phone ? ` (${c.phone})` : ""}
                      </option>
                    ))
                  )}
                </select>
              ) : (
                <div className="rounded-lg border border-border bg-muted/20 p-4 space-y-3">
                  <p className="text-xs font-medium text-muted-foreground">
                    New client will be saved together with this appointment.
                  </p>
                  <div className="space-y-1.5">
                    <Label htmlFor="inline-client-name">
                      Full Name <span className="text-destructive">*</span>
                    </Label>
                    <Input
                      id="inline-client-name"
                      placeholder="e.g. Lucia Gomez"
                      value={inlineName}
                      onChange={(e) => setInlineName(e.target.value)}
                    />
                  </div>
                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                    <div className="space-y-1.5">
                      <Label htmlFor="inline-client-phone">Phone</Label>
                      <Input
                        id="inline-client-phone"
                        placeholder="+34 600 000 000"
                        value={inlinePhone}
                        onChange={(e) => setInlinePhone(e.target.value)}
                      />
                    </div>
                    <div className="space-y-1.5">
                      <Label htmlFor="inline-client-email">Email</Label>
                      <Input
                        id="inline-client-email"
                        type="email"
                        placeholder="client@example.com"
                        value={inlineEmail}
                        onChange={(e) => setInlineEmail(e.target.value)}
                      />
                    </div>
                  </div>
                </div>
              )}
            </div>
          ) : (
            <div className="text-sm">
              <span className="text-muted-foreground">Client:</span>{" "}
              <span className="font-medium">
                {initialAppointment?.client_name}
              </span>
            </div>
          )}

          <div className="space-y-2">
            <Label>
              Services <span className="text-destructive">*</span>
            </Label>
            {services.length === 0 ? (
              <p className="text-xs text-muted-foreground rounded-md border border-dashed border-border p-3">
                No active services found. Add at least one service in the
                Services section first.
              </p>
            ) : (
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                {services.map((svc) => {
                  const checked = selectedServiceIds.includes(svc.id);
                  return (
                    <label
                      key={svc.id}
                      className={`flex items-center justify-between gap-2 rounded-lg border px-3 py-2 text-sm cursor-pointer transition-colors ${
                        checked
                          ? "border-primary bg-accent/60 text-foreground"
                          : "border-border bg-background hover:bg-accent/30"
                      }`}
                    >
                      <span className="flex items-center gap-2 min-w-0">
                        <input
                          type="checkbox"
                          checked={checked}
                          onChange={() => toggleService(svc.id)}
                          className="rounded border-input"
                        />
                        <span className="truncate font-medium">{svc.name}</span>
                      </span>
                      <span className="text-xs text-muted-foreground shrink-0">
                        {formatDuration(svc.default_duration_minutes)}
                      </span>
                    </label>
                  );
                })}
              </div>
            )}
          </div>

          <div className="grid grid-cols-1 gap-4">
            <DateTimeLocalPicker
              id="appt-start"
              label={`Start (${businessTimezone})`}
              value={startLocal}
              onChange={handleStartChange}
              required
            />

            <DateTimeLocalPicker
              id="appt-end"
              label={`End (${businessTimezone})`}
              value={endLocal}
              onChange={setEndLocal}
              required
            />
          </div>

          {showDstDisambiguation && (
            <div className="space-y-1.5 rounded-lg border border-border bg-muted/20 p-3">
              <Label htmlFor="dst-choice">
                Daylight-Saving Occurrence (Optional)
              </Label>
              <select
                id="dst-choice"
                value={dstChoice}
                onChange={(e) =>
                  setDstChoice(e.target.value as DstDisambiguation | "")
                }
                className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm text-foreground"
              >
                <option value="">Not specified</option>
                <option value="earlier">Earlier occurrence (before clock change)</option>
                <option value="later">Later occurrence (after clock change)</option>
              </select>
            </div>
          )}

          {overlaps.length > 0 && (
            <div
              role="status"
              className="flex items-start gap-2.5 rounded-lg border border-amber-500/40 bg-amber-500/10 p-3 text-xs text-foreground"
            >
              <AlertTriangle className="size-4 text-amber-600 dark:text-amber-400 shrink-0 mt-0.5" />
              <div className="space-y-1">
                <p className="font-medium">
                  Overlap warning: this time overlaps {overlaps.length} scheduled{" "}
                  {overlaps.length === 1 ? "appointment" : "appointments"}.
                </p>
                <ul className="list-disc pl-4 space-y-0.5 text-muted-foreground">
                  {overlaps.map((o) => (
                    <li key={o.appointment_id}>
                      {o.client_name} ({o.start_local.replace("T", " ")} –{" "}
                      {o.end_local.slice(11)})
                    </li>
                  ))}
                </ul>
                <p className="text-muted-foreground">
                  You may still save if an overlapping booking is intentional.
                </p>
              </div>
            </div>
          )}

          {clientMode === "existing" && (
            <div className="space-y-1.5">
              <Label htmlFor="appt-plan">
                Session Plan{" "}
                <span className="text-xs font-normal text-muted-foreground">
                  (Optional)
                </span>
              </Label>
              <select
                id="appt-plan"
                value={sessionPlanId}
                onChange={(e) => setSessionPlanId(e.target.value)}
                className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm text-foreground shadow-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                <option value="">Independent appointment (no session plan)</option>
                {clientPlans.map((plan) => (
                  <option key={plan.id} value={plan.id}>
                    {plan.title} ({plan.linked_completed}/{plan.planned_sessions}{" "}
                    completed)
                  </option>
                ))}
              </select>
            </div>
          )}

          <div className="space-y-1.5">
            <Label htmlFor="appt-notes">Notes</Label>
            <textarea
              id="appt-notes"
              rows={2}
              placeholder="Optional appointment notes"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              className="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground shadow-xs placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            />
          </div>

          {submitError && (
            <div
              role="alert"
              className="rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive"
            >
              {submitError}
            </div>
          )}

          <div className="flex items-center justify-end gap-2 pt-2 border-t border-border">
            <Button
              type="button"
              variant="outline"
              onClick={onClose}
              disabled={isSubmitting}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting
                ? "Saving..."
                : mode === "create"
                  ? "Schedule Appointment"
                  : "Save Changes"}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
