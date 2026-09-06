# Boomaga-IPP Codex sandbox image

This directory defines the custom container image and OpenShell policy used by
the persistent Boomaga-IPP Codex sandbox.

## Directory layout

```text
openshell/
└── codex/
    ├── bipp-codex-start.sh
    ├── create-bipp-sandbox.sh
    ├── README.md
    └── image/
        ├── build-image.sh
        ├── Dockerfile
        ├── bipp-project-policy.yaml
        ├── CONTEXT.md
        └── README.md
```

## Requirements

- NVIDIA OpenShell 0.0.116 CLI and gateway
- A local Docker-backed gateway
- Gateway-minted sandbox JWT authentication
- A gateway callback endpoint reachable from sandbox containers
- A configured GitHub provider named `github-BIPP`
- Docker access from the host account
- Access to `https://github.com/GaryScottMartin/Boomaga-IPP.git`

## Image contents

The Dockerfile installs:

- the current Codex CLI supplied through npm;
- the stable Rust toolchain;
- Cargo, rustfmt, and Clippy;
- native compilation tools;
- pkg-config;
- GLib and Cairo development files;
- Poppler GLib development files;
- QPDF development files; and
- libclang for bindgen.

Rust is installed outside `/sandbox`, while per-user Cargo data may be stored
under `/sandbox/.cargo`. Wrapper commands set `RUSTUP_HOME`, `CARGO_HOME`, and
`LIBCLANG_PATH` consistently for the sandbox user.

The Boomaga-IPP repository is deliberately not copied into the image. A new
sandbox clones the current online repository into `/sandbox/BIPP`.

## Image construction

`build-image.sh` builds the local image tag:

```text
openshell/bipp-codex:latest
```

It invokes:

```text
docker build --pull --no-cache
```

`--pull` checks for a newer base image. `--no-cache` ensures that commands such
as `npm install --global @openai/codex@latest`, Rust installation, and package
installation actually run during a fresh build.

Ordinary sandbox resume does not rebuild the image. A build occurs when the
normal sandbox is absent, when `--fresh` or `--force-fresh` is requested, and
when verification mode is used.

The helper can also be invoked directly:

```bash
./openshell/codex/image/build-image.sh
```

An alternative image tag may be supplied as its sole argument.

## Sandbox creation and reuse

Normally launch the environment through:

```bash
bipp-codex-start.sh
```

The launcher invokes `create-bipp-sandbox.sh`, which:

1. creates `bipp-codex` if absent;
2. starts it if stopped;
3. reuses it if already running;
4. clones the repository if `/sandbox/BIPP/.git` is absent;
5. safely fast-forwards a clean checkout to `origin/main`; and
6. runs Codex interactively in `/sandbox/BIPP`.

The sandbox's persistent main process is `sleep infinity`. Exiting Codex leaves
the sandbox available for a later launch.

## Repository safety

For an existing checkout, the launcher updates only when:

- `git status --porcelain` is empty; and
- the current `HEAD` is an ancestor of `origin/main`.

If the checkout is dirty, locally ahead, or diverged, the launcher preserves it
and reports the condition instead of resetting, cleaning, or overwriting it.

Before deleting an existing sandbox, `--fresh` additionally verifies that its
work is represented in `origin/main`:

```bash
create-bipp-sandbox.sh --fresh
```

Use the destructive override only after independently preserving the work:

```bash
create-bipp-sandbox.sh --force-fresh
```

## Policy

`bipp-project-policy.yaml` supplies the filesystem and network policy attached
when the sandbox is created. It permits the binaries and endpoints required for
Codex, GitHub Git and REST operations, npm, Rust tooling, Ubuntu packages, and
the Boomaga-IPP build.

Changing the stored YAML file does not automatically change a running
sandbox's effective policy. Apply the revised policy through the host-side
OpenShell policy commands or perform a safe fresh rebuild.

## Verification

Run:

```bash
BIPP_VERIFY=1 ./openshell/codex/create-bipp-sandbox.sh
```

Verification performs a fresh uncached build and creates a temporary
`bipp-codex-verify` sandbox. It then:

1. clones the current online repository;
2. reports the Codex version;
3. reports the Rust tool versions;
4. verifies the native pkg-config dependencies; and
5. runs `cargo check`.

A successful run ends with:

```text
CARGO_CHECK_OK
Deleting verification sandbox bipp-codex-verify.
```

Compiler warnings do not constitute a verification failure when `cargo check`
finishes successfully.

## Troubleshooting

### Policy fetch fails during provisioning

Inspect the sandbox container log. If it reports that it cannot connect to the
OpenShell server, verify that `host.openshell.internal` resolves to the Docker
network's gateway address and that the gateway's port 8080 is published on that
specific address. Binding only to host loopback is insufficient for sandbox
callbacks.

Do not expose an unauthenticated gateway on every host interface merely to fix
the callback path. Retain the loopback binding and add only the required Docker
gateway-address binding.

### Sandbox fails after a gateway or container restart

OpenShell 0.0.116 provides `sandbox stop` and `sandbox start`. The local gateway
uses non-expiring gateway-minted sandbox JWTs so a restarted supervisor can
authenticate again.

If a sandbox cannot be started or inspected, preserve its checkout before
using `--force-fresh`.

### Dockerfile changes do not appear

A normal launch resumes the existing sandbox. Apply image changes with:

```bash
create-bipp-sandbox.sh --fresh
```

The image helper deliberately disables Docker's build cache.

### Git update is skipped

Review the warning and inspect the checkout inside the sandbox. Commit and push
valid work before retrying. The launcher intentionally does not clean files,
reset branches, rebase commits, or resolve divergence automatically.

## Additional documentation

`CONTEXT.md` contains design history, implementation notes, and lessons learned
while developing the image. It is reference material for future maintenance
and for assistants modifying the environment.
