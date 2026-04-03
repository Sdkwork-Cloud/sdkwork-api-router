#!/usr/bin/env sh

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# shellcheck source=bin/lib/runtime-common.sh
. "$SCRIPT_DIR/lib/runtime-common.sh"

REPO_ROOT=$(router_repo_root "$SCRIPT_DIR")
DEV_HOME=$(router_default_dev_home "$REPO_ROOT")
DRY_RUN=0
WAIT_SECONDS=30
FORCE_MODE=1

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --wait-seconds)
      [ "$#" -ge 2 ] || router_die "--wait-seconds requires a value"
      WAIT_SECONDS="$2"
      shift 2
      ;;
    --graceful-only)
      FORCE_MODE=0
      shift
      ;;
    *)
      router_die "unknown option: $1"
      ;;
  esac
done

PID_FILE="$DEV_HOME/run/start-workspace.pid"
STOP_FILE="$DEV_HOME/run/start-workspace.stop"
STATE_FILE="$DEV_HOME/run/start-workspace.state.env"
STDOUT_LOG="$DEV_HOME/log/start-workspace.stdout.log"
STDERR_LOG="$DEV_HOME/log/start-workspace.stderr.log"

if router_is_windows; then
  PS_SCRIPT="$(router_windows_path "$SCRIPT_DIR/stop-dev.ps1")"
  set --
  [ "$DRY_RUN" = '1' ] && set -- "$@" -DryRun
  [ "$WAIT_SECONDS" != '30' ] && set -- "$@" -WaitSeconds "$WAIT_SECONDS"
  [ "$FORCE_MODE" = '0' ] && set -- "$@" -GracefulOnly
  exec powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$PS_SCRIPT" "$@"
fi

if [ "$DRY_RUN" = '1' ]; then
  router_log "would stop development workspace using pid file $PID_FILE and stop file $STOP_FILE"
  exit 0
fi

if ! [ -f "$PID_FILE" ]; then
  rm -f "$STOP_FILE"
  router_remove_managed_state "$STATE_FILE"
  router_log "pid file not found, nothing to stop: $PID_FILE"
  exit 0
fi

PID=$(router_get_running_pid "$PID_FILE" "$STATE_FILE")
if [ -z "$PID" ]; then
  rm -f "$PID_FILE"
  rm -f "$STOP_FILE"
  router_remove_managed_state "$STATE_FILE"
  router_log "process already stopped, removed stale pid file: $PID_FILE"
  exit 0
fi

: > "$STOP_FILE"

if router_wait_for_pid_exit "$PID" "$WAIT_SECONDS"; then
  rm -f "$PID_FILE"
  rm -f "$STOP_FILE"
  router_remove_managed_state "$STATE_FILE"
  router_log "stopped development workspace pid=$PID"
  exit 0
fi

if [ "$FORCE_MODE" != '1' ]; then
  router_tail_log "$STDOUT_LOG"
  router_tail_log "$STDERR_LOG"
  router_die "failed to stop development workspace pid=$PID"
fi

router_log "graceful stop timed out for development workspace pid=$PID, falling back to process termination"
if ! router_stop_pid "$PID" "$WAIT_SECONDS" "$FORCE_MODE"; then
  router_tail_log "$STDOUT_LOG"
  router_tail_log "$STDERR_LOG"
  router_die "failed to stop development workspace pid=$PID"
fi

rm -f "$PID_FILE"
rm -f "$STOP_FILE"
router_remove_managed_state "$STATE_FILE"
router_log "stopped development workspace pid=$PID"
