import { invokeCommand } from "@/services/tauri";

export type CsvEntity =
  | "clients"
  | "services"
  | "appointments"
  | "appointment_services";

export type BackupType = "manual" | "automatic" | "safety";

export interface ExportResult {
  format: string;
  filename: string;
  file_path: string;
  size_bytes: number;
  record_count: number;
  exported_at_utc: string;
}

export interface BackupMetadata {
  id: string;
  filename: string;
  backup_type: BackupType;
  app_version: string;
  schema_version: number;
  created_at_utc: string;
  size_bytes: number;
  integrity_status: string;
  notes: string | null;
}

export interface BackupValidationResult {
  valid: boolean;
  integrity_ok: boolean;
  schema_compatible: boolean;
  backup_schema_version: number;
  current_schema_version: number;
  message: string;
}

export interface RestoreResult {
  restored_filename: string;
  restored_schema_version: number;
  safety_backup: BackupMetadata;
}

export async function exportBusinessJson(
  customFilename?: string | null
): Promise<ExportResult> {
  return invokeCommand<ExportResult>("export_business_json", {
    customFilename: customFilename || null,
  });
}

export async function exportEntityCsv(
  entity: CsvEntity,
  customFilename?: string | null
): Promise<ExportResult> {
  return invokeCommand<ExportResult>("export_entity_csv", {
    entity,
    customFilename: customFilename || null,
  });
}

export async function listBackups(): Promise<BackupMetadata[]> {
  return invokeCommand<BackupMetadata[]>("list_backups");
}

export async function createManualBackup(
  notes?: string | null
): Promise<BackupMetadata> {
  return invokeCommand<BackupMetadata>("create_manual_backup", {
    notes: notes || null,
  });
}

export async function validateBackup(
  backupType: BackupType,
  filename: string
): Promise<BackupValidationResult> {
  return invokeCommand<BackupValidationResult>("validate_backup", {
    backupType,
    filename,
  });
}

export async function restoreBackup(
  backupType: BackupType,
  filename: string
): Promise<RestoreResult> {
  return invokeCommand<RestoreResult>("restore_backup", {
    backupType,
    filename,
  });
}
