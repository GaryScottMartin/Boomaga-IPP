# Boomaga-IPP OpenShell Codex Environment

This directory contains the scripts and Docker image used to create a reproducible NVIDIA OpenShell development environment for the Boomaga-IPP project.

The environment is designed so that a newly created sandbox is immediately ready for development without installing additional software.

---

## Directory Layout

```text
openshell/
└── codex/
    ├── BIPP-codex-start.sh
    ├── create-bipp-sandbox.sh
    ├── README.md
    └── image/
        ├── Dockerfile
        ├── BIPP-project-policy.yaml
        └── CONTEXT.md
```

---

## Prerequisites

* NVIDIA OpenShell 0.0.86
* A running local OpenShell Gateway
* Docker installed and functioning
* A GitHub provider already configured in the gateway
* Access to the Boomaga-IPP GitHub repository

---

## Purpose

The custom image includes everything required to build and test Boomaga-IPP:

* Current Codex CLI
* Rust toolchain
* Cargo
* rustfmt
* rustdoc
* Clippy
* Boomaga native build dependencies
* OpenShell project policy

The startup scripts then:

1. Create a fresh sandbox.
2. Attach the project policy.
3. Attach the configured GitHub provider.
4. Clone the Boomaga-IPP repository (if necessary).
5. Authenticate Codex (if required).
6. Start Codex in the project directory.

---

## Creating a Sandbox

Normally, simply run:

```bash
BIPP-codex-start.sh
```

The launcher may be executed from any working directory.

The launcher recreates the sandbox each time so that changes to the Docker image are incorporated automatically.

---

## Updating the Image

When changes are made to:

* `image/Dockerfile`
* `image/BIPP-project-policy.yaml`

simply run the normal startup script again.

The existing sandbox will be removed and recreated from the updated image.

No separate Docker build step is normally required.

Docker layer caching keeps rebuilds reasonably fast by rebuilding only the layers affected by your changes.

---

## Verifying the Environment

After a new sandbox is created, the following should succeed:

```bash
codex --version

rustc --version
cargo --version
rustfmt --version
rustdoc --version
cargo-clippy --version

cargo check
cargo test --workspace
```

A successful `cargo test --workspace` confirms that the image contains the complete toolchain and native dependencies required by Boomaga-IPP.

---

## Design Goals

This environment follows a few guiding principles:

* Sandboxes are disposable.
* All development tools are installed in the image.
* Startup scripts provision projects—they do not install software.
* Wrapper scripts configure the Rust environment instead of relying on shell startup files.
* A newly created sandbox should be immediately usable.

---

## Troubleshooting

### Sandbox stuck in "Provisioning"

OpenShell 0.0.86 may leave sandboxes in a permanent provisioning state after the local gateway restarts.

Delete the sandbox and recreate it using:

```bash
BIPP-codex-start.sh
```

---

### Dockerfile changes do not appear

The startup script recreates the sandbox each time it runs.

If an expected Dockerfile change is still missing, verify that the Docker build did not reuse an unexpected cached layer. Rebuilding without cache may be useful during troubleshooting.

---

## Additional Documentation

`image/CONTEXT.md` contains the design history, implementation notes, and lessons learned while developing this environment.

It is intended primarily as reference material for future maintenance and for AI assistants modifying the Docker image.
