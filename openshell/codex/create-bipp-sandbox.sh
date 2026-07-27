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
POLICY="./openshell/codex/BIPP-project-policy.yaml"
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

# Download and extract Boomaga-IPP's Ubuntu build dependencies without requiring
# sandbox root. apt resolves transitive dependencies against the base image's
# installed package database, then unpacks downloads into a writable sysroot.
NATIVE_ROOT="/sandbox/.local/bipp-native-root"
APT_STATE="/sandbox/.cache/bipp-apt"
NATIVE_PACKAGES="pkgconf libglib2.0-dev libcairo2-dev libpoppler-glib-dev libqpdf-dev libclang-dev"
INSTALL_NATIVE="install_native_dependencies() { . /etc/os-release || return 1; codename=\"\${VERSION_CODENAME:-noble}\"; mkdir -p '$APT_STATE/lists/partial' '$APT_STATE/cache/archives/partial' '$NATIVE_ROOT' || return 1; : > '$APT_STATE/status' || return 1; : > '$APT_STATE/extended_states' || return 1; printf 'deb http://archive.ubuntu.com/ubuntu %s main universe\ndeb http://archive.ubuntu.com/ubuntu %s-updates main universe\ndeb http://security.ubuntu.com/ubuntu %s-security main universe\n' \"\$codename\" \"\$codename\" \"\$codename\" > '$APT_STATE/sources.list' || return 1; printf '%s\n' 'Dir::Etc::Main \"/dev/null\";' 'Dir::Etc::Parts \"/dev/null\";' 'Dir::Etc::sourcelist \"$APT_STATE/sources.list\";' 'Dir::Etc::sourceparts \"/dev/null\";' 'Dir::State::status \"$APT_STATE/status\";' 'Dir::State::extended_states \"$APT_STATE/extended_states\";' 'Dir::State::lists \"$APT_STATE/lists\";' 'Dir::Cache \"$APT_STATE/cache\";' 'Dir::Cache::archives \"$APT_STATE/cache/archives\";' 'APT::Sandbox::User \"sandbox\";' 'APT::Get::List-Cleanup \"0\";' > '$APT_STATE/apt.conf' || return 1; apt_private() { APT_CONFIG='$APT_STATE/apt.conf' apt-get \"\$@\"; }; apt_private update || return 1; rm -f '$APT_STATE/cache/archives/'*.deb || return 1; apt_private --download-only --no-install-recommends -y install $NATIVE_PACKAGES || return 1; for package in '$APT_STATE/cache/archives/'*.deb; do [ -f \"\$package\" ] || return 1; dpkg-deb --extract \"\$package\" '$NATIVE_ROOT' || return 1; done; }; if install_native_dependencies; then export PATH='$NATIVE_ROOT/usr/bin':\"\$PATH\"; export PKG_CONFIG_SYSROOT_DIR='$NATIVE_ROOT'; export PKG_CONFIG_LIBDIR='$NATIVE_ROOT/usr/lib/x86_64-linux-gnu/pkgconfig:$NATIVE_ROOT/usr/lib/pkgconfig:$NATIVE_ROOT/usr/share/pkgconfig'; export LD_LIBRARY_PATH='$NATIVE_ROOT/usr/lib/x86_64-linux-gnu:$NATIVE_ROOT/usr/lib'\${LD_LIBRARY_PATH:+:\"\$LD_LIBRARY_PATH\"}; LIBCLANG_FILE=\"\$(find '$NATIVE_ROOT/usr/lib' -name 'libclang.so*' -print -quit)\"; [ -n \"\$LIBCLANG_FILE\" ] || { echo 'libclang was not found after package extraction.' >&2; false; }; export LIBCLANG_PATH=\"\${LIBCLANG_FILE%/*}\"; hash -r; pkg-config --atleast-version=2.56 glib-2.0 && pkg-config --exists gio-2.0 gobject-2.0 cairo cairo-gobject poppler-glib && pkg-config --atleast-version=10.6.3 libqpdf; echo \"Native build dependencies installed; pkg-config \$(pkg-config --version).\"; else echo 'WARNING: native build-dependency installation failed; continuing into Codex.' >&2; fi"

# The sandbox's default .bashrc replaces PATH rather than extending it. Persist
# the user-local toolchain and native sysroot for later login shells.
CONFIGURE_SHELL="grep -qF '# Boomaga-IPP toolchain environment' /sandbox/.bashrc || printf '\n%s\n%s\n%s\n%s\n' '# Boomaga-IPP toolchain environment' 'export RUSTUP_HOME=/sandbox/.rustup' 'export CARGO_HOME=/sandbox/.cargo' 'export PATH=\"/sandbox/.cargo/bin:/sandbox/.local/bin:/sandbox/.venv/bin:/usr/local/bin:/usr/bin:/bin\"' >> /sandbox/.bashrc"
CONFIGURE_NATIVE="grep -qF '# Boomaga-IPP native build environment' /sandbox/.bashrc || { libclang_file=\"\$(find '$NATIVE_ROOT/usr/lib' -name 'libclang.so*' -print -quit 2>/dev/null)\"; if [ -n \"\$libclang_file\" ]; then libclang_path=\"\${libclang_file%/*}\"; printf '\n%s\n%s\n%s\n%s\n%s\n%s\n' '# Boomaga-IPP native build environment' 'export PATH=\"/sandbox/.local/bipp-native-root/usr/bin:\$PATH\"' 'export PKG_CONFIG_SYSROOT_DIR=/sandbox/.local/bipp-native-root' 'export PKG_CONFIG_LIBDIR=/sandbox/.local/bipp-native-root/usr/lib/x86_64-linux-gnu/pkgconfig:/sandbox/.local/bipp-native-root/usr/lib/pkgconfig:/sandbox/.local/bipp-native-root/usr/share/pkgconfig' 'export LD_LIBRARY_PATH=\"/sandbox/.local/bipp-native-root/usr/lib/x86_64-linux-gnu:/sandbox/.local/bipp-native-root/usr/lib\${LD_LIBRARY_PATH:+:\$LD_LIBRARY_PATH}\"' \"export LIBCLANG_PATH=\$libclang_path\" >> /sandbox/.bashrc; fi; }"
CONFIGURE_BINDGEN="clang_resource_header=\"\$(find '$NATIVE_ROOT/usr/lib' -path '*/lib/clang/*/include/stddef.h' -print -quit 2>/dev/null)\"; [ -n \"\$clang_resource_header\" ] || { echo 'Clang resource headers were not found after package extraction.' >&2; false; }; clang_resource_dir=\"\${clang_resource_header%/include/stddef.h}\"; export BINDGEN_EXTRA_CLANG_ARGS=\"-resource-dir=\$clang_resource_dir --sysroot=$NATIVE_ROOT\"; grep -qF 'export BINDGEN_EXTRA_CLANG_ARGS=' /sandbox/.bashrc || printf '%s\n' \"export BINDGEN_EXTRA_CLANG_ARGS='\$BINDGEN_EXTRA_CLANG_ARGS'\" >> /sandbox/.bashrc"
VERIFY_WORKSPACE="test -n \"\$BINDGEN_EXTRA_CLANG_ARGS\"; cargo check --workspace; echo WORKSPACE_CHECK_OK"

if [ -n "${BIPP_VERIFY:-}" ]; then
  NAME="BIPP-codex-verify"
  EXTRA=(--no-keep)
  ENTRY="set -e; trap 'status=\$?; echo \"Sandbox provisioning failed with status \$status.\" >&2; exec bash -l' ERR; $UPDATE_CODEX; $INSTALL_RUST; $INSTALL_NATIVE; $CONFIGURE_SHELL; $CONFIGURE_NATIVE; $CONFIGURE_BINDGEN; $CLONE; cd '$DIR'; echo \"PWD=\$(pwd)\"; test -d .git && echo GIT_OK; command -v codex >/dev/null && echo CODEX_OK; codex --version; command -v cargo >/dev/null && echo RUST_OK; rustc --version; cargo --version; pkg-config --modversion glib-2.0 cairo poppler-glib libqpdf; test -n \"\$LIBCLANG_PATH\" && echo NATIVE_DEPS_OK; $VERIFY_WORKSPACE"
else
  NAME="${1:-BIPP-codex}"
  EXTRA=()

  # Fresh sandboxes normally have no ChatGPT session. Check first so this also
  # works if authentication is restored by another mechanism in the future.
  ENTRY="set -e; trap 'status=\$?; echo \"Sandbox provisioning failed with status \$status.\" >&2; exec bash -l' ERR; $UPDATE_CODEX; $INSTALL_RUST; $INSTALL_NATIVE; $CONFIGURE_SHELL; $CONFIGURE_NATIVE; $CONFIGURE_BINDGEN; $CLONE; cd '$DIR'; if ! codex login status >/dev/null 2>&1; then echo 'Codex authentication required; complete the device-code flow in your browser.'; codex login --device-auth; fi; exec codex --model gpt-5.6-sol"
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
