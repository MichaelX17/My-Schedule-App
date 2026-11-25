import { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { extractErrorMessage } from "@/services/business";
import type { Client } from "@/services/clients";
import {
  clientFormSchema,
  normalizeClientFormInput,
  type ClientFormValues,
} from "../validation";

interface ClientFormDialogProps {
  open: boolean;
  mode: "create" | "edit";
  initialClient?: Client | null;
  isSubmitting?: boolean;
  onClose: () => void;
  onSubmit: (values: ReturnType<typeof normalizeClientFormInput>) => Promise<void>;
}

export function ClientFormDialog({
  open,
  mode,
  initialClient,
  isSubmitting = false,
  onClose,
  onSubmit,
}: ClientFormDialogProps) {
  const [submitError, setSubmitError] = useState<string | null>(null);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<ClientFormValues>({
    resolver: zodResolver(clientFormSchema),
    defaultValues: {
      full_name: initialClient?.full_name ?? "",
      phone: initialClient?.phone ?? "",
      email: initialClient?.email ?? "",
      address: initialClient?.address ?? "",
      notes: initialClient?.notes ?? "",
    },
  });

  useEffect(() => {
    if (open) {
      reset({
        full_name: initialClient?.full_name ?? "",
        phone: initialClient?.phone ?? "",
        email: initialClient?.email ?? "",
        address: initialClient?.address ?? "",
        notes: initialClient?.notes ?? "",
      });
    }
  }, [open, initialClient, reset]);

  const handleClose = () => {
    setSubmitError(null);
    onClose();
  };

  useEffect(() => {
    if (!open) return;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !isSubmitting) {
        setSubmitError(null);
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [open, isSubmitting, onClose]);

  if (!open) return null;

  const handleFormSubmit = async (values: ClientFormValues) => {
    setSubmitError(null);
    try {
      await onSubmit(normalizeClientFormInput(values));
      handleClose();
    } catch (err) {
      setSubmitError(extractErrorMessage(err));
    }
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="client-dialog-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-4"
    >
      <div className="w-full max-w-lg rounded-xl border border-border bg-card text-card-foreground shadow-lg overflow-hidden">
        <div className="flex items-center justify-between border-b border-border px-6 py-4">
          <h2 id="client-dialog-title" className="text-lg font-semibold">
            {mode === "create" ? "New Client" : "Edit Client"}
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

        <form onSubmit={handleSubmit(handleFormSubmit)} className="p-6 space-y-4">
          <div className="space-y-1.5">
            <Label htmlFor="client-full-name">
              Full Name <span className="text-destructive">*</span>
            </Label>
            <Input
              id="client-full-name"
              placeholder="e.g. Maria Garcia"
              autoFocus
              aria-invalid={Boolean(errors.full_name)}
              {...register("full_name")}
            />
            {errors.full_name && (
              <p className="text-xs text-destructive" role="alert">
                {errors.full_name.message}
              </p>
            )}
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <Label htmlFor="client-phone">Phone</Label>
              <Input
                id="client-phone"
                placeholder="+34 600 000 000"
                {...register("phone")}
              />
              {errors.phone && (
                <p className="text-xs text-destructive" role="alert">
                  {errors.phone.message}
                </p>
              )}
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="client-email">Email</Label>
              <Input
                id="client-email"
                type="email"
                placeholder="client@example.com"
                aria-invalid={Boolean(errors.email)}
                {...register("email")}
              />
              {errors.email && (
                <p className="text-xs text-destructive" role="alert">
                  {errors.email.message}
                </p>
              )}
            </div>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="client-address">Address</Label>
            <Input
              id="client-address"
              placeholder="Optional street address or location"
              {...register("address")}
            />
            {errors.address && (
              <p className="text-xs text-destructive" role="alert">
                {errors.address.message}
              </p>
            )}
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="client-notes">General Notes</Label>
            <textarea
              id="client-notes"
              rows={3}
              placeholder="Optional general notes (scheduling preferences, etc.)"
              className="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground shadow-xs placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
              {...register("notes")}
            />
            {errors.notes && (
              <p className="text-xs text-destructive" role="alert">
                {errors.notes.message}
              </p>
            )}
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
              onClick={handleClose}
              disabled={isSubmitting}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting
                ? "Saving..."
                : mode === "create"
                  ? "Create Client"
                  : "Save Changes"}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
