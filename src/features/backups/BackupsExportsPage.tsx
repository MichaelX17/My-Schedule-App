import { useState } from "react";
import {
  AlertCircle,
  AlertTriangle,
  CheckCircle2,
  DatabaseBackup,
  Download,
  FileJson,
  FileSpreadsheet,
  RotateCcw,
  ShieldCheck,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type {
  BackupMetadata,
  CsvEntity,
  ExportResult,
} from "@/services/backups";
import { extractErrorMessage } from "@/services/business";
import {
  useBackupsList,
  useCreateManualBackup,
  useExportCsv,
  useExportJson,
  useRestoreBackup,
  useValidateBackup,
} from "./hooks/useBackups";

const CSV_ENTITIES: { value: CsvEntity; label: string }[] = [
  { value: "clients", label: "Clients CSV" },
  { value: "services", label: "Services CSV" },
  { value: "appointments", label: "Appointments CSV" },
  { value: "appointment_services", label: "Appointment Services CSV" },
];

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

function BackupTypeBadge({ type }: { type: BackupMetadata["backup_type"] }) {
  if (type === "automatic") {
    return (
      <span className="rounded-full bg-secondary px-2 py-0.5 text-[11px] font-medium text-secondary-foreground">
        Automatic
      </span>
    );
  }
  if (type === "safety") {
    return (
      <span className="rounded-full bg-amber-500/15 px-2 py-0.5 text-[11px] font-medium text-amber-800 dark:text-amber-300">
        Safety
      </span>
    );
  }
  return (
    <span className="rounded-full bg-primary/10 px-2 py-0.5 text-[11px] font-medium text-primary">
      Manual
    </span>
  );
}

export function BackupsExportsPage() {
  const [customExportName, setCustomExportName] = useState("");
  const [lastExport, setLastExport] = useState<ExportResult | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [restoreTarget, setRestoreTarget] = useState<BackupMetadata | null>(
    null
  );

  const {
    data: backups = [],
    isLoading,
    isError,
    error,
  } = useBackupsList();

  const createManualBackup = useCreateManualBackup();
  const validateBackup = useValidateBackup();
  const restoreBackup = useRestoreBackup();
  const exportJson = useExportJson();
  const exportCsv = useExportCsv();

  const handleExportJson = async () => {
    setErrorMessage(null);
    setStatusMessage(null);
    try {
      const res = await exportJson.mutateAsync(customExportName.trim() || null);
      setLastExport(res);
      setCustomExportName("");
    } catch (err) {
      setErrorMessage(extractErrorMessage(err));
    }
  };

  const handleExportCsv = async (entity: CsvEntity) => {
    setErrorMessage(null);
    setStatusMessage(null);
    try {
      const res = await exportCsv.mutateAsync({
        entity,
        customFilename: customExportName.trim() || null,
      });
      setLastExport(res);
      setCustomExportName("");
    } catch (err) {
      setErrorMessage(extractErrorMessage(err));
    }
  };

  const handleCreateManualBackup = async () => {
    setErrorMessage(null);
    setStatusMessage(null);
    try {
      const meta = await createManualBackup.mutateAsync("Manual operator backup");
      setStatusMessage(
        `Backup created and verified: ${meta.filename} (${formatBytes(meta.size_bytes)}).`
      );
    } catch (err) {
      setErrorMessage(extractErrorMessage(err));
    }
  };

  const handleValidateBackup = async (item: BackupMetadata) => {
    setErrorMessage(null);
    setStatusMessage(null);
    try {
      const result = await validateBackup.mutateAsync({
        backupType: item.backup_type,
        filename: item.filename,
      });
      if (result.valid) {
        setStatusMessage(result.message);
      } else {
        setErrorMessage(result.message);
      }
    } catch (err) {
      setErrorMessage(extractErrorMessage(err));
    }
  };

  const handleConfirmRestore = async () => {
    if (!restoreTarget) return;
    setErrorMessage(null);
    setStatusMessage(null);
    try {
      const res = await restoreBackup.mutateAsync({
        backupType: restoreTarget.backup_type,
        filename: restoreTarget.filename,
      });
      setRestoreTarget(null);
      setStatusMessage(
        `Restored ${res.restored_filename} successfully. Pre-restore safety backup saved as ${res.safety_backup.filename}.`
      );
    } catch (err) {
      setRestoreTarget(null);
      setErrorMessage(extractErrorMessage(err));
    }
  };

  const busyExport = exportJson.isPending || exportCsv.isPending;

  return (
    <div className="mx-auto w-full max-w-5xl p-6 sm:p-8 space-y-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">
          Backups &amp; Exports
        </h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Export business data to versioned JSON or CSV files, and manage
          SQLite-safe local backups and restores.
        </p>
      </div>

      {statusMessage && (
        <div
          role="status"
          className="flex items-start gap-2.5 rounded-xl border border-emerald-500/40 bg-emerald-500/5 p-3.5 text-sm text-foreground"
        >
          <CheckCircle2 className="size-4 text-emerald-600 dark:text-emerald-400 shrink-0 mt-0.5" />
          <span>{statusMessage}</span>
        </div>
      )}

      {errorMessage && (
        <div
          role="alert"
          className="flex items-start gap-2.5 rounded-xl border border-destructive/40 bg-card p-3.5 text-sm text-destructive"
        >
          <AlertCircle className="size-4 shrink-0 mt-0.5" />
          <span>{errorMessage}</span>
        </div>
      )}

      {/* Exports Section */}
      <section
        aria-label="Data exports"
        className="rounded-xl border border-border bg-card p-5 space-y-4"
      >
        <div className="space-y-1">
          <h2 className="text-base font-semibold">Data Exports</h2>
          <p className="text-xs text-muted-foreground">
            Files are written atomically to your managed{" "}
            <code className="font-mono">data/exports/</code> folder with UTF-8
            encoding and no internal secrets.
          </p>
        </div>

        <div className="max-w-sm">
          <label
            htmlFor="export-filename"
            className="block text-xs font-medium text-muted-foreground mb-1"
          >
            Optional custom filename (leave blank for timestamped default)
          </label>
          <Input
            id="export-filename"
            value={customExportName}
            onChange={(e) => setCustomExportName(e.target.value)}
            placeholder="e.g. quarterly_snapshot"
          />
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <Button
            type="button"
            disabled={busyExport}
            onClick={handleExportJson}
          >
            <FileJson className="size-4" />
            Export Full JSON (v1)
          </Button>

          {CSV_ENTITIES.map((ent) => (
            <Button
              key={ent.value}
              type="button"
              variant="outline"
              disabled={busyExport}
              onClick={() => void handleExportCsv(ent.value)}
            >
              <FileSpreadsheet className="size-4" />
              {ent.label}
            </Button>
          ))}
        </div>

        {lastExport && (
          <div
            role="status"
            className="rounded-lg border border-border bg-muted/40 p-3 text-xs space-y-1"
          >
            <div className="flex items-center gap-1.5 font-medium text-foreground">
              <Download className="size-3.5 text-primary" />
              <span>
                Exported {lastExport.record_count} record(s) (
                {formatBytes(lastExport.size_bytes)})
              </span>
            </div>
            <p className="text-muted-foreground break-all">
              Saved to:{" "}
              <code className="font-mono text-foreground">
                {lastExport.file_path}
              </code>
            </p>
          </div>
        )}
      </section>

      {/* Backups & Restore Section */}
      <section
        aria-label="Database backups"
        className="rounded-xl border border-border bg-card overflow-hidden"
      >
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-border bg-muted/30 p-4">
          <div className="space-y-0.5">
            <h2 className="text-base font-semibold">SQLite Database Backups</h2>
            <p className="text-xs text-muted-foreground">
              Snapshots use SQLite <code className="font-mono">VACUUM INTO</code>{" "}
              and integrity verification. Automatic daily backups retain the
              latest 30 snapshots by default.
            </p>
          </div>

          <Button
            type="button"
            disabled={createManualBackup.isPending}
            onClick={handleCreateManualBackup}
          >
            <DatabaseBackup className="size-4" />
            {createManualBackup.isPending
              ? "Creating Backup..."
              : "Create Manual Backup"}
          </Button>
        </div>

        {isError ? (
          <div role="alert" className="p-4 text-sm text-destructive">
            Unable to list backups: {extractErrorMessage(error)}
          </div>
        ) : isLoading ? (
          <div className="p-8 text-center text-sm text-muted-foreground">
            Loading backups...
          </div>
        ) : backups.length === 0 ? (
          <div className="p-10 text-center text-sm text-muted-foreground">
            No backups found yet. Create a manual backup above.
          </div>
        ) : (
          <div className="divide-y divide-border">
            {backups.map((item) => (
              <div
                key={`${item.backup_type}-${item.filename}`}
                className="flex flex-col md:flex-row md:items-center justify-between gap-3 p-4"
              >
                <div className="space-y-1 min-w-0">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="text-sm font-medium font-mono truncate">
                      {item.filename}
                    </span>
                    <BackupTypeBadge type={item.backup_type} />
                    <span className="text-[11px] text-muted-foreground">
                      Schema v{item.schema_version} · App v{item.app_version} ·{" "}
                      {formatBytes(item.size_bytes)}
                    </span>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    {item.created_at_utc
                      ? `Created ${item.created_at_utc.replace("T", " ").slice(0, 19)} UTC`
                      : "Timestamp unavailable"}
                    {item.notes ? ` · ${item.notes}` : ""}
                  </p>
                </div>

                <div className="flex items-center gap-2 shrink-0">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    disabled={validateBackup.isPending}
                    onClick={() => void handleValidateBackup(item)}
                  >
                    <ShieldCheck className="size-3.5" />
                    Validate
                  </Button>
                  <Button
                    type="button"
                    variant="secondary"
                    size="sm"
                    disabled={restoreBackup.isPending}
                    onClick={() => setRestoreTarget(item)}
                  >
                    <RotateCcw className="size-3.5" />
                    Restore
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Explicit Restore Confirmation Dialog */}
      {restoreTarget && (
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="restore-dialog-title"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
        >
          <div className="w-full max-w-md rounded-xl border border-border bg-card p-6 shadow-lg space-y-4">
            <div className="flex items-start gap-3">
              <AlertTriangle className="size-5 text-amber-600 dark:text-amber-400 shrink-0 mt-0.5" />
              <div className="space-y-1">
                <h2
                  id="restore-dialog-title"
                  className="text-base font-semibold"
                >
                  Confirm Database Restore
                </h2>
                <p className="text-sm text-muted-foreground">
                  Restoring{" "}
                  <code className="font-mono text-xs text-foreground">
                    {restoreTarget.filename}
                  </code>{" "}
                  will replace your current active database.
                </p>
                <p className="text-xs text-muted-foreground">
                  A verified pre-restore <strong>safety backup</strong> of your
                  current database will be created automatically before any data
                  is replaced.
                </p>
              </div>
            </div>

            <div className="flex justify-end gap-2 pt-2">
              <Button
                type="button"
                variant="outline"
                disabled={restoreBackup.isPending}
                onClick={() => setRestoreTarget(null)}
              >
                Cancel
              </Button>
              <Button
                type="button"
                variant="destructive"
                disabled={restoreBackup.isPending}
                onClick={handleConfirmRestore}
              >
                {restoreBackup.isPending
                  ? "Restoring..."
                  : "Yes, Restore Backup"}
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
