import { useEffect, useMemo, useState } from "react";
import { CalendarPlus, X } from "lucide-react";
import { DateTimeLocalPicker } from "@/components/shared/DateTimeLocalPicker";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { formatDuration } from "@/features/services/validation";
import type { CreateSessionPlanWithAppointmentsInput } from "@/services/appointments";
import { extractErrorMessage } from "@/services/business";
import type { Client } from "@/services/clients";
import type { Service } from "@/services/serviceCatalog";

interface SessionSlot {
  start_local: string;
  end_local: string;
}

interface SessionPlanFormDialogProps {
  open: boolean;
  clients: Client[];
  services: Service[];
  businessTimezone: string;
  defaultClientId?: string;
  isSubmitting?: boolean;
  onClose: () => void;
  onSubmit: (values: CreateSessionPlanWithAppointmentsInput) => Promise<void>;
}

function formatLocalDateTime(date: Date): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

function parseLocalDateTime(value: string): Date | null {
  const match = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})$/.exec(value);
  if (!match) return null;
  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  const hour = Number(match[4]);
  const minute = Number(match[5]);
  return new Date(year, month - 1, day, hour, minute, 0, 0);
}

function addMinutesLocal(startLocal: string, minutes: number): string {
  const parsed = parseLocalDateTime(startLocal);
  if (!parsed) return startLocal;
  return formatLocalDateTime(new Date(parsed.getTime() + minutes * 60_000));
}

function addDaysLocal(localStr: string, days: number): string {
  const parsed = parseLocalDateTime(localStr);
  if (!parsed) return localStr;
  const shifted = new Date(parsed.getTime());
  shifted.setDate(shifted.getDate() + days);
  return formatLocalDateTime(shifted);
}

function buildDefaultSlots(count: number, durationMinutes: number): SessionSlot[] {
  const base = new Date();
  base.setMinutes(0, 0, 0);
  base.setHours(base.getHours() + 1);
  const firstStart = formatLocalDateTime(base);

  return Array.from({ length: count }, (_, idx) => {
    const start = addDaysLocal(firstStart, idx * 7);
    return {
      start_local: start,
      end_local: addMinutesLocal(start, durationMinutes),
    };
  });
}

export function SessionPlanFormDialog({
  open,
  clients,
  services,
  businessTimezone,
  defaultClientId,
  isSubmitting = false,
  onClose,
  onSubmit,
}: SessionPlanFormDialogProps) {
  const [clientId, setClientId] = useState(defaultClientId ?? "");
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [plannedSessions, setPlannedSessions] = useState(4);
  const [selectedServiceIds, setSelectedServiceIds] = useState<string[]>(() =>
    services[0]?.id ? [services[0].id] : []
  );
  const [sessions, setSessions] = useState<SessionSlot[]>(() =>
    buildDefaultSlots(4, services[0]?.default_duration_minutes ?? 60)
  );
  const [error, setError] = useState<string | null>(null);

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

  const effectiveServiceIds = useMemo(() => {
    if (selectedServiceIds.length > 0) return selectedServiceIds;
    return services[0]?.id ? [services[0].id] : [];
  }, [selectedServiceIds, services]);

  const totalDurationMinutes = useMemo(() => {
    const sum = services
      .filter((s) => effectiveServiceIds.includes(s.id))
      .reduce((acc, s) => acc + s.default_duration_minutes, 0);
    return sum > 0 ? sum : 60;
  }, [services, effectiveServiceIds]);

  if (!open) return null;

  const effectiveClientId = clientId || defaultClientId || clients[0]?.id || "";

  const handleClose = () => {
    setError(null);
    setTitle("");
    setDescription("");
    setPlannedSessions(4);
    const defaultDuration = services[0]?.default_duration_minutes ?? 60;
    setSelectedServiceIds(services[0]?.id ? [services[0].id] : []);
    setSessions(buildDefaultSlots(4, defaultDuration));
    onClose();
  };

  const handlePlannedSessionsChange = (raw: string) => {
    const parsed = Math.max(1, Math.min(500, Number(raw) || 1));
    setPlannedSessions(parsed);
    setSessions((prev) => {
      if (prev.length === parsed) return prev;
      if (prev.length > parsed) return prev.slice(0, parsed);
      const next = [...prev];
      const lastStart =
        next[next.length - 1]?.start_local ??
        buildDefaultSlots(1, totalDurationMinutes)[0]?.start_local ??
        "";
      for (let i = prev.length; i < parsed; i++) {
        const start = addDaysLocal(lastStart, (i - prev.length + 1) * 7);
        next.push({
          start_local: start,
          end_local: addMinutesLocal(start, totalDurationMinutes),
        });
      }
      return next;
    });
  };

  const toggleService = (serviceId: string) => {
    const nextIds = effectiveServiceIds.includes(serviceId)
      ? effectiveServiceIds.filter((id) => id !== serviceId)
      : [...effectiveServiceIds, serviceId];
    setSelectedServiceIds(nextIds);

    const nextDuration = services
      .filter((s) => nextIds.includes(s.id))
      .reduce((acc, s) => acc + s.default_duration_minutes, 0);
    if (nextDuration > 0) {
      setSessions((prev) =>
        prev.map((slot) => ({
          ...slot,
          end_local: addMinutesLocal(slot.start_local, nextDuration),
        }))
      );
    }
  };

  const handleSlotStartChange = (index: number, nextStart: string) => {
    setSessions((prev) =>
      prev.map((slot, i) =>
        i === index
          ? {
              start_local: nextStart,
              end_local: addMinutesLocal(nextStart, totalDurationMinutes),
            }
          : slot
      )
    );
  };

  const handleSlotEndChange = (index: number, nextEnd: string) => {
    setSessions((prev) =>
      prev.map((slot, i) =>
        i === index ? { ...slot, end_local: nextEnd } : slot
      )
    );
  };

  const handleAutoSpaceWeekly = () => {
    const first = sessions[0];
    if (!first?.start_local) return;
    setSessions((prev) =>
      prev.map((_, idx) => {
        const start = addDaysLocal(first.start_local, idx * 7);
        return {
          start_local: start,
          end_local: addMinutesLocal(start, totalDurationMinutes),
        };
      })
    );
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    if (!effectiveClientId) {
      setError("Please select a client.");
      return;
    }
    if (!title.trim()) {
      setError("Session plan title is required.");
      return;
    }
    if (plannedSessions < 1 || plannedSessions > 500) {
      setError("Planned sessions must be between 1 and 500.");
      return;
    }
    if (effectiveServiceIds.length === 0) {
      setError("Please select at least one service for the planned sessions.");
      return;
    }
    for (let i = 0; i < sessions.length; i++) {
      const slot = sessions[i];
      if (!slot?.start_local || !slot?.end_local) {
        setError(`Session ${i + 1}: Please set both start and end date/time.`);
        return;
      }
    }

    try {
      await onSubmit({
        client_id: effectiveClientId,
        title: title.trim(),
        description: description.trim() || null,
        planned_sessions: plannedSessions,
        service_ids: effectiveServiceIds,
        sessions: sessions.map((s) => ({
          start_local: s.start_local,
          end_local: s.end_local,
        })),
      });
      handleClose();
    } catch (err) {
      setError(extractErrorMessage(err));
    }
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="plan-dialog-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-4"
    >
      <div className="w-full max-w-2xl max-h-[90vh] flex flex-col rounded-xl border border-border bg-card text-card-foreground shadow-lg overflow-hidden">
        <div className="flex items-center justify-between border-b border-border px-6 py-4 shrink-0">
          <div>
            <h2 id="plan-dialog-title" className="text-lg font-semibold">
              New Session Plan
            </h2>
            <p className="text-xs text-muted-foreground mt-0.5">
              Creates the session plan together with an individual scheduled
              appointment for each session.
            </p>
          </div>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            onClick={handleClose}
            disabled={isSubmitting}
            aria-label="Close dialog"
          >
            <X className="size-4" />
          </Button>
        </div>

        <form
          onSubmit={handleSubmit}
          className="flex-1 overflow-y-auto p-6 space-y-5"
        >
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <Label htmlFor="plan-client">
                Client <span className="text-destructive">*</span>
              </Label>
              <select
                id="plan-client"
                value={effectiveClientId}
                onChange={(e) => setClientId(e.target.value)}
                className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm text-foreground shadow-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                {clients.length === 0 ? (
                  <option value="">No active clients available</option>
                ) : (
                  clients.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.full_name}
                    </option>
                  ))
                )}
              </select>
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="plan-sessions">
                Planned Sessions <span className="text-destructive">*</span>
              </Label>
              <Input
                id="plan-sessions"
                type="number"
                min={1}
                max={500}
                value={plannedSessions}
                onChange={(e) => handlePlannedSessionsChange(e.target.value)}
              />
            </div>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="plan-title">
              Plan Title <span className="text-destructive">*</span>
            </Label>
            <Input
              id="plan-title"
              placeholder="e.g. 8-Session Coaching Series"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
            />
          </div>

          <div className="space-y-2">
            <div className="flex items-center justify-between gap-2">
              <Label>
                Services for Each Session{" "}
                <span className="text-destructive">*</span>
              </Label>
              <span className="text-xs text-muted-foreground">
                Default duration: {formatDuration(totalDurationMinutes)}
              </span>
            </div>
            {services.length === 0 ? (
              <p className="text-xs text-muted-foreground rounded-md border border-border bg-muted/30 p-3">
                No active services available. Create a service first.
              </p>
            ) : (
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 max-h-32 overflow-y-auto rounded-lg border border-border bg-muted/20 p-2.5">
                {services.map((svc) => {
                  const checked = effectiveServiceIds.includes(svc.id);
                  return (
                    <label
                      key={svc.id}
                      className={`flex items-center justify-between gap-2 rounded-md border px-2.5 py-2 text-xs cursor-pointer transition-colors ${
                        checked
                          ? "border-primary bg-accent/60 text-foreground"
                          : "border-transparent hover:bg-accent/30 text-foreground"
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
                      <span className="text-muted-foreground shrink-0">
                        {formatDuration(svc.default_duration_minutes)}
                      </span>
                    </label>
                  );
                })}
              </div>
            )}
          </div>

          <div className="space-y-3">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div>
                <h3 className="text-sm font-medium">
                  Session Appointments ({sessions.length})
                </h3>
                <p className="text-xs text-muted-foreground">
                  Timezone: {businessTimezone}. Set date, hour, and minute for
                  each session.
                </p>
              </div>
              {sessions.length > 1 && (
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={handleAutoSpaceWeekly}
                >
                  <CalendarPlus className="size-3.5" />
                  Auto-space weekly from Session 1
                </Button>
              )}
            </div>

            <div className="space-y-3 max-h-72 overflow-y-auto pr-1">
              {sessions.map((slot, idx) => (
                <div
                  key={idx}
                  className="rounded-lg border border-border bg-muted/20 p-3 space-y-2.5"
                >
                  <div className="text-xs font-semibold text-foreground">
                    Session {idx + 1} of {sessions.length}
                  </div>
                  <div className="grid grid-cols-1 gap-2.5">
                    <DateTimeLocalPicker
                      id={`plan-session-${idx + 1}-start`}
                      label={`Session ${idx + 1} Start`}
                      value={slot.start_local}
                      onChange={(val) => handleSlotStartChange(idx, val)}
                      required
                    />
                    <DateTimeLocalPicker
                      id={`plan-session-${idx + 1}-end`}
                      label={`Session ${idx + 1} End`}
                      value={slot.end_local}
                      onChange={(val) => handleSlotEndChange(idx, val)}
                      required
                    />
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="plan-description">Description</Label>
            <textarea
              id="plan-description"
              rows={2}
              placeholder="Optional overview of this session plan"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground shadow-xs placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            />
          </div>

          {error && (
            <div
              role="alert"
              className="rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive"
            >
              {error}
            </div>
          )}

          <div className="flex items-center justify-end gap-2 pt-3 border-t border-border">
            <Button
              type="button"
              variant="outline"
              onClick={handleClose}
              disabled={isSubmitting}
            >
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={
                isSubmitting || clients.length === 0 || services.length === 0
              }
            >
              {isSubmitting
                ? "Creating Plan & Appointments..."
                : `Create Plan & ${sessions.length} ${sessions.length === 1 ? "Appointment" : "Appointments"}`}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
