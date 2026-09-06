#!/usr/bin/env bash

set -euo pipefail

image_dir=$(
    cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
    pwd -P
)
readonly image_dir

readonly image_name="${1:-openshell/bipp-codex:latest}"

command -v docker >/dev/null 2>&1 || {
    printf "Error: 'docker' is not available in PATH.\n" >&2
    exit 1
}

printf 'Building fresh image: %s\n' "$image_name"
printf 'Build context: %s\n' "$image_dir"

docker build \
    --pull \
    --no-cache \
    --tag "$image_name" \
    "$image_dir"

docker image inspect "$image_name" \
    --format 'Image={{.RepoTags}} ID={{.Id}} Created={{.Created}}'
