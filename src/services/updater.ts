import type { BackupMetadata } from "@/services/backups";
import { invokeCommand } from "@/services/tauri";

export type UpdateCheckStatus =
  | "update_available"
  | "up_to_date"
  | "platform_unsupported"
  | "offline"
  | "rate_limited"
  | "no_release_published"
  | "signing_key_unconfigured"
  | "verification_failed"
  | "error";

export interface UpdaterInfo {
  current_version: string;
  platform_key: string;
  github_repo: string;
  releases_url: string;
  signing_key_configured: boolean;
}

export interface UpdateCheckResult {
  status: UpdateCheckStatus;
  current_version: string;
  platform_key: string;
  latest_version: string | null;
  notes: string | null;
  pub_date: string | null;
  artifact_filename: string | null;
  message: string;
}

export interface StagedUpdateResult {
  version: string;
  platform_key: string;
  staged_artifact_path: string;
  apply_script_path: string;
  sha256: string;
  safety_backup: BackupMetadata;
  message: string;
}

export async function getUpdaterInfo(): Promise<UpdaterInfo> {
  return invokeCommand<UpdaterInfo>("get_updater_info");
}

export async function checkForUpdates(): Promise<UpdateCheckResult> {
  return invokeCommand<UpdateCheckResult>("check_for_updates");
}

export async function downloadAndStageUpdate(): Promise<StagedUpdateResult> {
  return invokeCommand<StagedUpdateResult>("download_and_stage_update");
}
