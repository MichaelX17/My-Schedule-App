import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createManualBackup,
  exportBusinessJson,
  exportEntityCsv,
  listBackups,
  restoreBackup,
  validateBackup,
  type BackupType,
  type CsvEntity,
} from "@/services/backups";
import { isTauriRuntime } from "@/services/tauri";

export const BACKUP_QUERY_KEYS = {
  all: ["backups"] as const,
  list: ["backups", "list"] as const,
};

export function useBackupsList() {
  return useQuery({
    queryKey: BACKUP_QUERY_KEYS.list,
    queryFn: listBackups,
    enabled: isTauriRuntime(),
  });
}

export function useCreateManualBackup() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (notes?: string | null) => createManualBackup(notes),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: BACKUP_QUERY_KEYS.all });
    },
  });
}

export function useValidateBackup() {
  return useMutation({
    mutationFn: ({
      backupType,
      filename,
    }: {
      backupType: BackupType;
      filename: string;
    }) => validateBackup(backupType, filename),
  });
}

export function useRestoreBackup() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      backupType,
      filename,
    }: {
      backupType: BackupType;
      filename: string;
    }) => restoreBackup(backupType, filename),
    onSuccess: async () => {
      // Reload all queries across the app so every screen reflects the restored database.
      await queryClient.invalidateQueries();
    },
  });
}

export function useExportJson() {
  return useMutation({
    mutationFn: (customFilename?: string | null) =>
      exportBusinessJson(customFilename),
  });
}

export function useExportCsv() {
  return useMutation({
    mutationFn: ({
      entity,
      customFilename,
    }: {
      entity: CsvEntity;
      customFilename?: string | null;
    }) => exportEntityCsv(entity, customFilename),
  });
}
