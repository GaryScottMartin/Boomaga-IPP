#!/usr/bin/env bash
set -euo pipefail

SANDBOX="${CODEX_SANDBOX:-codex-dev}"
PROJECT_DIR="${1:-/sandbox/workspace}"

openshell sandbox exec \
  -n "$SANDBOX" \
  --tty \
  -- \
  bash -lc "
    set -e
    /sandbox/tools/bootstrap.sh
    mkdir -p '$PROJECT_DIR'
    cd '$PROJECT_DIR'
    exec codex
  "
