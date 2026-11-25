import { describe, expect, it } from "vitest";
import {
  formatDuration,
  normalizeServiceFormInput,
  serviceFormSchema,
} from "../validation";

describe("serviceFormSchema", () => {
  it("requires a non-blank name and valid duration range", () => {
    expect(
      serviceFormSchema.safeParse({ name: "  ", default_duration_minutes: 30 })
        .success
    ).toBe(false);

    expect(
      serviceFormSchema.safeParse({
        name: "Short",
        default_duration_minutes: 4,
      }).success
    ).toBe(false);

    expect(
      serviceFormSchema.safeParse({
        name: "Too Long",
        default_duration_minutes: 1441,
      }).success
    ).toBe(false);
  });

  it("normalizes valid service input and formats durations clearly", () => {
    const parsed = serviceFormSchema.safeParse({
      name: "  Strategy Session  ",
      description: "   ",
      default_duration_minutes: 90,
    });
    expect(parsed.success).toBe(true);
    if (parsed.success) {
      expect(normalizeServiceFormInput(parsed.data)).toEqual({
        name: "Strategy Session",
        description: null,
        default_duration_minutes: 90,
      });
    }

    expect(formatDuration(45)).toBe("45 min");
    expect(formatDuration(60)).toBe("1h");
    expect(formatDuration(90)).toBe("1h 30m");
  });
});
