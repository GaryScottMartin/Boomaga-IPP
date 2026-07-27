#!/usr/bin/env bash
#
# Create a fresh Boomaga-IPP Codex sandbox from the custom image, attach the
# project policy and GitHub provider, clone the repository, authenticate Codex
# when necessary, and launch Codex in the project directory.
#
# Run this script from the Boomaga-IPP repository root.
#
# Usage:
#   ./openshell/codex/create-bipp-sandbox.sh
#   ./openshell/codex/create-bipp-sandbox.sh custom-sandbox-name
#
# Verification mode:
#   BIPP_VERIFY=1 ./openshell/codex/create-bipp-sandbox.sh
#
# Assumptions:
#   * NVIDIA OpenShell 0.0.86 with a running local gateway.
#   * The gateway already has a provider named "github-BIPP".
#   * The custom Dockerfile and policy are located in:
#       openshell/codex/image/
#   * The script is started from the repository root.

set -euo pipefail

readonly DEFAULT_SANDBOX_NAME="BIPP-codex"
readonly VERIFY_SANDBOX_NAME="BIPP-codex-verify"

readonly SANDBOX_NAME="${1:-$DEFAULT_SANDBOX_NAME}"

readonly SCRIPT_DIR="$(
    cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &&
        pwd -P
)"

readonly IMAGE_DIR="$SCRIPT_DIR/image"
readonly POLICY_FILE="$IMAGE_DIR/BIPP-project-policy.yaml"

readonly GITHUB_PROVIDER="github-BIPP"
readonly REPOSITORY_URL="https://github.com/GaryScottMartin/Boomaga-IPP.git"
readonly SANDBOX_PROJECT_DIR="/sandbox/BIPP"

die() {
    printf 'Error: %s\n' "$*" >&2
    exit 1
}

command -v openshell >/dev/null 2>&1 ||
    die "'openshell' is not available in PATH."

[[ -d "$IMAGE_DIR" ]] ||
    die "image directory does not exist: $IMAGE_DIR"

[[ -f "$IMAGE_DIR/Dockerfile" ]] ||
    die "Dockerfile does not exist: $IMAGE_DIR/Dockerfile"

[[ -f "$POLICY_FILE" ]] ||
    die "policy file does not exist: $POLICY_FILE"

clone_command=$(
    printf \
        '[ -d %q/.git ] || git clone %q %q' \
        "$SANDBOX_PROJECT_DIR" \
        "$REPOSITORY_URL" \
        "$SANDBOX_PROJECT_DIR"
)

if [[ -n ${BIPP_VERIFY:-} ]]; then
    sandbox_name="$VERIFY_SANDBOX_NAME"
    extra_options=(--no-keep)

    entry_command="
        set -e

        $clone_command

        cd '$SANDBOX_PROJECT_DIR'

        printf 'PWD=%s\n' \"\$PWD\"

        test -d .git
        printf 'GIT_OK\n'

        command -v codex >/dev/null
        printf 'CODEX_OK\n'
        codex --version

        command -v rustc >/dev/null
        command -v cargo >/dev/null
        command -v rustfmt >/dev/null
        command -v cargo-clippy >/dev/null

        printf 'RUST_OK\n'
        rustc --version
        cargo --version
        rustfmt --version
        cargo-clippy --version

        printf 'LIBCLANG_PATH=%s\n' \"\${LIBCLANG_PATH:-unset}\"

        pkg-config --modversion glib-2.0
        pkg-config --modversion cairo
        pkg-config --modversion poppler-glib
        pkg-config --modversion libqpdf

        cargo check

        printf 'CARGO_CHECK_OK\n'
    "
else
    sandbox_name="$SANDBOX_NAME"
    extra_options=()

    entry_command="
        set -e

        $clone_command

        cd '$SANDBOX_PROJECT_DIR'

        if ! codex login status >/dev/null 2>&1; then
            printf '%s\n' \
                'Codex authentication is required.' \
                'Complete the device-code login in your browser.'

            codex login --device-auth
        fi

        exec codex
    "
fi

printf 'Removing any existing sandbox named %s.\n' "$sandbox_name"
openshell sandbox delete "$sandbox_name" >/dev/null 2>&1 || true

printf 'Creating sandbox %s from %s.\n' "$sandbox_name" "$IMAGE_DIR"

exec openshell sandbox create \
    --name "$sandbox_name" \
    --from "$IMAGE_DIR" \
    --policy "$POLICY_FILE" \
    --provider "$GITHUB_PROVIDER" \
    "${extra_options[@]}" \
    -- bash -lc "$entry_command"
