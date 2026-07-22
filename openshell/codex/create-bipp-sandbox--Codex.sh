#!/bin/bash
#
# Host-side helper — run this on the OpenShell host to replace any unusable
# BIPP sandbox, create a fresh one, update the bundled Codex CLI, clone the
# repository into /sandbox/BIPP, authenticate Codex against a ChatGPT account
# when necessary, and launch Codex in the project.
#
# Assumptions:
#   * NVIDIA OpenShell 0.0.86 with a running local gateway.
#   * A gateway provider named "github-BIPP" already exists.
#   * ChatGPT device-code login is enabled for the account.
#   * Run from the repository root so the policy path below resolves.
#
# USAGE:
#   ./openshell/codex/create-bipp-sandbox.sh [sandbox-name]
#   BIPP_VERIFY=1 ./openshell/codex/create-bipp-sandbox--Codex.sh
#
# After a gateway restart, an existing sandbox may remain stuck in provisioning.
# OpenShell 0.0.86 cannot resurrect it, so this script deletes any sandbox with
# the selected name before creating its replacement.

set -euo pipefail

DIR="/sandbox/BIPP"
REPO_URL="https://github.com/GaryScottMartin/Boomaga-IPP.git"
POLICY="./openshell/codex/BIPP-project-policy--Codex.yaml"
GITHUB_PROVIDER="github-BIPP"

# Keep the .git suffix: the project policy may allow this exact URL only.
CLONE="[ -d '$DIR/.git' ] || git clone '$REPO_URL' '$DIR'"

# The Codex CLI bundled in the published sandbox image may be stale. The
# sandbox user cannot replace the system copy under /usr/lib/node_modules, so
# install the current CLI into a user-writable prefix and put it first in PATH.
CODEX_PREFIX="/sandbox/.local"
UPDATE_CODEX="mkdir -p '$CODEX_PREFIX/bin' && npm install -g --prefix '$CODEX_PREFIX' @openai/codex@latest && export PATH='$CODEX_PREFIX/bin':\$PATH && hash -r && command -v codex && codex --version"

# Install rustup and the stable Rust toolchain into sandbox-writable locations.
# Re-running this is safe: rustup is downloaded only when it is not already present,
# and `rustup toolchain install` updates or confirms the requested toolchain.
RUSTUP_HOME="/sandbox/.rustup"
CARGO_HOME="/sandbox/.cargo"
INSTALL_RUST="export RUSTUP_HOME='$RUSTUP_HOME' CARGO_HOME='$CARGO_HOME'; mkdir -p \"\$RUSTUP_HOME\" \"\$CARGO_HOME\"; if [ ! -x \"\$CARGO_HOME/bin/rustup\" ]; then curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --default-toolchain none; fi; export PATH=\"\$CARGO_HOME/bin:$CODEX_PREFIX/bin:\$PATH\"; rustup toolchain install stable --profile minimal --component rustfmt --component clippy; rustup default stable; hash -r; rustc --version; cargo --version"

if [ -n "${BIPP_VERIFY:-}" ]; then
  NAME="BIPP-codex-verify"
  EXTRA=(--no-keep)
  ENTRY="set -e; $UPDATE_CODEX; $INSTALL_RUST; $CLONE; cd '$DIR'; echo \"PWD=\$(pwd)\"; test -d .git && echo GIT_OK; command -v codex >/dev/null && echo CODEX_OK; codex --version; command -v cargo >/dev/null && echo RUST_OK; rustc --version; cargo --version"
else
  NAME="${1:-BIPP-codex}"
  EXTRA=()

  # Fresh sandboxes normally have no ChatGPT session. Check first so this also
  # works if authentication is restored by another mechanism in the future.
  ENTRY="set -e; $UPDATE_CODEX; $INSTALL_RUST; $CLONE; cd '$DIR'; if ! codex login status >/dev/null 2>&1; then echo 'Codex authentication required; complete the device-code flow in your browser.'; codex login --device-auth; fi; exec codex"
fi

# Ignore "not found" and similar deletion failures so first-time creation works.
# A same-named sandbox that still exists will cause the create command to fail
# rather than silently targeting the wrong sandbox.
openshell sandbox delete "$NAME" 2>/dev/null || true

exec openshell sandbox create \
  --name "$NAME" \
  --policy "$POLICY" \
  --provider "$GITHUB_PROVIDER" \
  "${EXTRA[@]}" \
  -- bash -lc "$ENTRY"
