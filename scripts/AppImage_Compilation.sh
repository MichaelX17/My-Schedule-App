#!/usr/bin/env bash
set -euo pipefail

# Builds a Linux .AppImage release bundle for My Schedule App.
#
# Usage:
#   ./scripts/AppImage_Compilation.sh
#   ./scripts/AppImage_Compilation.sh --skip-checks
#   MY_SCHEDULE_UPDATE_PUBKEY="<base64-ed25519-pubkey>" ./scripts/AppImage_Compilation.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

SKIP_CHECKS=0
OUT_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-checks)
      SKIP_CHECKS=1
      shift
      ;;
    --out-dir)
      if [[ $# -lt 2 ]]; then
        echo "Error: --out-dir requires a directory path." >&2
        exit 1
      fi
      OUT_DIR="$2"
      shift 2
      ;;
    -h|--help)
      echo "Usage: $0 [--skip-checks] [--out-dir <dir>]"
      echo ""
      echo "Options:"
      echo "  --skip-checks    Skip typecheck, lint, and tests before bundling"
      echo "  --out-dir <dir>  Copy the resulting .AppImage into <dir>"
      echo ""
      echo "Environment:"
      echo "  MY_SCHEDULE_UPDATE_PUBKEY  Optional base64 Ed25519 public key for signed updates"
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

for cmd in node npm cargo; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Error: Required command '$cmd' not found in PATH." >&2
    exit 1
  fi
done

cd "$ROOT_DIR"

echo "==> Project root: $ROOT_DIR"
if [[ -n "${MY_SCHEDULE_UPDATE_PUBKEY:-}" ]]; then
  echo "==> Signed update public key detected (MY_SCHEDULE_UPDATE_PUBKEY is set)"
else
  echo "==> Note: MY_SCHEDULE_UPDATE_PUBKEY is not set (local/unsigned build)"
fi

if [[ "$SKIP_CHECKS" -eq 0 ]]; then
  echo "==> Running frontend typecheck, lint, and tests..."
  npm run typecheck
  npm run lint
  npm run test
fi

echo "==> Compiling Linux .AppImage release bundle..."
npm run tauri build -- --bundles appimage

BUNDLE_DIR="${ROOT_DIR}/src-tauri/target/release/bundle/appimage"
APPIMAGE_PATH="$(find "$BUNDLE_DIR" -maxdepth 1 -type f -name '*.AppImage' -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -n 1 | cut -d' ' -f2-)"

if [[ -z "$APPIMAGE_PATH" || ! -f "$APPIMAGE_PATH" ]]; then
  echo "Error: No .AppImage file was produced in $BUNDLE_DIR" >&2
  exit 1
fi

chmod +x "$APPIMAGE_PATH"

if [[ -n "$OUT_DIR" ]]; then
  mkdir -p "$OUT_DIR"
  cp -f "$APPIMAGE_PATH" "$OUT_DIR/"
  FINAL_PATH="${OUT_DIR}/$(basename "$APPIMAGE_PATH")"
  chmod +x "$FINAL_PATH"
else
  FINAL_PATH="$APPIMAGE_PATH"
fi

echo ""
echo "=========================================="
echo " Linux .AppImage build succeeded!"
echo "=========================================="
echo "File:   $FINAL_PATH"
echo "Size:   $(du -h "$FINAL_PATH" | cut -f1)"
if command -v sha256sum >/dev/null 2>&1; then
  echo "SHA256: $(sha256sum "$FINAL_PATH" | awk '{print $1}')"
fi
echo ""
echo "Run it with:"
echo "  \"$FINAL_PATH\""
