#!/usr/bin/env sh

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# shellcheck source=bin/lib/runtime-common.sh
. "$SCRIPT_DIR/lib/runtime-common.sh"

REPO_ROOT=$(router_repo_root "$SCRIPT_DIR")
DEV_HOME=$(router_default_dev_home "$REPO_ROOT")
FOREGROUND=0
DRY_RUN=0
WAIT_SECONDS=60
INSTALL_DEPS=0
PREVIEW_MODE=1
TAURI_MODE=0

CLI_DATABASE_URL=''
CLI_GATEWAY_BIND=''
CLI_ADMIN_BIND=''
CLI_PORTAL_BIND=''
CLI_WEB_BIND=''

while [ "$#" -gt 0 ]; do
  case "$1" in
    --foreground)
      FOREGROUND=1
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --wait-seconds)
      [ "$#" -ge 2 ] || router_die "--wait-seconds requires a value"
      WAIT_SECONDS="$2"
      shift 2
      ;;
    --install)
      INSTALL_DEPS=1
      shift
      ;;
    --preview)
      PREVIEW_MODE=1
      TAURI_MODE=0
      shift
      ;;
    --browser)
      PREVIEW_MODE=0
      TAURI_MODE=0
      shift
      ;;
    --tauri)
      PREVIEW_MODE=0
      TAURI_MODE=1
      shift
      ;;
    --database-url)
      [ "$#" -ge 2 ] || router_die "--database-url requires a value"
      CLI_DATABASE_URL="$2"
      shift 2
      ;;
    --gateway-bind)
      [ "$#" -ge 2 ] || router_die "--gateway-bind requires a value"
      CLI_GATEWAY_BIND="$2"
      shift 2
      ;;
    --admin-bind)
      [ "$#" -ge 2 ] || router_die "--admin-bind requires a value"
      CLI_ADMIN_BIND="$2"
      shift 2
      ;;
    --portal-bind)
      [ "$#" -ge 2 ] || router_die "--portal-bind requires a value"
      CLI_PORTAL_BIND="$2"
      shift 2
      ;;
    --web-bind)
      [ "$#" -ge 2 ] || router_die "--web-bind requires a value"
      CLI_WEB_BIND="$2"
      shift 2
      ;;
    *)
      router_die "unknown option: $1"
      ;;
  esac
done

CONFIG_DIR="$DEV_HOME/config"
DATA_DIR="$DEV_HOME/data"
LOG_DIR="$DEV_HOME/log"
RUN_DIR="$DEV_HOME/run"
ENV_FILE="$CONFIG_DIR/router-dev.env"
PID_FILE="$RUN_DIR/start-workspace.pid"
STOP_FILE="$RUN_DIR/start-workspace.stop"
STDOUT_LOG="$LOG_DIR/start-workspace.stdout.log"
STDERR_LOG="$LOG_DIR/start-workspace.stderr.log"
PLAN_FILE="$RUN_DIR/start-workspace.plan.txt"

router_ensure_dir "$CONFIG_DIR"
router_ensure_dir "$DATA_DIR"
router_ensure_dir "$LOG_DIR"
router_ensure_dir "$RUN_DIR"

router_load_env_file "$ENV_FILE"

SDKWORK_DATABASE_URL=${SDKWORK_DATABASE_URL:-"sqlite://$(router_portable_path "$DATA_DIR")/sdkwork-api-router-dev.db"}
SDKWORK_GATEWAY_BIND=${SDKWORK_GATEWAY_BIND:-"127.0.0.1:9980"}
SDKWORK_ADMIN_BIND=${SDKWORK_ADMIN_BIND:-"127.0.0.1:9981"}
SDKWORK_PORTAL_BIND=${SDKWORK_PORTAL_BIND:-"127.0.0.1:9982"}
SDKWORK_WEB_BIND=${SDKWORK_WEB_BIND:-"127.0.0.1:9983"}

[ -n "$CLI_DATABASE_URL" ] && SDKWORK_DATABASE_URL="$CLI_DATABASE_URL"
[ -n "$CLI_GATEWAY_BIND" ] && SDKWORK_GATEWAY_BIND="$CLI_GATEWAY_BIND"
[ -n "$CLI_ADMIN_BIND" ] && SDKWORK_ADMIN_BIND="$CLI_ADMIN_BIND"
[ -n "$CLI_PORTAL_BIND" ] && SDKWORK_PORTAL_BIND="$CLI_PORTAL_BIND"
[ -n "$CLI_WEB_BIND" ] && SDKWORK_WEB_BIND="$CLI_WEB_BIND"

if [ ! -d "$REPO_ROOT/apps/sdkwork-router-admin/node_modules" ] || [ ! -d "$REPO_ROOT/apps/sdkwork-router-portal/node_modules" ]; then
  INSTALL_DEPS=1
fi

router_validate_file "workspace launcher" "$REPO_ROOT/scripts/dev/start-workspace.mjs"

set -- \
  scripts/dev/start-workspace.mjs \
  --database-url "$SDKWORK_DATABASE_URL" \
  --gateway-bind "$SDKWORK_GATEWAY_BIND" \
  --admin-bind "$SDKWORK_ADMIN_BIND" \
  --portal-bind "$SDKWORK_PORTAL_BIND" \
  --web-bind "$SDKWORK_WEB_BIND" \
  --stop-file "$STOP_FILE"

[ "$INSTALL_DEPS" = '1' ] && set -- "$@" --install
[ "$PREVIEW_MODE" = '1' ] && set -- "$@" --preview
[ "$TAURI_MODE" = '1' ] && set -- "$@" --tauri

cd "$REPO_ROOT"
node "$@" --dry-run > "$PLAN_FILE"

if [ "$DRY_RUN" = '1' ]; then
  cat "$PLAN_FILE"
  exit 0
fi

if [ "$FOREGROUND" = '1' ]; then
  rm -f "$STOP_FILE"
  exec node "$@"
fi

router_require_not_running "$PID_FILE"
rm -f "$STOP_FILE"
: > "$STDOUT_LOG"
: > "$STDERR_LOG"

nohup node "$@" >> "$STDOUT_LOG" 2>> "$STDERR_LOG" &
PID=$!
printf '%s\n' "$PID" > "$PID_FILE"

GATEWAY_HEALTH_URL=$(router_resolve_loopback_url "$SDKWORK_GATEWAY_BIND" "/health")
ADMIN_HEALTH_URL=$(router_resolve_loopback_url "$SDKWORK_ADMIN_BIND" "/admin/health")
PORTAL_HEALTH_URL=$(router_resolve_loopback_url "$SDKWORK_PORTAL_BIND" "/portal/health")

if ! router_wait_for_url "$GATEWAY_HEALTH_URL" "$WAIT_SECONDS" "$PID" \
  || ! router_wait_for_url "$ADMIN_HEALTH_URL" "$WAIT_SECONDS" "$PID" \
  || ! router_wait_for_url "$PORTAL_HEALTH_URL" "$WAIT_SECONDS" "$PID"; then
  WORKSPACE_EXITED=0
  if ! router_is_pid_running "$PID"; then
    WORKSPACE_EXITED=1
  fi
  router_tail_log "$STDOUT_LOG"
  router_tail_log "$STDERR_LOG"
  router_stop_pid "$PID" "$WAIT_SECONDS" 1 || true
  rm -f "$PID_FILE"
  rm -f "$STOP_FILE"
  if [ "$WORKSPACE_EXITED" = '1' ]; then
    router_die "development workspace exited before backend health checks completed; see startup log above"
  fi
  router_die "development services failed health checks"
fi

if [ "$PREVIEW_MODE" = '1' ] || [ "$TAURI_MODE" = '1' ]; then
  DEV_ADMIN_URL=$(router_resolve_loopback_url "$SDKWORK_WEB_BIND" "/admin/")
  DEV_PORTAL_URL=$(router_resolve_loopback_url "$SDKWORK_WEB_BIND" "/portal/")
else
  DEV_ADMIN_URL='http://127.0.0.1:5173/admin/'
  DEV_PORTAL_URL='http://127.0.0.1:5174/portal/'
fi

if ! router_wait_for_url "$DEV_ADMIN_URL" "$WAIT_SECONDS" "$PID" \
  || ! router_wait_for_url "$DEV_PORTAL_URL" "$WAIT_SECONDS" "$PID"; then
  WORKSPACE_EXITED=0
  if ! router_is_pid_running "$PID"; then
    WORKSPACE_EXITED=1
  fi
  router_tail_log "$STDOUT_LOG"
  router_tail_log "$STDERR_LOG"
  router_stop_pid "$PID" "$WAIT_SECONDS" 1 || true
  rm -f "$PID_FILE"
  rm -f "$STOP_FILE"
  if [ "$WORKSPACE_EXITED" = '1' ]; then
    router_die "development workspace exited before web surfaces became ready; see startup log above"
  fi
  router_die "development web surfaces failed health checks"
fi

router_log "started development workspace (pid=$PID)"
if [ "$PREVIEW_MODE" = '1' ]; then
  ROUTER_MODE='development preview'
  UNIFIED_ACCESS_ENABLED='1'
elif [ "$TAURI_MODE" = '1' ]; then
  ROUTER_MODE='development tauri'
  UNIFIED_ACCESS_ENABLED='1'
else
  ROUTER_MODE='development browser'
  UNIFIED_ACCESS_ENABLED='0'
fi

router_startup_summary \
  "$ROUTER_MODE" \
  "$UNIFIED_ACCESS_ENABLED" \
  "$SDKWORK_WEB_BIND" \
  "$SDKWORK_GATEWAY_BIND" \
  "$SDKWORK_ADMIN_BIND" \
  "$SDKWORK_PORTAL_BIND" \
  "$DEV_ADMIN_URL" \
  "$DEV_PORTAL_URL" \
  "$STDOUT_LOG" \
  "$STDERR_LOG"
