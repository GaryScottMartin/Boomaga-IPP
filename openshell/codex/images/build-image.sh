#!/usr/bin/env bash

set -euo pipefail

sandbox_name=${1:-boomaga-codex}
image_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)

if openshell sandbox get "$sandbox_name" >/dev/null 2>&1; then
    printf 'Deleting existing sandbox: %s\n' "$sandbox_name"
    openshell sandbox delete "$sandbox_name"
fi

printf 'Creating %s from %s\n' "$sandbox_name" "$image_dir"

openshell sandbox create \
    --name "$sandbox_name" \
    --from "$image_dir" \
    -- codex
