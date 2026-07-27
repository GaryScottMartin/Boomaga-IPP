#!/usr/bin/env bash

set -euo pipefail

gateway_dir="$HOME/Applications/openshell-gateway"

if [[ ! -d $gateway_dir ]]; then
    printf 'Error: directory does not exist: %s\n' "$gateway_dir" >&2
    exit 1
fi

konsole \
    --new-tab \
    --workdir "$gateway_dir" \
    -e bash -lc '
        docker compose up -d
        status=$?

        if (( status != 0 )); then
            printf "\ndocker-compose failed with status %d\n" "$status" >&2
        fi

        exec bash -l
    '
