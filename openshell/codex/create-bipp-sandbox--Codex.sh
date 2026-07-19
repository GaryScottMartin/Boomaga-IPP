#!/bin/bash
#
# Host-side helper — run this on the OpenShell host to create a disposable
# BIPP sandbox, clone the repository into /sandbox/BIPP, authenticate Codex
# against a ChatGPT account when necessary, and launch Codex in the project.
#
# Assumptions:
#   * NVIDIA OpenShell 0.0.86 with a running local gateway.
#   * A gateway provider named "github" already exists.
#   * ChatGPT device-code login is enabled for the account.
#   * Run from the repository root so the policy path below resolves.
#
# USAGE:
#   ./openshell/codex/create-bipp-sandbox.sh [sandbox-name]
#   BIPP_VERIFY=1 ./openshell/codex/create-bipp-sandbox.sh
#
# A sandbox is disposable and must be recreated after a gateway restart.

set -euo pipefail

DIR="/sandbox/BIPP"
REPO_URL="https://github.com/GaryScottMartin/Boomaga-IPP.git"
POLICY="./openshell/codex/BIPP-project-policy--Codex.yaml"
GITHUB_PROVIDER="github-BIPP"

# Keep the .git suffix: the project policy may allow this exact URL only.
CLONE="[ -d '$DIR/.git' ] || git clone '$REPO_URL' '$DIR'"

if [ -n "${BIPP_VERIFY:-}" ]; then
  NAME="BIPP-codex-verify"
  EXTRA=(--no-keep)
  ENTRY="$CLONE; cd '$DIR'; echo \"PWD=\$(pwd)\"; test -d .git && echo GIT_OK; command -v codex >/dev/null && echo CODEX_OK; codex --version"
else
  NAME="${1:-BIPP-codex}"
  EXTRA=()

  # Fresh sandboxes normally have no ChatGPT session. Check first so this also
  # works if authentication is restored by another mechanism in the future.
  ENTRY="$CLONE; cd '$DIR'; if ! codex login status >/dev/null 2>&1; then echo 'Codex authentication required; complete the device-code flow in your browser.'; codex login --device-auth; fi; exec codex"
fi

exec openshell sandbox create \
  --name "$NAME" \
  --policy "$POLICY" \
  --provider "$GITHUB_PROVIDER" \
  "${EXTRA[@]}" \
  -- bash -lc "$ENTRY"
