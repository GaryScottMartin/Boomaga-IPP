#!/usr/bin/env bash

set -euo pipefail

readonly GATEWAY_DIR="$HOME/Applications/openshell-gateway-0.0.116"
readonly IMAGE_TAG="0.0.116"
readonly PROJECT_NAME="openshell-gateway"
readonly SANDBOX_NETWORK="openshell-docker"

readonly STARTUP_TIMEOUT=120
readonly POLL_INTERVAL=2

die() {
    printf 'Error: %s\n' "$*" >&2
    exit 1
}

command -v docker >/dev/null 2>&1 ||
    die "'docker' is not available in PATH"

command -v konsole >/dev/null 2>&1 ||
    die "'konsole' is not available in PATH"

command -v curl >/dev/null 2>&1 ||
    die "'curl' is not available in PATH"

[[ -d $GATEWAY_DIR ]] ||
    die "gateway directory does not exist: $GATEWAY_DIR"

[[ -f $GATEWAY_DIR/docker-compose.yml ]] ||
    die "Compose file does not exist: $GATEWAY_DIR/docker-compose.yml"

[[ -f $GATEWAY_DIR/docker-compose.override.yml ]] ||
    die "Compose override does not exist: $GATEWAY_DIR/docker-compose.override.yml"

[[ -f $GATEWAY_DIR/gateway.toml ]] ||
    die "gateway configuration does not exist: $GATEWAY_DIR/gateway.toml"

sandbox_host_ip=$(
    docker network inspect "$SANDBOX_NETWORK" \
        --format '{{(index .IPAM.Config 0).Gateway}}' \
        2>/dev/null
) || die "Docker network does not exist: $SANDBOX_NETWORK"

[[ $sandbox_host_ip =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]] ||
    die "invalid gateway address for $SANDBOX_NETWORK: $sandbox_host_ip"

readonly sandbox_host_ip

printf 'OpenShell gateway directory: %s\n' "$GATEWAY_DIR"
printf 'OpenShell version: %s\n' "$IMAGE_TAG"
printf 'Sandbox callback address: %s:8080\n' "$sandbox_host_ip"

IMAGE_TAG="$IMAGE_TAG" \
OPENSHELL_SANDBOX_HOST_IP="$sandbox_host_ip" \
docker compose \
    --project-name "$PROJECT_NAME" \
    --project-directory "$GATEWAY_DIR" \
    config --quiet ||
    die "Compose configuration validation failed"

# Variables in this single-quoted program expand in the new Konsole shell.
# shellcheck disable=SC2016
konsole \
    --new-tab \
    --workdir "$GATEWAY_DIR" \
    -e bash -lc '
        set -uo pipefail

        gateway_dir=$1
        image_tag=$2
        project_name=$3
        sandbox_host_ip=$4
        startup_timeout=$5
        poll_interval=$6

        export IMAGE_TAG="$image_tag"
        export OPENSHELL_SANDBOX_HOST_IP="$sandbox_host_ip"

        docker compose \
            --project-name "$project_name" \
            --project-directory "$gateway_dir" \
            up -d

        status=$?

        if (( status != 0 )); then
            printf "\nGateway startup failed with status %d.\n" \
                "$status" >&2
            exec bash -l
        fi

        printf "Waiting for the OpenShell gateway"

        elapsed=0
        until curl -fsS \
            http://127.0.0.1:8081/readyz \
            >/dev/null 2>&1
        do
            if (( elapsed >= startup_timeout )); then
                printf "\nGateway did not become ready within %d seconds.\n" \
                    "$startup_timeout" >&2

                docker compose \
                    --project-name "$project_name" \
                    --project-directory "$gateway_dir" \
                    logs --tail=80 gateway

                exec bash -l
            fi

            printf "."
            sleep "$poll_interval"
            (( elapsed += poll_interval ))
        done

        printf " ready.\n"

        docker compose \
            --project-name "$project_name" \
            --project-directory "$gateway_dir" \
            ps

        printf "\nGateway health:\n"
        curl -fsS http://127.0.0.1:8081/readyz
        printf "\n"

        exec bash -l
    ' bash \
        "$GATEWAY_DIR" \
        "$IMAGE_TAG" \
        "$PROJECT_NAME" \
        "$sandbox_host_ip" \
        "$STARTUP_TIMEOUT" \
        "$POLL_INTERVAL"
