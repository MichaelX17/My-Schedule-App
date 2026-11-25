import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type {
  BackupMetadata,
  BackupType,
  ExportResult,
  RestoreResult,
} from "@/services/backups";
import { BackupsExportsPage } from "../BackupsExportsPage";

const mockListBackups = vi.fn<() => Promise<BackupMetadata[]>>();
const mockCreateManualBackup = vi.fn<
  (notes?: string | null) => Promise<BackupMetadata>
>();
const mockRestoreBackup = vi.fn<
  (backupType: BackupType, filename: string) => Promise<RestoreResult>
>();
const mockExportJson = vi.fn<
  (customFilename?: string | null) => Promise<ExportResult>
>();

vi.mock("@/services/tauri", () => ({
  isTauriRuntime: () => true,
  invokeCommand: vi.fn(),
}));

vi.mock("@/services/backups", async () => {
  const actual = await vi.importActual<typeof import("@/services/backups")>(
    "@/services/backups"
  );
  return {
    ...actual,
    listBackups: () => mockListBackups(),
    createManualBackup: (notes?: string | null) =>
      mockCreateManualBackup(notes),
    restoreBackup: (backupType: BackupType, filename: string) =>
      mockRestoreBackup(backupType, filename),
    exportBusinessJson: (customFilename?: string | null) =>
      mockExportJson(customFilename),
  };
});

const sampleBackup: BackupMetadata = {
  id: "manual_20261124_220000_000_abcd1234",
  filename: "manual_20261124_220000_000_abcd1234.sqlite",
  backup_type: "manual",
  app_version: "0.1.0",
  schema_version: 6,
  created_at_utc: "2026-11-24T22:00:00.000Z",
  size_bytes: 32768,
  integrity_status: "ok",
  notes: "Manual operator backup",
};

function renderPage() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <BackupsExportsPage />
    </QueryClientProvider>
  );
}

describe("BackupsExportsPage", () => {
  beforeEach(() => {
    mockListBackups.mockReset();
    mockCreateManualBackup.mockReset();
    mockRestoreBackup.mockReset();
    mockExportJson.mockReset();

    mockListBackups.mockResolvedValue([sampleBackup]);
    mockExportJson.mockResolvedValue({
      format: "json",
      filename: "my_schedule_export_20261124.json",
      file_path: "/data/exports/my_schedule_export_20261124.json",
      size_bytes: 4096,
      record_count: 12,
      exported_at_utc: "2026-11-24T22:05:00.000Z",
    });
    mockRestoreBackup.mockResolvedValue({
      restored_filename: sampleBackup.filename,
      restored_schema_version: 6,
      safety_backup: {
        ...sampleBackup,
        id: "safety_1",
        filename: "safety_1.sqlite",
        backup_type: "safety",
      },
    });
  });

  it("exports JSON with visible destination and requires explicit confirmation before restore", async () => {
    const user = userEvent.setup();
    renderPage();

    expect(
      await screen.findByText(sampleBackup.filename)
    ).toBeInTheDocument();

    // Trigger JSON export and verify destination path feedback
    await user.click(
      screen.getByRole("button", { name: /Export Full JSON \(v1\)/i })
    );
    expect(
      await screen.findByText("/data/exports/my_schedule_export_20261124.json")
    ).toBeInTheDocument();

    // Click Restore -> opens confirmation dialog without restoring yet
    await user.click(screen.getByRole("button", { name: /^Restore$/i }));
    expect(
      screen.getByRole("dialog", { name: /Confirm Database Restore/i })
    ).toBeInTheDocument();
    expect(mockRestoreBackup).not.toHaveBeenCalled();

    // Confirm restore
    await user.click(
      screen.getByRole("button", { name: /Yes, Restore Backup/i })
    );
    await waitFor(() => {
      expect(mockRestoreBackup).toHaveBeenCalledWith(
        "manual",
        sampleBackup.filename
      );
    });
    expect(
      await screen.findByText(/Pre-restore safety backup saved as safety_1\.sqlite/i)
    ).toBeInTheDocument();
  });
});
