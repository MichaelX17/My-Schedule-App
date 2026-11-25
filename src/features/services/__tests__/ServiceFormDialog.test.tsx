import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ServiceFormDialog } from "../components/ServiceFormDialog";

describe("ServiceFormDialog", () => {
  it("validates required name before submitting", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    const onClose = vi.fn();

    render(
      <ServiceFormDialog
        open
        mode="create"
        onClose={onClose}
        onSubmit={onSubmit}
      />
    );

    await user.click(screen.getByRole("button", { name: "Create Service" }));
    expect(
      await screen.findByText("Service name is required.")
    ).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("applies duration presets and submits normalized service data", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    const onClose = vi.fn();

    render(
      <ServiceFormDialog
        open
        mode="create"
        onClose={onClose}
        onSubmit={onSubmit}
      />
    );

    await user.type(screen.getByLabelText(/Service Name/i), "  Follow-Up  ");
    await user.click(screen.getByRole("button", { name: "45 min" }));
    await user.type(
      screen.getByLabelText(/Description/i),
      "Standard check-in"
    );
    await user.click(screen.getByRole("button", { name: "Create Service" }));

    expect(onSubmit).toHaveBeenCalledWith({
      name: "Follow-Up",
      description: "Standard check-in",
      default_duration_minutes: 45,
    });
    expect(onClose).toHaveBeenCalled();
  });
});
