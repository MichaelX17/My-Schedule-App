import { z } from "zod";

export const clientFormSchema = z.object({
  full_name: z
    .string()
    .trim()
    .min(1, "Client name is required.")
    .max(150, "Client name must be 150 characters or fewer."),
  phone: z
    .string()
    .max(50, "Phone must be 50 characters or fewer.")
    .optional(),
  email: z
    .string()
    .max(255, "Email must be 255 characters or fewer.")
    .optional()
    .refine(
      (val) => {
        if (!val || val.trim().length === 0) return true;
        return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(val.trim());
      },
      { message: "Please enter a valid email address or leave it blank." }
    ),
  address: z
    .string()
    .max(255, "Address must be 255 characters or fewer.")
    .optional(),
  notes: z
    .string()
    .max(4000, "Notes must be 4000 characters or fewer.")
    .optional(),
});

export type ClientFormValues = z.infer<typeof clientFormSchema>;

export function normalizeClientFormInput(values: ClientFormValues) {
  const clean = (value?: string) => {
    const trimmed = value?.trim();
    return trimmed && trimmed.length > 0 ? trimmed : null;
  };
  return {
    full_name: values.full_name.trim(),
    phone: clean(values.phone),
    email: clean(values.email),
    address: clean(values.address),
    notes: clean(values.notes),
  };
}
