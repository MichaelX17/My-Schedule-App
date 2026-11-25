import { useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import {
  AlertCircle,
  CheckCircle2,
  Download,
  RefreshCw,
  ShieldCheck,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { extractErrorMessage } from "@/services/business";
import { isTauriRuntime } from "@/services/tauri";
import {
  checkForUpdates,
  downloadAndStageUpdate,
  getUpdaterInfo,
  type StagedUpdateResult,
  type UpdateCheckResult,
} from "@/services/updater";

export function UpdatesSection() {
  const [checkResult, setCheckResult] = useState<UpdateCheckResult | null>(
    null
  );
  const [stagedResult, setStagedResult] = useState<StagedUpdateResult | null>(
    null
  );
  const [stageError, setStageError] = useState<string | null>(null);

  const { data: info } = useQuery({
    queryKey: ["system", "updater-info"],
    queryFn: getUpdaterInfo,
    enabled: isTauriRuntime(),
    staleTime: Infinity,
  });

  const checkMutation = useMutation({
    mutationFn: checkForUpdates,
    onSuccess: (res) => {
      setCheckResult(res);
      setStageError(null);
    },
  });

  const stageMutation = useMutation({
    mutationFn: downloadAndStageUpdate,
    onSuccess: (res) => {
      setStagedResult(res);
      setStageError(null);
    },
    onError: (err) => {
      setStageError(extractErrorMessage(err));
    },
  });

  return (
    <section
      aria-label="Application updates"
      className="rounded-xl border border-border bg-card p-5 space-y-4"
    >
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div className="space-y-0.5">
          <div className="flex items-center gap-2">
            <ShieldCheck className="size-4 text-primary" aria-hidden="true" />
            <h2 className="text-base font-semibold">
              GitHub Releases &amp; Updates
            </h2>
          </div>
          <p className="text-xs text-muted-foreground">
            Current version{" "}
            <span className="font-mono text-foreground">
              v{info?.current_version ?? "0.1.0"}
            </span>{" "}
            ({info?.platform_key ?? "..."}) · Source:{" "}
            <span className="font-mono">
              {info?.github_repo ?? "MichaelX17/My-Schedule-App"}
            </span>
          </p>
        </div>

        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={checkMutation.isPending}
          onClick={() => void checkMutation.mutateAsync()}
        >
          <RefreshCw
            className={`size-4 ${checkMutation.isPending ? "animate-spin" : ""}`}
          />
          {checkMutation.isPending ? "Checking..." : "Check for Updates"}
        </Button>
      </div>

      <p className="text-xs text-muted-foreground">
        Updates are verified using Ed25519 manifest signatures and SHA-256
        artifact digests. Before staging an update, My Schedule App
        automatically creates a verified SQLite safety backup. Updates are never
        installed without your explicit confirmation.
      </p>

      {checkResult && (
        <div
          role="status"
          className={`rounded-lg border p-3.5 text-xs space-y-2 ${
            checkResult.status === "update_available"
              ? "border-primary/40 bg-primary/5"
              : checkResult.status === "verification_failed" ||
                  checkResult.status === "error"
                ? "border-destructive/40 bg-card text-destructive"
                : "border-border bg-muted/40 text-foreground"
          }`}
        >
          <p className="font-medium">{checkResult.message}</p>

          {checkResult.notes && (
            <p className="text-muted-foreground whitespace-pre-line">
              {checkResult.notes}
            </p>
          )}

          {checkResult.status === "update_available" && (
            <div className="pt-1">
              <Button
                type="button"
                size="sm"
                disabled={stageMutation.isPending}
                onClick={() => void stageMutation.mutateAsync()}
              >
                <Download className="size-4" />
                {stageMutation.isPending
                  ? "Backing Up & Staging..."
                  : `Back Up & Stage v${checkResult.latest_version}`}
              </Button>
            </div>
          )}
        </div>
      )}

      {stageError && (
        <div
          role="alert"
          className="flex items-start gap-2 rounded-lg border border-destructive/40 bg-card p-3 text-xs text-destructive"
        >
          <AlertCircle className="size-4 shrink-0 mt-0.5" />
          <span>{stageError}</span>
        </div>
      )}

      {stagedResult && (
        <div
          role="status"
          className="flex items-start gap-2 rounded-lg border border-emerald-500/40 bg-emerald-500/5 p-3.5 text-xs space-y-1"
        >
          <CheckCircle2 className="size-4 text-emerald-600 dark:text-emerald-400 shrink-0 mt-0.5" />
          <div className="space-y-1">
            <p className="font-medium text-foreground">
              v{stagedResult.version} verified and staged
            </p>
            <p className="text-muted-foreground">{stagedResult.message}</p>
            <p className="text-muted-foreground break-all">
              Safety backup:{" "}
              <code className="font-mono text-foreground">
                {stagedResult.safety_backup.filename}
              </code>
            </p>
          </div>
        </div>
      )}
    </section>
  );
}
