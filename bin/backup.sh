#!/usr/bin/env sh

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# shellcheck source=bin/lib/runtime-common.sh
. "$SCRIPT_DIR/lib/runtime-common.sh"

REPO_ROOT=$(router_repo_root "$SCRIPT_DIR")
DEFAULT_HOME=$(router_default_install_home "$REPO_ROOT")

RUNTIME_HOME=''
OUTPUT_PATH=''
DRY_RUN=0
FORCE=0
PLAN_FORMAT='json'

while [ "$#" -gt 0 ]; do
  case "$1" in
    --home)
      [ "$#" -ge 2 ] || router_die "--home requires a value"
      RUNTIME_HOME="$2"
      shift 2
      ;;
    --output)
      [ "$#" -ge 2 ] || router_die "--output requires a value"
      OUTPUT_PATH="$2"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --force)
      FORCE=1
      shift
      ;;
    --plan-format)
      [ "$#" -ge 2 ] || router_die "--plan-format requires a value"
      PLAN_FORMAT="$2"
      shift 2
      ;;
    *)
      router_die "unknown option: $1"
      ;;
  esac
done

[ -n "$OUTPUT_PATH" ] || router_die "--output requires a value"

if router_is_windows; then
  PS_SCRIPT="$(router_windows_path "$SCRIPT_DIR/backup.ps1")"
  set --
  [ -n "$RUNTIME_HOME" ] && set -- "$@" -Home "$(router_windows_cli_path "$RUNTIME_HOME")"
  [ -n "$OUTPUT_PATH" ] && set -- "$@" -OutputPath "$(router_windows_cli_path "$OUTPUT_PATH")"
  [ "$DRY_RUN" = '1' ] && set -- "$@" -DryRun
  [ "$FORCE" = '1' ] && set -- "$@" -Force
  [ "$PLAN_FORMAT" != 'json' ] && set -- "$@" -PlanFormat "$PLAN_FORMAT"
  exec powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$PS_SCRIPT" "$@"
fi

if [ -z "$RUNTIME_HOME" ]; then
  if [ -f "$SCRIPT_DIR/../release-manifest.json" ]; then
    RUNTIME_HOME=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
  elif [ -f "$SCRIPT_DIR/$(router_binary_name router-product-service)" ]; then
    RUNTIME_HOME=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
  else
    RUNTIME_HOME="$DEFAULT_HOME"
  fi
fi

RUNTIME_HOME=$(router_resolve_absolute_path "$PWD" "$RUNTIME_HOME")
MANIFEST_FILE=$(router_release_manifest_path "$RUNTIME_HOME")
MANIFEST_RELEASE_ROOT=$(router_release_manifest_string "$MANIFEST_FILE" 'releaseRoot' || true)
MANIFEST_ROUTER_BINARY=$(router_release_manifest_string "$MANIFEST_FILE" 'routerBinary' || true)

BINARY_PATH="$RUNTIME_HOME/bin/$(router_binary_name router-product-service)"
if [ -n "$MANIFEST_RELEASE_ROOT" ]; then
  BINARY_PATH="$MANIFEST_RELEASE_ROOT/bin/$(router_binary_name router-product-service)"
fi
if [ -n "$MANIFEST_ROUTER_BINARY" ]; then
  BINARY_PATH="$MANIFEST_ROUTER_BINARY"
fi
if [ ! -f "$BINARY_PATH" ]; then
  router_die "router-product-service binary not found: $BINARY_PATH"
fi

set -- "$BINARY_PATH" --runtime-home "$RUNTIME_HOME" --backup-output "$OUTPUT_PATH"
[ "$FORCE" = '1' ] && set -- "$@" --force
[ "$DRY_RUN" = '1' ] && set -- "$@" --dry-run --plan-format "$PLAN_FORMAT"

exec "$@"
