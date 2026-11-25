import { invoke } from "@tauri-apps/api/core";

export interface AppInfo {
  name: string;
  version: string;
}

export type StorageMode = "portable" | "fallback";

export interface StorageInfo {
  mode: StorageMode;
  data_dir: string;
  fallback_reason: string | null;
}

export interface DatabaseHealth {
  ok: boolean;
  integrity_ok: boolean;
  foreign_keys_enabled: boolean;
  journal_mode: string;
  applied_migrations: number;
  latest_migration_version: number | null;
}

export interface BackendError {
  code: string;
  message: string;
}

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  return invoke<T>(command, args);
}

export async function getAppInfo(): Promise<AppInfo> {
  return invokeCommand<AppInfo>("app_info");
}

export async function getStorageInfo(): Promise<StorageInfo> {
  return invokeCommand<StorageInfo>("get_storage_info");
}

export async function getDatabaseHealth(): Promise<DatabaseHealth> {
  return invokeCommand<DatabaseHealth>("get_database_health");
}
