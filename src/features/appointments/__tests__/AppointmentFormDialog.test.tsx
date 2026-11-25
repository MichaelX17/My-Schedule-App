import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Service } from "@/services/serviceCatalog";
import { AppointmentFormDialog } from "../components/AppointmentFormDialog";

const MOCK_SERVICES: Service[] = [
  {
    id: "svc-1",
    name: "Consultation",
    description: null,
    default_duration_minutes: 60,
    is_active: true,
    is_archived: false,
    archived_at: null,
    created_at: "2026-11-24T00:00:00Z",
    updated_at: "2026-11-24T00:00:00Z",
  },
  {
    id: "svc-2",
    name: "Follow-Up",
    description: null,
    default_duration_minutes: 30,
    is_active: true,
    is_archived: false,
    archived_at: null,
    created_at: "2026-11-24T00:00:00Z",
    updated_at: "2026-11-24T00:00:00Z",
  },
];

describe("AppointmentFormDialog", () => {
  it("supports inline client creation and multi-service selection without losing state", async () => {
    const user = userEvent.setup();
    const onCreate = vi.fn().mockResolvedValue(undefined);
    const onClose = vi.fn();

    render(
      <AppointmentFormDialog
        open
        mode="create"
        clients={[]}
        services={MOCK_SERVICES}
        sessionPlans={[]}
        businessTimezone="Europe/Madrid"
        onClose={onClose}
        onCreate={onCreate}
      />
    );

    // Select both services first
    await user.click(screen.getByLabelText(/Consultation/i));
    await user.click(screen.getByLabelText(/Follow-Up/i));

    // Switch to inline client mode without losing selected services
    await user.click(
      screen.getByRole("button", { name: /Create New Client Inline/i })
    );
    await user.type(screen.getByLabelText(/Full Name/i), "Lucia Gomez");
    await user.type(screen.getByLabelText(/Notes/i), "First visit");

    await user.click(
      screen.getByRole("button", { name: "Schedule Appointment" })
    );

    expect(onCreate).toHaveBeenCalledTimes(1);
    const payload = onCreate.mock.calls[0]?.[0];
    expect(payload.client_id).toBeNull();
    expect(payload.inline_client.full_name).toBe("Lucia Gomez");
    expect(payload.service_ids).toEqual(["svc-1", "svc-2"]);
    expect(payload.notes).toBe("First visit");
    expect(onClose).toHaveBeenCalled();
  });

  it("requires at least one service to be selected", async () => {
    const user = userEvent.setup();
    const onCreate = vi.fn().mockResolvedValue(undefined);

    render(
      <AppointmentFormDialog
        open
        mode="create"
        clients={[]}
        services={MOCK_SERVICES}
        sessionPlans={[]}
        businessTimezone="UTC"
        onClose={vi.fn()}
        onCreate={onCreate}
      />
    );

    await user.click(
      screen.getByRole("button", { name: "Schedule Appointment" })
    );

    expect(
      await screen.findByText("Please select at least one service.")
    ).toBeInTheDocument();
    expect(onCreate).not.toHaveBeenCalled();
  });
});
