# syntax=docker/dockerfile:1

FROM ghcr.io/nvidia/openshell-community/sandboxes/base:latest

USER root

ENV DEBIAN_FRONTEND=noninteractive
ENV NPM_CONFIG_UPDATE_NOTIFIER=false

# Replace the Codex CLI bundled in NVIDIA's base image with the newest version
# available when this image is built.
RUN node --version \
    && npm --version \
    && npm install -g @openai/codex@latest \
    && command -v codex \
    && codex --version

# Boomaga-IPP compile-time dependencies.
#
# The project-specific development packages match NATIVE_PACKAGES in
# create-bipp-sandbox.sh:
#
#   pkgconf
#   libglib2.0-dev
#   libcairo2-dev
#   libpoppler-glib-dev
#   libqpdf-dev
#   libclang-dev
#
# build-essential supplies the compiler, linker, libc headers, and make.
# ca-certificates and curl are needed to install Rust during the image build.
# git is needed to clone and work with the Boomaga-IPP repository.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
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

# Install the read-only Rust toolchain in an image-owned location.
#
# Do not install Rust under /opt. The current OpenShell filesystem policy does
# not make /opt accessible to the unprivileged sandbox user.
#
# RUSTUP_HOME contains installed toolchains and components.
# RUST_INSTALL_CARGO_HOME contains rustup's command proxies during the image
# build. Writable Cargo state will be stored separately under /sandbox at
# runtime.
ENV RUSTUP_HOME=/usr/local/lib/rustup
ENV RUST_INSTALL_CARGO_HOME=/usr/local/lib/rust-cargo

RUN mkdir -p "$RUSTUP_HOME" "$RUST_INSTALL_CARGO_HOME" \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
       | CARGO_HOME="$RUST_INSTALL_CARGO_HOME" \
         RUSTUP_HOME="$RUSTUP_HOME" \
         sh -s -- \
            -y \
            --no-modify-path \
            --profile minimal \
            --default-toolchain stable \
            --component rustfmt \
            --component clippy \
    && chmod -R a+rX "$RUSTUP_HOME" "$RUST_INSTALL_CARGO_HOME" \
    && CARGO_HOME="$RUST_INSTALL_CARGO_HOME" \
       "$RUST_INSTALL_CARGO_HOME/bin/rustc" --version \
    && CARGO_HOME="$RUST_INSTALL_CARGO_HOME" \
       "$RUST_INSTALL_CARGO_HOME/bin/cargo" --version \
    && CARGO_HOME="$RUST_INSTALL_CARGO_HOME" \
       "$RUST_INSTALL_CARGO_HOME/bin/rustfmt" --version \
    && CARGO_HOME="$RUST_INSTALL_CARGO_HOME" \
       "$RUST_INSTALL_CARGO_HOME/bin/cargo-clippy" --version

# Locate Ubuntu's installed libclang shared library for Rust crates that use
# bindgen.
#
# Ubuntu's libclang.so is normally a symbolic link beneath /usr/lib/llvm-*.
# The -L option follows symbolic links, avoiding the earlier failure caused by
# searching only for regular files.
RUN libclang_file="$(find -L /usr/lib \
        -name 'libclang.so' \
        -print \
        -quit)" \
    && test -n "$libclang_file" \
    && libclang_dir="${libclang_file%/*}" \
    && test -d "$libclang_dir" \
    && printf '%s\n' "$libclang_dir" \
       > /usr/local/lib/boomaga-ipp-libclang-path \
    && printf 'export LIBCLANG_PATH=%s\n' "$libclang_dir" \
       > /etc/profile.d/boomaga-ipp-libclang.sh \
    && chmod 0644 \
        /usr/local/lib/boomaga-ipp-libclang-path \
        /etc/profile.d/boomaga-ipp-libclang.sh \
    && printf 'libclang found at: %s\n' "$libclang_file" \
    && printf 'LIBCLANG_PATH=%s\n' "$libclang_dir"

# OpenShell sets the sandbox PATH independently of the image's ENV PATH:
#
#   /sandbox/.venv/bin:/usr/local/bin:/usr/bin:/bin
#
# Therefore, place command wrappers in /usr/local/bin rather than relying on
# /usr/local/lib/rust-cargo/bin being retained in PATH.
#
# At runtime, each wrapper:
#
#   1. Uses the read-only Rust toolchain baked into the image.
#   2. Stores writable Cargo state under /sandbox/.cargo.
#   3. Exposes the image's libclang directory to bindgen.
RUN for command in \
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
                'export RUSTUP_HOME=/usr/local/lib/rustup' \
                'export CARGO_HOME="${CARGO_HOME:-/sandbox/.cargo}"' \
                'if [ -r /usr/local/lib/boomaga-ipp-libclang-path ]; then' \
                '    export LIBCLANG_PATH="$(cat /usr/local/lib/boomaga-ipp-libclang-path)"' \
                'fi' \
                'mkdir -p "$CARGO_HOME"' \
                "exec /usr/local/lib/rust-cargo/bin/$command \"\$@\"" \
                > "/usr/local/bin/$command"; \
            chmod 0755 "/usr/local/bin/$command"; \
        fi; \
    done

# Verify that all Boomaga-IPP native compile-time dependencies are discoverable
# through the normal system pkg-config paths.
#
# No extracted private sysroot or custom PKG_CONFIG_LIBDIR should be necessary
# in a sandbox created from this image.
RUN pkg-config --atleast-version=2.56 glib-2.0 \
    && pkg-config --exists gio-2.0 \
    && pkg-config --exists gobject-2.0 \
    && pkg-config --exists cairo \
    && pkg-config --exists cairo-gobject \
    && pkg-config --exists poppler-glib \
    && pkg-config --atleast-version=10.6.3 libqpdf \
    && printf 'glib: %s\n' \
        "$(pkg-config --modversion glib-2.0)" \
    && printf 'cairo: %s\n' \
        "$(pkg-config --modversion cairo)" \
    && printf 'poppler-glib: %s\n' \
        "$(pkg-config --modversion poppler-glib)" \
    && printf 'libqpdf: %s\n' \
        "$(pkg-config --modversion libqpdf)"

# Verify the commands through the same /usr/local/bin wrappers that the
# unprivileged sandbox user will invoke.
RUN PATH=/usr/local/bin:/usr/bin:/bin \
    rustc --version \
    && cargo --version \
    && rustfmt --version \
    && cargo-clippy --version \
    && test -r /usr/local/lib/boomaga-ipp-libclang-path \
    && printf 'LIBCLANG_PATH: %s\n' \
        "$(cat /usr/local/lib/boomaga-ipp-libclang-path)"

WORKDIR /sandbox
