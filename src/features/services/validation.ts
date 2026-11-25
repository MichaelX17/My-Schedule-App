import { z } from "zod";

export const MIN_DURATION_MINUTES = 5;
export const MAX_DURATION_MINUTES = 1440;

export const serviceFormSchema = z.object({
  name: z
    .string()
    .trim()
    .min(1, "Service name is required.")
    .max(120, "Service name must be 120 characters or fewer."),
  description: z
    .string()
    .max(1000, "Description must be 1000 characters or fewer.")
    .optional(),
  default_duration_minutes: z
    .number({ message: "Duration must be a whole number of minutes." })
    .int("Duration must be a whole number of minutes.")
    .min(
      MIN_DURATION_MINUTES,
      `Default duration must be at least ${MIN_DURATION_MINUTES} minutes.`
    )
    .max(
      MAX_DURATION_MINUTES,
      `Default duration must be ${MAX_DURATION_MINUTES} minutes or fewer.`
    ),
});

export type ServiceFormValues = z.infer<typeof serviceFormSchema>;

export function normalizeServiceFormInput(values: ServiceFormValues) {
  const trimmedDesc = values.description?.trim();
  return {
    name: values.name.trim(),
    description: trimmedDesc && trimmedDesc.length > 0 ? trimmedDesc : null,
    default_duration_minutes: values.default_duration_minutes,
  };
}

export function formatDuration(minutes: number): string {
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  if (hours === 0) return `${mins} min`;
  if (mins === 0) return `${hours}h`;
  return `${hours}h ${mins}m`;
}
