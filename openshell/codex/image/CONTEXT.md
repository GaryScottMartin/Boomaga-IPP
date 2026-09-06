# OpenShell Codex environment notes

## Current environment

- Target OpenShell version: 0.0.116.
- The CLI and local Docker-backed gateway use the same OpenShell version.
- The gateway runs in Docker Compose and is registered as `openshell-docker`.
- The gateway uses gateway-minted sandbox JWT authentication.
- Sandbox JWTs use `ttl_secs = 0` for this single-user local Docker workflow.
- The gateway API is published on host loopback for CLI access and on the
  specific `openshell-docker` network-gateway address for sandbox callbacks.
- The gateway provider used for repository access is `github-BIPP`.

## Sandbox lifecycle

- The normal sandbox is named `bipp-codex`.
- The verification sandbox is named `bipp-codex-verify`.
- OpenShell 0.0.116 provides supported `sandbox stop` and `sandbox start`
  operations.
- The sandbox uses `sleep infinity` as its persistent main process.
- Codex runs interactively through `openshell sandbox exec`.
- Exiting Codex does not delete the sandbox or its checkout.
- A normal launch resumes an existing sandbox and updates its repository only
  when the checkout is clean and can be fast-forwarded safely.
- `--fresh` verifies that sandbox work is represented in `origin/main` before
  deleting and rebuilding.
- `--force-fresh` bypasses that safety check and is reserved for recovery after
  the sandbox contents have been preserved independently.

## Image construction

- The custom image is tagged `openshell/bipp-codex:latest`.
- `build-image.sh` uses `docker build --pull --no-cache`.
- A fresh build therefore checks the base image and reinstalls Codex, Rust, and
  native dependencies instead of reusing stale Docker layers.
- A fresh build verified on September 6, 2026 installed Codex CLI 0.153.4.
- The Boomaga-IPP repository is not baked into the image; a new sandbox clones
  the current online repository into `/sandbox/BIPP`.

## Runtime toolchain

- The runtime PATH observed through `openshell sandbox exec` was:

      /sandbox/.venv/bin:/usr/local/bin:/usr/bin:/bin

- Rust was initially installed under `/opt`, but the runtime sandbox user could
  not traverse `/opt`.
- Do not rely on `.bashrc` to expose Rust through OpenShell exec.
- Rust and its command wrappers are installed under `/usr/local`.
- Wrapper commands set `RUSTUP_HOME`, `CARGO_HOME`, and `LIBCLANG_PATH` for the
  sandbox user.
