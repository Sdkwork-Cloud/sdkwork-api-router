#!/usr/bin/env sh

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# shellcheck source=bin/lib/runtime-common.sh
. "$SCRIPT_DIR/lib/runtime-common.sh"

if router_is_windows; then
  PS_SCRIPT="$(router_windows_path "$SCRIPT_DIR/install.ps1")"
  exec powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$PS_SCRIPT" "$@"
fi

exec node "$SCRIPT_DIR/router-ops.mjs" install "$@"
