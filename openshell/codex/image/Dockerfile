FROM ghcr.io/nvidia/openshell-community/sandboxes/base:latest

USER root

ENV DEBIAN_FRONTEND=noninteractive \
    NPM_CONFIG_UPDATE_NOTIFIER=false

# Install the current Codex CLI supplied through npm.
RUN npm install --global @openai/codex@latest \
    && command -v codex \
    && codex --version

# Native build tools and Boomaga-IPP compile-time dependencies.
RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        git \
        pkgconf \
        libglib2.0-dev \
        libcairo2-dev \
        libpoppler-glib-dev \
        libqpdf-dev \
        libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# Keep the immutable Rust installation outside /sandbox.
#
# RUSTUP_HOME contains the installed toolchains.
# RUST_INSTALL_CARGO_HOME contains rustup and its command proxies.
ENV RUSTUP_HOME=/usr/local/lib/rustup \
    RUST_INSTALL_CARGO_HOME=/usr/local/lib/rust-cargo

# Install stable Rust with rustfmt and Clippy.
RUN mkdir -p "$RUSTUP_HOME" "$RUST_INSTALL_CARGO_HOME" \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | RUSTUP_HOME="$RUSTUP_HOME" \
          CARGO_HOME="$RUST_INSTALL_CARGO_HOME" \
          sh -s -- \
            -y \
            --no-modify-path \
            --profile minimal \
            --default-toolchain stable \
            --component rustfmt \
            --component clippy \
    && chmod -R a+rX "$RUSTUP_HOME" "$RUST_INSTALL_CARGO_HOME" \
    && RUSTUP_HOME="$RUSTUP_HOME" \
       CARGO_HOME="$RUST_INSTALL_CARGO_HOME" \
       "$RUST_INSTALL_CARGO_HOME/bin/rustc" --version \
    && RUSTUP_HOME="$RUSTUP_HOME" \
       CARGO_HOME="$RUST_INSTALL_CARGO_HOME" \
       "$RUST_INSTALL_CARGO_HOME/bin/cargo" --version \
    && RUSTUP_HOME="$RUSTUP_HOME" \
       CARGO_HOME="$RUST_INSTALL_CARGO_HOME" \
       "$RUST_INSTALL_CARGO_HOME/bin/rustfmt" --version \
    && RUSTUP_HOME="$RUSTUP_HOME" \
       CARGO_HOME="$RUST_INSTALL_CARGO_HOME" \
       "$RUST_INSTALL_CARGO_HOME/bin/cargo-clippy" --version

# Locate libclang. Ubuntu's unversioned libclang.so is commonly a symbolic link,
# so use find -L rather than limiting the search to regular files.
RUN libclang_file="$(find -L /usr/lib \
        -type f \
        -name 'libclang.so' \
        -print \
        -quit)" \
    && test -n "$libclang_file" \
    && libclang_dir="${libclang_file%/*}" \
    && printf '%s\n' "$libclang_dir" \
        > /usr/local/lib/boomaga-ipp-libclang-path \
    && printf 'export LIBCLANG_PATH=%s\n' "$libclang_dir" \
        > /etc/profile.d/boomaga-ipp-libclang.sh \
    && chmod 0644 \
        /usr/local/lib/boomaga-ipp-libclang-path \
        /etc/profile.d/boomaga-ipp-libclang.sh

# Install stable command paths visible to OpenShell policy enforcement.
#
# The wrappers:
#   * use the immutable image-owned Rust toolchain;
#   * use /sandbox/.cargo for writable registry and Git caches at runtime;
#   * do not create /sandbox/.cargo during the image build;
#   * export LIBCLANG_PATH without depending on shell profile initialization.
RUN set -eu; \
    for command in \
        rustup \
        rustc \
        cargo \
        rustfmt \
        cargo-fmt \
        cargo-clippy \
        clippy-driver; \
    do \
        if [ -x "$RUST_INSTALL_CARGO_HOME/bin/$command" ]; then \
            printf '%s\n' \
                '#!/bin/sh' \
                'set -eu' \
                'export RUSTUP_HOME=/usr/local/lib/rustup' \
                'export CARGO_HOME="${CARGO_HOME:-/sandbox/.cargo}"' \
                'if [ -r /usr/local/lib/boomaga-ipp-libclang-path ]; then' \
                '    LIBCLANG_PATH="$(cat /usr/local/lib/boomaga-ipp-libclang-path)"' \
                '    export LIBCLANG_PATH' \
                'fi' \
                "exec /usr/local/lib/rust-cargo/bin/$command \"\$@\"" \
                > "/usr/local/bin/$command"; \
            chmod 0755 "/usr/local/bin/$command"; \
        fi; \
    done

# Verify the native libraries required by Boomaga-IPP.
RUN pkg-config --modversion glib-2.0 \
    && pkg-config --modversion cairo \
    && pkg-config --modversion poppler-glib \
    && pkg-config --modversion libqpdf

# Verify the public wrappers without allowing the build to create the runtime
# Cargo cache under /sandbox.
RUN CARGO_HOME=/tmp/cargo-image-verification \
    rustc --version \
    && cargo --version \
    && rustfmt --version \
    && cargo-clippy --version \
    && test -r /usr/local/lib/boomaga-ipp-libclang-path \
    && printf 'LIBCLANG_PATH=%s\n' \
        "$(cat /usr/local/lib/boomaga-ipp-libclang-path)" \
    && rm -rf /tmp/cargo-image-verification

WORKDIR /sandbox
