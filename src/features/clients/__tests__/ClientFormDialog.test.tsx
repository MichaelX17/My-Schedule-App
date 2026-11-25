import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ClientFormDialog } from "../components/ClientFormDialog";

describe("ClientFormDialog", () => {
  it("shows validation error when submitted with an empty name", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    const onClose = vi.fn();

    render(
      <ClientFormDialog
        open
        mode="create"
        onClose={onClose}
        onSubmit={onSubmit}
      />
    );

    await user.click(screen.getByRole("button", { name: "Create Client" }));

    expect(await screen.findByText("Client name is required.")).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("submits normalized values and closes on success", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    const onClose = vi.fn();

    render(
      <ClientFormDialog
        open
        mode="create"
        onClose={onClose}
        onSubmit={onSubmit}
      />
    );

    await user.type(screen.getByLabelText(/Full Name/i), "  Sofia Navarro  ");
    await user.type(screen.getByLabelText(/Phone/i), "+44 20 7946 0958");
    await user.type(screen.getByLabelText(/Email/i), "sofia@example.co.uk");
    await user.click(screen.getByRole("button", { name: "Create Client" }));

    expect(onSubmit).toHaveBeenCalledWith({
      full_name: "Sofia Navarro",
      phone: "+44 20 7946 0958",
      email: "sofia@example.co.uk",
      address: null,
      notes: null,
    });
    expect(onClose).toHaveBeenCalled();
  });

  it("surfaces backend errors clearly without closing the dialog", async () => {
    const user = userEvent.setup();
    const onSubmit = vi
      .fn()
      .mockRejectedValue({ code: "VALIDATION_ERROR", message: "Backend rejected input." });
    const onClose = vi.fn();

    render(
      <ClientFormDialog
        open
        mode="create"
        onClose={onClose}
        onSubmit={onSubmit}
      />
    );

    await user.type(screen.getByLabelText(/Full Name/i), "Sofia Navarro");
    await user.click(screen.getByRole("button", { name: "Create Client" }));

    expect(await screen.findByText("Backend rejected input.")).toBeInTheDocument();
    expect(onClose).not.toHaveBeenCalled();
  });
});
