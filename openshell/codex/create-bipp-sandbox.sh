#!/usr/bin/env bash
#
# Manage the persistent Boomaga-IPP Codex sandbox.
#
# Usage:
#   create-bipp-sandbox.sh
#   create-bipp-sandbox.sh --fresh
#   create-bipp-sandbox.sh --force-fresh
#   create-bipp-sandbox.sh [--fresh|--force-fresh] sandbox-name
#
# Verification mode:
#   BIPP_VERIFY=1 create-bipp-sandbox.sh
#
# --fresh:
#   Delete and rebuild only after confirming that the sandbox repository is
#   clean and that its HEAD is contained in origin/main.
#
# --force-fresh:
#   Delete and rebuild even when the existing sandbox cannot be inspected.
#   Use only after independently confirming that its work is recoverable.

set -euo pipefail

readonly DEFAULT_SANDBOX_NAME="bipp-codex"
readonly VERIFY_SANDBOX_NAME="bipp-codex-verify"

SCRIPT_DIR="$(
    cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &&
        pwd -P
)"
readonly SCRIPT_DIR

readonly IMAGE_DIR="$SCRIPT_DIR/image"
readonly POLICY_FILE="$IMAGE_DIR/bipp-project-policy.yaml"
readonly IMAGE_BUILD_SCRIPT="$IMAGE_DIR/build-image.sh"
readonly SANDBOX_IMAGE="openshell/bipp-codex:latest"

readonly GITHUB_PROVIDER="github-BIPP"
readonly REPOSITORY_URL="https://github.com/GaryScottMartin/Boomaga-IPP.git"
readonly SANDBOX_PROJECT_DIR="/sandbox/BIPP"

readonly STARTUP_TIMEOUT=120
readonly POLL_INTERVAL=2

fresh=false
force_fresh=false
sandbox_name="$DEFAULT_SANDBOX_NAME"
name_was_set=false

die() {
    printf 'Error: %s\n' "$*" >&2
    exit 1
}

usage() {
    sed -n '3,19p' "${BASH_SOURCE[0]}" |
        sed 's/^# \{0,1\}//'
}

while (( $# > 0 )); do
    case $1 in
        --fresh)
            fresh=true
            ;;
        --force-fresh)
            fresh=true
            force_fresh=true
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        -*)
            die "unknown option: $1"
            ;;
        *)
            if [[ $name_was_set == true ]]; then
                die "only one sandbox name may be supplied"
            fi

            sandbox_name=$1
            name_was_set=true
            ;;
    esac

    shift
done

if [[ -n ${BIPP_VERIFY:-} ]]; then
    sandbox_name="$VERIFY_SANDBOX_NAME"

    if [[ $fresh == true ]]; then
        die "--fresh and --force-fresh are not used with BIPP_VERIFY"
    fi
fi

[[ $sandbox_name =~ ^[a-z0-9]+([a-z0-9-]*[a-z0-9])?$ ]] ||
    die "sandbox name must contain only lowercase letters, numbers, and hyphens"

command -v openshell >/dev/null 2>&1 ||
    die "'openshell' is not available in PATH"

[[ $(openshell --version) == "openshell 0.0.116" ]] ||
    die "this script requires OpenShell 0.0.116"

[[ -d $IMAGE_DIR ]] ||
    die "image directory does not exist: $IMAGE_DIR"

[[ -f $IMAGE_DIR/Dockerfile ]] ||
    die "Dockerfile does not exist: $IMAGE_DIR/Dockerfile"

[[ -f $POLICY_FILE ]] ||
    die "policy file does not exist: $POLICY_FILE"

sandbox_exists() {
    openshell sandbox get "$sandbox_name" >/dev/null 2>&1
}

sandbox_responds() {
    openshell sandbox exec \
        --name "$sandbox_name" \
        --no-tty \
        -- true >/dev/null 2>&1
}

wait_for_sandbox() {
    local elapsed=0

    printf 'Waiting for sandbox %s' "$sandbox_name"

    until sandbox_responds; do
        if (( elapsed >= STARTUP_TIMEOUT )); then
            printf '\n' >&2
            die "sandbox did not become responsive within ${STARTUP_TIMEOUT} seconds"
        fi

        printf '.'
        sleep "$POLL_INTERVAL"
        (( elapsed += POLL_INTERVAL ))
    done

    printf ' ready.\n'
}

ensure_sandbox_running() {
    if sandbox_responds; then
        printf 'Sandbox %s is already running.\n' "$sandbox_name"
        return
    fi

    printf 'Starting sandbox %s.\n' "$sandbox_name"
    openshell sandbox start "$sandbox_name"
    wait_for_sandbox
}

create_sandbox() {
    printf 'Building a fresh sandbox image.\n'
    "$IMAGE_BUILD_SCRIPT" "$SANDBOX_IMAGE"

    printf 'Creating sandbox %s from %s.\n' \
        "$sandbox_name" \
        "$SANDBOX_IMAGE"

    openshell sandbox create \
        --detach \
        --name "$sandbox_name" \
        --from "$SANDBOX_IMAGE" \
        --policy "$POLICY_FILE" \
        --provider "$GITHUB_PROVIDER" \
        --no-auto-providers \
        --no-tty \
        -- sleep infinity

    wait_for_sandbox
}

ensure_checkout_and_update() {
    local repository_script

    # Variables in this single-quoted template must expand inside the sandbox.
    # shellcheck disable=SC2016
    repository_script=$(printf '
        set -e

        repository_url=%q
        project_dir=%q

        if [[ ! -d $project_dir/.git ]]; then
            printf "Cloning %%s into %%s.\\n" \
                "$repository_url" \
                "$project_dir"

            git clone "$repository_url" "$project_dir"
            exit
        fi

        cd "$project_dir"

        if [[ -n $(git status --porcelain) ]]; then
            printf "WARNING: repository has uncommitted or untracked changes.\\n" >&2
            printf "Skipping fetch and update; sandbox contents are preserved.\\n" >&2
            git status --short --branch >&2
            exit
        fi

        git fetch --prune origin

        if git merge-base --is-ancestor HEAD origin/main; then
            git merge --ff-only origin/main
            printf "Repository is synchronized with origin/main.\\n"
        elif git merge-base --is-ancestor origin/main HEAD; then
            printf "WARNING: sandbox repository contains commits not in origin/main.\\n" >&2
            printf "Skipping update; push or otherwise preserve those commits.\\n" >&2
        else
            printf "WARNING: sandbox and origin/main have diverged.\\n" >&2
            printf "Skipping update; manual reconciliation is required.\\n" >&2
        fi
    ' "$REPOSITORY_URL" "$SANDBOX_PROJECT_DIR")

    openshell sandbox exec \
        --name "$sandbox_name" \
        --no-tty \
        -- bash -lc "$repository_script"
}

sandbox_is_safe_to_delete() {
    local safety_script

    # Variables in this single-quoted template must expand inside the sandbox.
    # shellcheck disable=SC2016
    safety_script=$(printf '
        set -e

        project_dir=%q

        if [[ ! -d $project_dir/.git ]]; then
            printf "No Git checkout exists in %%s; rebuild is safe.\\n" \
                "$project_dir"
            exit
        fi

        cd "$project_dir"

        if [[ -n $(git status --porcelain) ]]; then
            printf "Sandbox repository has uncommitted or untracked changes.\\n" >&2
            git status --short --branch >&2
            exit 20
        fi

        git fetch --prune origin

        if ! git merge-base --is-ancestor HEAD origin/main; then
            printf "Sandbox HEAD is not contained in origin/main.\\n" >&2
            git log -1 --oneline --decorate >&2
            exit 21
        fi

        printf "Sandbox work is contained in origin/main; rebuild is safe.\\n"
    ' "$SANDBOX_PROJECT_DIR")

    openshell sandbox exec \
        --name "$sandbox_name" \
        --no-tty \
        -- bash -lc "$safety_script"
}

run_verification() {
    local verification_script
    local status

    # Variables in this single-quoted template must expand inside the sandbox.
    # shellcheck disable=SC2016
    verification_script=$(printf '
        set -e

        repository_url=%q
        project_dir=%q

        git clone "$repository_url" "$project_dir"
        cd "$project_dir"

        printf "PWD=%%s\\n" "$PWD"
        printf "GIT_OK\\n"

        command -v codex
        codex --version
        printf "CODEX_OK\\n"

        command -v rustc
        command -v cargo
        command -v rustfmt
        command -v cargo-clippy

        rustc --version
        cargo --version
        rustfmt --version
        cargo-clippy --version
        printf "RUST_OK\\n"

        printf "LIBCLANG_PATH=%%s\\n" "${LIBCLANG_PATH:-unset}"

        pkg-config --modversion glib-2.0
        pkg-config --modversion cairo
        pkg-config --modversion poppler-glib
        pkg-config --modversion libqpdf

        cargo check
        printf "CARGO_CHECK_OK\\n"
    ' "$REPOSITORY_URL" "$SANDBOX_PROJECT_DIR")

    openshell sandbox delete "$sandbox_name" >/dev/null 2>&1 || true

    cleanup_verification() {
        printf 'Deleting verification sandbox %s.\n' "$sandbox_name"
        openshell sandbox delete "$sandbox_name" >/dev/null 2>&1 || true
    }

    trap cleanup_verification EXIT INT TERM

    create_sandbox

    set +e
    openshell sandbox exec \
        --name "$sandbox_name" \
        --no-tty \
        -- bash -lc "$verification_script"
    status=$?
    set -e

    cleanup_verification
    trap - EXIT INT TERM

    return "$status"
}

launch_codex() {
    exec openshell sandbox exec \
        --name "$sandbox_name" \
        --workdir "$SANDBOX_PROJECT_DIR" \
        --tty \
        -- bash -lc '
            set -e

            if ! codex login status >/dev/null 2>&1; then
                printf "%s\n" \
                    "Codex authentication is required." \
                    "Complete the device-code login in your browser."

                codex login --device-auth
            fi

            exec codex
        '
}

[[ -x $IMAGE_BUILD_SCRIPT ]] ||
    die "image build script is missing or not executable: $IMAGE_BUILD_SCRIPT"

if [[ -n ${BIPP_VERIFY:-} ]]; then
    run_verification
    exit
fi

if sandbox_exists; then
    if [[ $fresh == true ]]; then
        if [[ $force_fresh == false ]]; then
            ensure_sandbox_running

            sandbox_is_safe_to_delete ||
                die "refusing fresh rebuild; use --force-fresh only after preserving the sandbox"
        else
            printf 'WARNING: forcing deletion without repository-safety verification.\n' >&2
        fi

        printf 'Deleting sandbox %s for a fresh rebuild.\n' "$sandbox_name"
        openshell sandbox delete "$sandbox_name"
        create_sandbox
    else
        ensure_sandbox_running
    fi
else
    printf 'No sandbox named %s exists.\n' "$sandbox_name"
    create_sandbox
fi

ensure_checkout_and_update
launch_codex
