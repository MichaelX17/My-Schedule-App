import { useEffect, useState } from "react";
import { useForm, useWatch } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { extractErrorMessage } from "@/services/business";
import type { Service } from "@/services/serviceCatalog";
import {
  MAX_DURATION_MINUTES,
  MIN_DURATION_MINUTES,
  normalizeServiceFormInput,
  serviceFormSchema,
  type ServiceFormValues,
} from "../validation";

const DURATION_PRESETS = [15, 30, 45, 60, 90, 120] as const;

interface ServiceFormDialogProps {
  open: boolean;
  mode: "create" | "edit";
  initialService?: Service | null;
  isSubmitting?: boolean;
  onClose: () => void;
  onSubmit: (
    values: ReturnType<typeof normalizeServiceFormInput>
  ) => Promise<void>;
}

export function ServiceFormDialog({
  open,
  mode,
  initialService,
  isSubmitting = false,
  onClose,
  onSubmit,
}: ServiceFormDialogProps) {
  const [submitError, setSubmitError] = useState<string | null>(null);

  const {
    control,
    register,
    handleSubmit,
    reset,
    setValue,
    formState: { errors },
  } = useForm<ServiceFormValues>({
    resolver: zodResolver(serviceFormSchema),
    defaultValues: {
      name: initialService?.name ?? "",
      description: initialService?.description ?? "",
      default_duration_minutes: initialService?.default_duration_minutes ?? 60,
    },
  });

  const currentDuration = useWatch({
    control,
    name: "default_duration_minutes",
  });

  useEffect(() => {
    if (open) {
      reset({
        name: initialService?.name ?? "",
        description: initialService?.description ?? "",
        default_duration_minutes: initialService?.default_duration_minutes ?? 60,
      });
    }
  }, [open, initialService, reset]);

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

  const handleFormSubmit = async (values: ServiceFormValues) => {
    setSubmitError(null);
    try {
      await onSubmit(normalizeServiceFormInput(values));
      handleClose();
    } catch (err) {
      setSubmitError(extractErrorMessage(err));
    }
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="service-dialog-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-4"
    >
      <div className="w-full max-w-lg rounded-xl border border-border bg-card text-card-foreground shadow-lg overflow-hidden">
        <div className="flex items-center justify-between border-b border-border px-6 py-4">
          <h2 id="service-dialog-title" className="text-lg font-semibold">
            {mode === "create" ? "New Service" : "Edit Service"}
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
            <Label htmlFor="service-name">
              Service Name <span className="text-destructive">*</span>
            </Label>
            <Input
              id="service-name"
              placeholder="e.g. Initial Consultation"
              autoFocus
              aria-invalid={Boolean(errors.name)}
              {...register("name")}
            />
            {errors.name && (
              <p className="text-xs text-destructive" role="alert">
                {errors.name.message}
              </p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="service-duration">
              Default Duration (minutes){" "}
              <span className="text-destructive">*</span>
            </Label>
            <div className="flex flex-wrap gap-1.5">
              {DURATION_PRESETS.map((preset) => (
                <button
                  key={preset}
                  type="button"
                  onClick={() =>
                    setValue("default_duration_minutes", preset, {
                      shouldValidate: true,
                    })
                  }
                  className={`rounded-md border px-2.5 py-1 text-xs font-medium transition-colors ${
                    currentDuration === preset
                      ? "border-primary bg-primary text-primary-foreground"
                      : "border-border bg-background text-muted-foreground hover:text-foreground"
                  }`}
                >
                  {preset} min
                </button>
              ))}
            </div>
            <Input
              id="service-duration"
              type="number"
              min={MIN_DURATION_MINUTES}
              max={MAX_DURATION_MINUTES}
              step={5}
              aria-invalid={Boolean(errors.default_duration_minutes)}
              {...register("default_duration_minutes", { valueAsNumber: true })}
            />
            {errors.default_duration_minutes && (
              <p className="text-xs text-destructive" role="alert">
                {errors.default_duration_minutes.message}
              </p>
            )}
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="service-description">Description</Label>
            <textarea
              id="service-description"
              rows={3}
              placeholder="Optional description of what this service includes"
              className="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground shadow-xs placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
              {...register("description")}
            />
            {errors.description && (
              <p className="text-xs text-destructive" role="alert">
                {errors.description.message}
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
                  ? "Create Service"
                  : "Save Changes"}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
