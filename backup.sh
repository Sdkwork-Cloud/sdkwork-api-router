#!/usr/bin/env sh

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
TARGET_SCRIPT="$SCRIPT_DIR/bin/backup.sh"

if [ ! -f "$TARGET_SCRIPT" ]; then
  printf '%s\n' "Missing managed backup entrypoint: $TARGET_SCRIPT" >&2
  exit 1
fi

exec "$TARGET_SCRIPT" "$@"
