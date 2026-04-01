#!/usr/bin/env sh

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# shellcheck source=bin/lib/runtime-common.sh
. "$SCRIPT_DIR/lib/runtime-common.sh"

REPO_ROOT=$(router_repo_root "$SCRIPT_DIR")
DEFAULT_HOME=$(router_default_install_home "$REPO_ROOT")

RUNTIME_HOME=''
FOREGROUND=0
DRY_RUN=0
WAIT_SECONDS=60

CLI_BIND=''
CLI_CONFIG_DIR=''
CLI_CONFIG_FILE=''
CLI_DATABASE_URL=''
CLI_ROLES=''
CLI_NODE_ID_PREFIX=''
CLI_GATEWAY_BIND=''
CLI_ADMIN_BIND=''
CLI_PORTAL_BIND=''
CLI_GATEWAY_UPSTREAM=''
CLI_ADMIN_UPSTREAM=''
CLI_PORTAL_UPSTREAM=''
CLI_ADMIN_SITE_DIR=''
CLI_PORTAL_SITE_DIR=''

while [ "$#" -gt 0 ]; do
  case "$1" in
    --home)
      [ "$#" -ge 2 ] || router_die "--home requires a value"
      RUNTIME_HOME="$2"
      shift 2
      ;;
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
    --bind)
      [ "$#" -ge 2 ] || router_die "--bind requires a value"
      CLI_BIND="$2"
      shift 2
      ;;
    --config-dir)
      [ "$#" -ge 2 ] || router_die "--config-dir requires a value"
      CLI_CONFIG_DIR="$2"
      shift 2
      ;;
    --config-file)
      [ "$#" -ge 2 ] || router_die "--config-file requires a value"
      CLI_CONFIG_FILE="$2"
      shift 2
      ;;
    --database-url)
      [ "$#" -ge 2 ] || router_die "--database-url requires a value"
      CLI_DATABASE_URL="$2"
      shift 2
      ;;
    --roles)
      [ "$#" -ge 2 ] || router_die "--roles requires a value"
      CLI_ROLES="$2"
      shift 2
      ;;
    --node-id-prefix)
      [ "$#" -ge 2 ] || router_die "--node-id-prefix requires a value"
      CLI_NODE_ID_PREFIX="$2"
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
    --gateway-upstream)
      [ "$#" -ge 2 ] || router_die "--gateway-upstream requires a value"
      CLI_GATEWAY_UPSTREAM="$2"
      shift 2
      ;;
    --admin-upstream)
      [ "$#" -ge 2 ] || router_die "--admin-upstream requires a value"
      CLI_ADMIN_UPSTREAM="$2"
      shift 2
      ;;
    --portal-upstream)
      [ "$#" -ge 2 ] || router_die "--portal-upstream requires a value"
      CLI_PORTAL_UPSTREAM="$2"
      shift 2
      ;;
    --admin-site-dir)
      [ "$#" -ge 2 ] || router_die "--admin-site-dir requires a value"
      CLI_ADMIN_SITE_DIR="$2"
      shift 2
      ;;
    --portal-site-dir)
      [ "$#" -ge 2 ] || router_die "--portal-site-dir requires a value"
      CLI_PORTAL_SITE_DIR="$2"
      shift 2
      ;;
    *)
      router_die "unknown option: $1"
      ;;
  esac
done

if [ -z "$RUNTIME_HOME" ]; then
  if [ -f "$SCRIPT_DIR/$(router_binary_name router-product-service)" ]; then
    RUNTIME_HOME=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
  else
    RUNTIME_HOME="$DEFAULT_HOME"
  fi
fi

RUNTIME_HOME=$(CDPATH= cd -- "$RUNTIME_HOME" 2>/dev/null && pwd || printf '%s' "$RUNTIME_HOME")
BIN_DIR="$RUNTIME_HOME/bin"
BINARY_PATH="$BIN_DIR/$(router_binary_name router-product-service)"
CONFIG_DIR="$RUNTIME_HOME/config"
VAR_DIR="$RUNTIME_HOME/var"
DATA_DIR="$VAR_DIR/data"
LOG_DIR="$VAR_DIR/log"
RUN_DIR="$VAR_DIR/run"
ENV_FILE="$CONFIG_DIR/router.env"
PID_FILE="$RUN_DIR/router-product-service.pid"
STDOUT_LOG="$LOG_DIR/router-product-service.stdout.log"
STDERR_LOG="$LOG_DIR/router-product-service.stderr.log"
PLAN_FILE="$RUN_DIR/router-product-service.plan.json"
DEFAULT_ADMIN_SITE_DIR="$RUNTIME_HOME/sites/admin/dist"
DEFAULT_PORTAL_SITE_DIR="$RUNTIME_HOME/sites/portal/dist"

router_ensure_dir "$CONFIG_DIR"
router_ensure_dir "$DATA_DIR"
router_ensure_dir "$LOG_DIR"
router_ensure_dir "$RUN_DIR"

router_load_env_file "$ENV_FILE"

SDKWORK_ROUTER_BINARY=${SDKWORK_ROUTER_BINARY:-"$BINARY_PATH"}
SDKWORK_CONFIG_DIR=${SDKWORK_CONFIG_DIR:-"$(router_portable_path "$CONFIG_DIR")"}
SDKWORK_DATABASE_URL=${SDKWORK_DATABASE_URL:-"sqlite://$(router_portable_path "$DATA_DIR")/sdkwork-api-router.db"}
SDKWORK_WEB_BIND=${SDKWORK_WEB_BIND:-"0.0.0.0:9983"}
SDKWORK_GATEWAY_BIND=${SDKWORK_GATEWAY_BIND:-"127.0.0.1:9980"}
SDKWORK_ADMIN_BIND=${SDKWORK_ADMIN_BIND:-"127.0.0.1:9981"}
SDKWORK_PORTAL_BIND=${SDKWORK_PORTAL_BIND:-"127.0.0.1:9982"}
SDKWORK_ADMIN_SITE_DIR=${SDKWORK_ADMIN_SITE_DIR:-"$(router_portable_path "$DEFAULT_ADMIN_SITE_DIR")"}
SDKWORK_PORTAL_SITE_DIR=${SDKWORK_PORTAL_SITE_DIR:-"$(router_portable_path "$DEFAULT_PORTAL_SITE_DIR")"}

[ -n "$CLI_BIND" ] && SDKWORK_WEB_BIND="$CLI_BIND"
[ -n "$CLI_CONFIG_DIR" ] && SDKWORK_CONFIG_DIR="$CLI_CONFIG_DIR"
if [ -n "$CLI_CONFIG_FILE" ]; then
  SDKWORK_CONFIG_FILE="$CLI_CONFIG_FILE"
fi
[ -n "$CLI_DATABASE_URL" ] && SDKWORK_DATABASE_URL="$CLI_DATABASE_URL"
if [ -n "$CLI_ROLES" ]; then
  SDKWORK_ROUTER_ROLES="$CLI_ROLES"
fi
if [ -n "$CLI_NODE_ID_PREFIX" ]; then
  SDKWORK_ROUTER_NODE_ID_PREFIX="$CLI_NODE_ID_PREFIX"
fi
[ -n "$CLI_GATEWAY_BIND" ] && SDKWORK_GATEWAY_BIND="$CLI_GATEWAY_BIND"
[ -n "$CLI_ADMIN_BIND" ] && SDKWORK_ADMIN_BIND="$CLI_ADMIN_BIND"
[ -n "$CLI_PORTAL_BIND" ] && SDKWORK_PORTAL_BIND="$CLI_PORTAL_BIND"
if [ -n "$CLI_GATEWAY_UPSTREAM" ]; then
  SDKWORK_GATEWAY_PROXY_TARGET="$CLI_GATEWAY_UPSTREAM"
fi
if [ -n "$CLI_ADMIN_UPSTREAM" ]; then
  SDKWORK_ADMIN_PROXY_TARGET="$CLI_ADMIN_UPSTREAM"
fi
if [ -n "$CLI_PORTAL_UPSTREAM" ]; then
  SDKWORK_PORTAL_PROXY_TARGET="$CLI_PORTAL_UPSTREAM"
fi
[ -n "$CLI_ADMIN_SITE_DIR" ] && SDKWORK_ADMIN_SITE_DIR="$CLI_ADMIN_SITE_DIR"
[ -n "$CLI_PORTAL_SITE_DIR" ] && SDKWORK_PORTAL_SITE_DIR="$CLI_PORTAL_SITE_DIR"

export SDKWORK_ROUTER_BINARY
export SDKWORK_CONFIG_DIR
export SDKWORK_DATABASE_URL
export SDKWORK_WEB_BIND
export SDKWORK_GATEWAY_BIND
export SDKWORK_ADMIN_BIND
export SDKWORK_PORTAL_BIND
export SDKWORK_ADMIN_SITE_DIR
export SDKWORK_PORTAL_SITE_DIR
[ -n "${SDKWORK_CONFIG_FILE:-}" ] && export SDKWORK_CONFIG_FILE || true
[ -n "${SDKWORK_ROUTER_ROLES:-}" ] && export SDKWORK_ROUTER_ROLES || true
[ -n "${SDKWORK_ROUTER_NODE_ID_PREFIX:-}" ] && export SDKWORK_ROUTER_NODE_ID_PREFIX || true
[ -n "${SDKWORK_GATEWAY_PROXY_TARGET:-}" ] && export SDKWORK_GATEWAY_PROXY_TARGET || true
[ -n "${SDKWORK_ADMIN_PROXY_TARGET:-}" ] && export SDKWORK_ADMIN_PROXY_TARGET || true
[ -n "${SDKWORK_PORTAL_PROXY_TARGET:-}" ] && export SDKWORK_PORTAL_PROXY_TARGET || true

router_validate_file "router-product-service binary" "$SDKWORK_ROUTER_BINARY"
router_validate_dir "admin site directory" "$SDKWORK_ADMIN_SITE_DIR"
router_validate_dir "portal site directory" "$SDKWORK_PORTAL_SITE_DIR"

cd "$RUNTIME_HOME"
"$SDKWORK_ROUTER_BINARY" --dry-run --plan-format json > "$PLAN_FILE"

if [ "$DRY_RUN" = '1' ]; then
  cat "$PLAN_FILE"
  exit 0
fi

if [ "$FOREGROUND" = '1' ]; then
  exec "$SDKWORK_ROUTER_BINARY"
fi

router_require_not_running "$PID_FILE"
: > "$STDOUT_LOG"
: > "$STDERR_LOG"

nohup "$SDKWORK_ROUTER_BINARY" >> "$STDOUT_LOG" 2>> "$STDERR_LOG" &
PID=$!
printf '%s\n' "$PID" > "$PID_FILE"

GATEWAY_HEALTH_URL=$(router_resolve_loopback_url "$SDKWORK_WEB_BIND" "/api/v1/health")
ADMIN_HEALTH_URL=$(router_resolve_loopback_url "$SDKWORK_WEB_BIND" "/api/admin/health")
PORTAL_HEALTH_URL=$(router_resolve_loopback_url "$SDKWORK_WEB_BIND" "/api/portal/health")

if ! router_wait_for_url "$GATEWAY_HEALTH_URL" "$WAIT_SECONDS" "$PID" \
  || ! router_wait_for_url "$ADMIN_HEALTH_URL" "$WAIT_SECONDS" "$PID" \
  || ! router_wait_for_url "$PORTAL_HEALTH_URL" "$WAIT_SECONDS" "$PID"; then
  RUNTIME_EXITED=0
  if ! router_is_pid_running "$PID"; then
    RUNTIME_EXITED=1
  fi
  router_tail_log "$STDOUT_LOG"
  router_tail_log "$STDERR_LOG"
  router_stop_pid "$PID" "$WAIT_SECONDS" 1 || true
  rm -f "$PID_FILE"
  if [ "$RUNTIME_EXITED" = '1' ]; then
    router_die "production runtime exited before health checks completed; see startup log above"
  fi
  router_die "router-product-service failed health checks on $SDKWORK_WEB_BIND"
fi

router_log "started router-product-service (pid=$PID)"
router_startup_summary \
  'production release' \
  '1' \
  "$SDKWORK_WEB_BIND" \
  "$SDKWORK_GATEWAY_BIND" \
  "$SDKWORK_ADMIN_BIND" \
  "$SDKWORK_PORTAL_BIND" \
  '' \
  '' \
  "$STDOUT_LOG" \
  "$STDERR_LOG"
