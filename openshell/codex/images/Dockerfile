# syntax=docker/dockerfile:1

FROM ghcr.io/nvidia/openshell-community/sandboxes/base:latest

USER root

RUN npm install -g @openai/codex@latest \
    && codex --version

ENV DEBIAN_FRONTEND=noninteractive

# General native-build tools plus likely Boomaga/printing dependencies.
#
# This package list should be refined against Boomaga-IPP's actual
# Cargo.toml, build.rs files, and compiler output.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        clang \
        cmake \
        curl \
        git \
        libcups2-dev \
        libssl-dev \
        lld \
        ninja-build \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install Rust system-wide so it is available regardless of which
# sandbox user OpenShell selects.
ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo
ENV PATH=/opt/cargo/bin:${PATH}

RUN mkdir -p "${RUSTUP_HOME}" "${CARGO_HOME}" \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
       | sh -s -- -y --no-modify-path --profile default --default-toolchain stable \
    && chmod -R a+rX "${RUSTUP_HOME}" "${CARGO_HOME}" \
    && rustc --version \
    && cargo --version

WORKDIR /sandbox
