#!/usr/bin/env bash

set -euo pipefail

image_dir=$(
    cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
    pwd -P
)

sandbox_name=${1:-boomaga-codex}

printf 'Image directory: %s\n' "$image_dir"

openshell sandbox create \
    --name "$sandbox_name" \
    --from "$image_dir" \
    -- codex
