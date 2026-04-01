#!/usr/bin/env sh

set -eu

router_log() {
  printf '[sdkwork-router] %s\n' "$*"
}

router_die() {
  printf '[sdkwork-router] ERROR: %s\n' "$*" >&2
  exit 1
}

router_script_dir() {
  CDPATH= cd -- "$(dirname -- "$1")" && pwd
}

router_repo_root() {
  SCRIPT_DIR="$1"
  CDPATH= cd -- "$SCRIPT_DIR/.." && pwd
}

router_default_install_home() {
  REPO_ROOT="$1"
  printf '%s/artifacts/install/sdkwork-api-router/current' "$REPO_ROOT"
}

router_default_dev_home() {
  REPO_ROOT="$1"
  printf '%s/artifacts/runtime/dev' "$REPO_ROOT"
}

router_binary_name() {
  NAME="$1"
  case "$(uname -s 2>/dev/null || echo unknown)" in
    CYGWIN*|MINGW*|MSYS*)
      printf '%s.exe' "$NAME"
      ;;
    *)
      printf '%s' "$NAME"
      ;;
  esac
}

router_portable_path() {
  printf '%s' "$1" | sed 's#\\#/#g'
}

router_ensure_dir() {
  mkdir -p "$1"
}

router_trim() {
  printf '%s' "$1" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//'
}

router_unquote_env_value() {
  VALUE="$1"

  case "$VALUE" in
    \"*\")
      VALUE=${VALUE#\"}
      VALUE=${VALUE%\"}
      VALUE=$(printf '%s' "$VALUE" | sed 's/\\"/"/g; s/\\\\/\\/g')
      ;;
    \'*\')
      VALUE=${VALUE#\'}
      VALUE=${VALUE%\'}
      ;;
  esac

  printf '%s' "$VALUE"
}

router_load_env_file() {
  ENV_FILE="$1"
  if [ ! -f "$ENV_FILE" ]; then
    return 0
  fi

  while IFS= read -r RAW_LINE || [ -n "$RAW_LINE" ]; do
    LINE="$(router_trim "$RAW_LINE")"
    case "$LINE" in
      ''|'#'*)
        continue
        ;;
    esac

    KEY="${LINE%%=*}"
    VALUE="${LINE#*=}"
    KEY="$(router_trim "$KEY")"
    VALUE="$(router_trim "$VALUE")"
    VALUE="$(router_unquote_env_value "$VALUE")"
    export "$KEY=$VALUE"
  done < "$ENV_FILE"
}

router_is_pid_running() {
  PID="$1"
  if [ -z "$PID" ]; then
    return 1
  fi
  kill -0 "$PID" 2>/dev/null
}

router_cleanup_stale_pid_file() {
  PID_FILE="$1"
  if [ ! -f "$PID_FILE" ]; then
    return 0
  fi

  PID="$(tr -d '[:space:]' < "$PID_FILE" 2>/dev/null || true)"
  if [ -z "$PID" ]; then
    rm -f "$PID_FILE"
    return 0
  fi

  if router_is_pid_running "$PID"; then
    return 1
  fi

  rm -f "$PID_FILE"
  return 0
}

router_require_not_running() {
  PID_FILE="$1"
  if router_cleanup_stale_pid_file "$PID_FILE"; then
    return 0
  fi

  PID="$(tr -d '[:space:]' < "$PID_FILE")"
  router_die "process already running with pid $PID (pid file: $PID_FILE)"
}

router_wait_for_pid_exit() {
  PID="$1"
  WAIT_SECONDS="$2"
  COUNTER=0
  while router_is_pid_running "$PID"; do
    if [ "$COUNTER" -ge "$WAIT_SECONDS" ]; then
      return 1
    fi
    sleep 1
    COUNTER=$((COUNTER + 1))
  done
  return 0
}

router_stop_pid() {
  PID="$1"
  WAIT_SECONDS="$2"
  FORCE_MODE="$3"

  if ! router_is_pid_running "$PID"; then
    return 0
  fi

  kill "$PID" 2>/dev/null || true
  if router_wait_for_pid_exit "$PID" "$WAIT_SECONDS"; then
    return 0
  fi

  if [ "$FORCE_MODE" != '1' ]; then
    return 1
  fi

  kill -9 "$PID" 2>/dev/null || true
  router_wait_for_pid_exit "$PID" "$WAIT_SECONDS" || return 1
}

router_resolve_loopback_url() {
  BIND_ADDR="$1"
  REQUEST_PATH="$2"
  HOST="${BIND_ADDR%:*}"
  PORT="${BIND_ADDR##*:}"

  case "$HOST" in
    ''|'0.0.0.0'|'[::]'|'::')
      HOST='127.0.0.1'
      ;;
  esac

  printf 'http://%s:%s%s' "$HOST" "$PORT" "$REQUEST_PATH"
}

router_http_ready() {
  URL="$1"
  if command -v curl >/dev/null 2>&1; then
    curl --silent --show-error --fail --max-time 3 "$URL" >/dev/null 2>&1
    return $?
  fi

  router_die "curl is required for health checks in shell scripts"
}

router_wait_for_url() {
  URL="$1"
  WAIT_SECONDS="$2"
  COUNTER=0
  while ! router_http_ready "$URL"; do
    if [ "$COUNTER" -ge "$WAIT_SECONDS" ]; then
      return 1
    fi
    sleep 1
    COUNTER=$((COUNTER + 1))
  done
  return 0
}

router_tail_log() {
  LOG_FILE="$1"
  if [ -f "$LOG_FILE" ]; then
    tail -n 60 "$LOG_FILE" 2>/dev/null || true
  fi
}

router_validate_file() {
  LABEL="$1"
  FILE_PATH="$2"
  if [ ! -f "$FILE_PATH" ]; then
    router_die "$LABEL not found: $FILE_PATH"
  fi
}

router_validate_dir() {
  LABEL="$1"
  DIR_PATH="$2"
  if [ ! -d "$DIR_PATH" ]; then
    router_die "$LABEL not found: $DIR_PATH"
  fi
}

router_default_admin_email() {
  printf '%s' 'admin@sdkwork.local'
}

router_default_admin_password() {
  printf '%s' 'ChangeMe123!'
}

router_default_portal_email() {
  printf '%s' 'portal@sdkwork.local'
}

router_default_portal_password() {
  printf '%s' 'ChangeMe123!'
}

router_log_detail() {
  LABEL="$1"
  VALUE="$2"
  router_log "  $LABEL: $VALUE"
}

router_startup_summary() {
  MODE="$1"
  UNIFIED_ACCESS_ENABLED="$2"
  WEB_BIND="$3"
  GATEWAY_BIND="$4"
  ADMIN_BIND="$5"
  PORTAL_BIND="$6"
  ADMIN_APP_URL="$7"
  PORTAL_APP_URL="$8"
  STDOUT_LOG="$9"
  STDERR_LOG="${10}"

  [ -n "$ADMIN_APP_URL" ] || ADMIN_APP_URL=$(router_resolve_loopback_url "$WEB_BIND" "/admin/")
  [ -n "$PORTAL_APP_URL" ] || PORTAL_APP_URL=$(router_resolve_loopback_url "$WEB_BIND" "/portal/")

  GATEWAY_UNIFIED_URL=$(router_resolve_loopback_url "$WEB_BIND" "/api/v1/health")
  ADMIN_UNIFIED_URL=$(router_resolve_loopback_url "$WEB_BIND" "/api/admin/health")
  PORTAL_UNIFIED_URL=$(router_resolve_loopback_url "$WEB_BIND" "/api/portal/health")
  GATEWAY_DIRECT_URL=$(router_resolve_loopback_url "$GATEWAY_BIND" "/health")
  ADMIN_DIRECT_URL=$(router_resolve_loopback_url "$ADMIN_BIND" "/admin/health")
  PORTAL_DIRECT_URL=$(router_resolve_loopback_url "$PORTAL_BIND" "/portal/health")

  router_log '------------------------------------------------------------'
  router_log "Mode: $MODE"
  router_log "Bind Summary: web=$WEB_BIND gateway=$GATEWAY_BIND admin=$ADMIN_BIND portal=$PORTAL_BIND"

  if [ "$UNIFIED_ACCESS_ENABLED" = '1' ]; then
    router_log 'Unified Access'
    router_log_detail 'Admin App' "$ADMIN_APP_URL"
    router_log_detail 'Portal App' "$PORTAL_APP_URL"
    router_log_detail 'Gateway API Health' "$GATEWAY_UNIFIED_URL"
    router_log_detail 'Admin API Health' "$ADMIN_UNIFIED_URL"
    router_log_detail 'Portal API Health' "$PORTAL_UNIFIED_URL"
  else
    router_log 'Frontend Access'
    router_log_detail 'Admin App' "$ADMIN_APP_URL"
    router_log_detail 'Portal App' "$PORTAL_APP_URL"
  fi

  router_log 'Direct Service Access'
  router_log_detail 'Gateway Service' "$GATEWAY_DIRECT_URL"
  router_log_detail 'Admin Service' "$ADMIN_DIRECT_URL"
  router_log_detail 'Portal Service' "$PORTAL_DIRECT_URL"

  router_log 'Initial Credentials'
  router_log_detail 'Admin Console' "$(router_default_admin_email) / $(router_default_admin_password)"
  router_log_detail 'Portal Console' "$(router_default_portal_email) / $(router_default_portal_password)"
  router_log_detail 'Gateway API' 'sign in through the portal and create an API key.'

  router_log 'Logs'
  router_log_detail 'STDOUT' "$STDOUT_LOG"
  router_log_detail 'STDERR' "$STDERR_LOG"
}
