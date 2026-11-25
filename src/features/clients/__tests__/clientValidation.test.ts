import { describe, expect, it } from "vitest";
import {
  clientFormSchema,
  normalizeClientFormInput,
} from "../validation";

describe("clientFormSchema", () => {
  it("requires a non-blank client name", () => {
    const result = clientFormSchema.safeParse({ full_name: "   " });
    expect(result.success).toBe(false);
  });

  it("accepts international phone formats and optional empty fields", () => {
    const result = clientFormSchema.safeParse({
      full_name: "  Elena Morales  ",
      phone: "+34 600 123 456",
      email: "",
      address: "   ",
      notes: "Prefers afternoon slots.",
    });

    expect(result.success).toBe(true);
    if (result.success) {
      expect(normalizeClientFormInput(result.data)).toEqual({
        full_name: "Elena Morales",
        phone: "+34 600 123 456",
        email: null,
        address: null,
        notes: "Prefers afternoon slots.",
      });
    }
  });

  it("rejects malformed email addresses when provided", () => {
    const result = clientFormSchema.safeParse({
      full_name: "Elena Morales",
      email: "not-an-email",
    });
    expect(result.success).toBe(false);
  });
});
