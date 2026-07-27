# syntax=docker/dockerfile:1

FROM ghcr.io/nvidia/openshell-community/sandboxes/base:latest

USER root

ENV DEBIAN_FRONTEND=noninteractive

# Replace the Codex CLI bundled in NVIDIA's base image with the newest version
# available when this image is built.
RUN npm install -g @openai/codex@latest \
    && command -v codex \
    && codex --version

# Boomaga-IPP compile-time dependencies.
#
# The six project-specific development packages below match NATIVE_PACKAGES in
# create-bipp-sandbox.sh:
#
#   pkgconf
#   libglib2.0-dev
#   libcairo2-dev
#   libpoppler-glib-dev
#   libqpdf-dev
#   libclang-dev
#
# build-essential supplies the system compiler, linker, libc headers and make.
# ca-certificates and curl are needed to install Rust during the image build.
# git is used later to clone and work with the repository.
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
# Do not install it under /opt: /opt is not accessible under the current
# OpenShell filesystem policy.
ENV RUSTUP_HOME=/usr/local/lib/rustup
ENV RUST_INSTALL_CARGO_HOME=/usr/local/lib/rust-cargo

RUN mkdir -p "$RUSTUP_HOME" "$RUST_INSTALL_CARGO_HOME" \
    && CARGO_HOME="$RUST_INSTALL_CARGO_HOME" \
       curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
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

# OpenShell replaces PATH at sandbox startup with:
#
#   /sandbox/.venv/bin:/usr/local/bin:/usr/bin:/bin
#
# Install wrappers in /usr/local/bin instead of relying on an image-level PATH.
#
# At runtime:
#   * RUSTUP_HOME points to the read-only toolchain baked into the image.
#   * CARGO_HOME points to /sandbox/.cargo, where the sandbox user can maintain
#     the Cargo registry, Git checkouts and other writable Cargo state.
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
                'mkdir -p "$CARGO_HOME"' \
                "exec /usr/local/lib/rust-cargo/bin/$command \"\$@\"" \
                > "/usr/local/bin/$command"; \
            chmod 0755 "/usr/local/bin/$command"; \
        fi; \
    done

# Record libclang's library directory for bindgen-based Rust crates.
#
# The small environment script can be sourced by project tooling if bindgen
# does not discover Ubuntu's system libclang automatically.
RUN libclang_file="$(find /usr/lib -type f -name 'libclang.so*' -print -quit)" \
    && test -n "$libclang_file" \
    && libclang_dir="${libclang_file%/*}" \
    && printf 'export LIBCLANG_PATH=%s\n' "$libclang_dir" \
       > /etc/profile.d/boomaga-ipp-libclang.sh \
    && chmod 0644 /etc/profile.d/boomaga-ipp-libclang.sh \
    && printf '%s\n' "$libclang_dir" \
       > /usr/local/lib/boomaga-ipp-libclang-path

# Verify that all project-specific pkg-config modules are available through
# normal system paths. No private sysroot or custom PKG_CONFIG_LIBDIR should
# be needed in the finished image.
RUN pkg-config --atleast-version=2.56 glib-2.0 \
    && pkg-config --exists \
        gio-2.0 \
        gobject-2.0 \
        cairo \
        cairo-gobject \
        poppler-glib \
    && pkg-config --atleast-version=10.6.3 libqpdf \
    && printf 'glib: %s\n' "$(pkg-config --modversion glib-2.0)" \
    && printf 'cairo: %s\n' "$(pkg-config --modversion cairo)" \
    && printf 'poppler-glib: %s\n' "$(pkg-config --modversion poppler-glib)" \
    && printf 'libqpdf: %s\n' "$(pkg-config --modversion libqpdf)"

# Verify the commands through the same /usr/local/bin wrappers that the
# sandbox user will invoke.
RUN PATH=/usr/local/bin:/usr/bin:/bin \
    rustc --version \
    && cargo --version \
    && rustfmt --version \
    && cargo-clippy --version

WORKDIR /sandbox
