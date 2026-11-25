import { useEffect, useState } from "react";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { extractErrorMessage } from "@/services/business";
import type { Client } from "@/services/clients";

interface SessionPlanFormDialogProps {
  open: boolean;
  clients: Client[];
  defaultClientId?: string;
  isSubmitting?: boolean;
  onClose: () => void;
  onSubmit: (values: {
    client_id: string;
    title: string;
    description: string | null;
    planned_sessions: number;
  }) => Promise<void>;
}

export function SessionPlanFormDialog({
  open,
  clients,
  defaultClientId,
  isSubmitting = false,
  onClose,
  onSubmit,
}: SessionPlanFormDialogProps) {
  const [clientId, setClientId] = useState(defaultClientId ?? "");
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [plannedSessions, setPlannedSessions] = useState(4);
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

  if (!open) return null;

  const effectiveClientId = clientId || defaultClientId || clients[0]?.id || "";

  const handleClose = () => {
    setError(null);
    setTitle("");
    setDescription("");
    setPlannedSessions(4);
    onClose();
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

    try {
      await onSubmit({
        client_id: effectiveClientId,
        title: title.trim(),
        description: description.trim() || null,
        planned_sessions: plannedSessions,
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
      <div className="w-full max-w-md rounded-xl border border-border bg-card text-card-foreground shadow-lg overflow-hidden">
        <div className="flex items-center justify-between border-b border-border px-6 py-4">
          <h2 id="plan-dialog-title" className="text-lg font-semibold">
            New Session Plan
          </h2>
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

        <form onSubmit={handleSubmit} className="p-6 space-y-4">
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
            <Label htmlFor="plan-title">
              Plan Title <span className="text-destructive">*</span>
            </Label>
            <Input
              id="plan-title"
              placeholder="e.g. 6-Session Onboarding Series"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
            />
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
              onChange={(e) => setPlannedSessions(Number(e.target.value) || 1)}
            />
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

          <div className="flex items-center justify-end gap-2 pt-2 border-t border-border">
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
              disabled={isSubmitting || clients.length === 0}
            >
              {isSubmitting ? "Creating..." : "Create Plan"}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
