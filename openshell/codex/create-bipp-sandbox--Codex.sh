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

# Download and extract Ubuntu's pkgconf packages without requiring sandbox root.
# Package provisioning is best-effort so a mirror or apt failure cannot prevent
# the interactive Codex session from starting.
PKGCONF_ROOT="/sandbox/.local/pkgconf-root"
APT_STATE="/sandbox/.cache/bipp-apt"
PKGCONF_TMP="/sandbox/.cache/bipp-pkgconf"
INSTALL_PKGCONF="install_pkgconf() { if [ -x '$PKGCONF_ROOT/usr/bin/pkgconf' ]; then return 0; fi; . /etc/os-release || return 1; codename=\"\${VERSION_CODENAME:-noble}\"; mkdir -p '$APT_STATE/lists/partial' '$APT_STATE/cache/archives/partial' '$PKGCONF_TMP' '$PKGCONF_ROOT' || return 1; : > '$APT_STATE/status' || return 1; : > '$APT_STATE/extended_states' || return 1; printf 'deb http://archive.ubuntu.com/ubuntu %s main universe\ndeb http://archive.ubuntu.com/ubuntu %s-updates main universe\ndeb http://security.ubuntu.com/ubuntu %s-security main universe\n' \"\$codename\" \"\$codename\" \"\$codename\" > '$APT_STATE/sources.list' || return 1; printf '%s\n' 'Dir::Etc::Main \"/dev/null\";' 'Dir::Etc::Parts \"/dev/null\";' 'Dir::Etc::sourcelist \"$APT_STATE/sources.list\";' 'Dir::Etc::sourceparts \"/dev/null\";' 'Dir::State::status \"$APT_STATE/status\";' 'Dir::State::extended_states \"$APT_STATE/extended_states\";' 'Dir::State::lists \"$APT_STATE/lists\";' 'Dir::Cache \"$APT_STATE/cache\";' 'Dir::Cache::archives \"$APT_STATE/cache/archives\";' 'APT::Sandbox::User \"sandbox\";' 'APT::Get::List-Cleanup \"0\";' > '$APT_STATE/apt.conf' || return 1; apt_private() { APT_CONFIG='$APT_STATE/apt.conf' apt-get \"\$@\"; }; apt_private update || return 1; cd '$PKGCONF_TMP' || return 1; rm -f ./*.deb || return 1; apt_private download pkgconf-bin libpkgconf3 || return 1; for package in ./*.deb; do [ -f \"\$package\" ] || return 1; dpkg-deb --extract \"\$package\" '$PKGCONF_ROOT' || return 1; done; [ -x '$PKGCONF_ROOT/usr/bin/pkgconf' ]; }; if install_pkgconf; then export PATH='$PKGCONF_ROOT/usr/bin':\"\$PATH\"; export LD_LIBRARY_PATH='$PKGCONF_ROOT/usr/lib/x86_64-linux-gnu'\${LD_LIBRARY_PATH:+:\"\$LD_LIBRARY_PATH\"}; hash -r; echo \"pkg-config installed: \$(pkg-config --version)\"; else echo 'WARNING: pkg-config installation failed; continuing into Codex.' >&2; fi"

# The sandbox's default .bashrc replaces PATH rather than extending it. Persist
# the user-local Codex and Rust paths so later login shells can find both tools.
CONFIGURE_SHELL="grep -qF '# Boomaga-IPP toolchain environment' /sandbox/.bashrc || printf '\n%s\n%s\n%s\n%s\n' '# Boomaga-IPP toolchain environment' 'export RUSTUP_HOME=/sandbox/.rustup' 'export CARGO_HOME=/sandbox/.cargo' 'export PATH=\"/sandbox/.cargo/bin:/sandbox/.local/bin:/sandbox/.venv/bin:/usr/local/bin:/usr/bin:/bin\"' >> /sandbox/.bashrc"
CONFIGURE_PKGCONF="grep -qF '# Boomaga-IPP pkgconf environment' /sandbox/.bashrc || printf '\n%s\n%s\n%s\n' '# Boomaga-IPP pkgconf environment' 'export PATH=\"/sandbox/.local/pkgconf-root/usr/bin:\$PATH\"' 'export LD_LIBRARY_PATH=\"/sandbox/.local/pkgconf-root/usr/lib/x86_64-linux-gnu\${LD_LIBRARY_PATH:+:\$LD_LIBRARY_PATH}\"' >> /sandbox/.bashrc"

if [ -n "${BIPP_VERIFY:-}" ]; then
  NAME="BIPP-codex-verify"
  EXTRA=(--no-keep)
  ENTRY="set -e; trap 'status=\$?; echo \"Sandbox provisioning failed with status \$status.\" >&2; exec bash -l' ERR; $UPDATE_CODEX; $INSTALL_RUST; $INSTALL_PKGCONF; $CONFIGURE_SHELL; $CONFIGURE_PKGCONF; $CLONE; cd '$DIR'; echo \"PWD=\$(pwd)\"; test -d .git && echo GIT_OK; command -v codex >/dev/null && echo CODEX_OK; codex --version; command -v cargo >/dev/null && echo RUST_OK; rustc --version; cargo --version"
else
  NAME="${1:-BIPP-codex}"
  EXTRA=()

  # Fresh sandboxes normally have no ChatGPT session. Check first so this also
  # works if authentication is restored by another mechanism in the future.
  ENTRY="set -e; trap 'status=\$?; echo \"Sandbox provisioning failed with status \$status.\" >&2; exec bash -l' ERR; $UPDATE_CODEX; $INSTALL_RUST; $INSTALL_PKGCONF; $CONFIGURE_SHELL; $CONFIGURE_PKGCONF; $CLONE; cd '$DIR'; if ! codex login status >/dev/null 2>&1; then echo 'Codex authentication required; complete the device-code flow in your browser.'; codex login --device-auth; fi; exec codex --model gpt-5.6-sol"
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
