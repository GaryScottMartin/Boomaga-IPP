# Codex OpenShell notes

This directory contains the tracked policy, image definition, and host-side
scripts used to manage the persistent Boomaga-IPP Codex sandbox.

## Host entry point

`bipp-codex-start.sh` is the tracked host entry point and may be symlinked from
`~/bin`. It ensures that the OpenShell gateway is available and then invokes
`create-bipp-sandbox.sh` in a new Konsole tab.

The host must resolve `openshell` to a CLI version compatible with the gateway.
The current configuration uses OpenShell 0.0.116 for both.

## Sandbox lifecycle

The sandbox manager supports these operations:

```bash
create-bipp-sandbox.sh
create-bipp-sandbox.sh --fresh
create-bipp-sandbox.sh --force-fresh
```

A normal launch:

1. Creates `bipp-codex` when it does not exist.
2. Starts it when it exists but is stopped.
3. Reuses it when it is already running.
4. Clones the Boomaga-IPP repository when the checkout is absent.
5. Fetches `origin` and fast-forwards to `origin/main` only when the working
   tree is clean and the histories are compatible.
6. Preserves the checkout and warns when it is dirty, locally ahead, or
   diverged.
7. Starts Codex in `/sandbox/BIPP`.

The sandbox uses `sleep infinity` as its persistent main process. Codex is run
interactively through `openshell sandbox exec`, so exiting Codex does not delete
the sandbox or its checkout.

### Fresh rebuild

`--fresh` verifies that the sandbox checkout is clean and that its `HEAD` is
contained in `origin/main` before deleting and rebuilding the sandbox.

```bash
create-bipp-sandbox.sh --fresh
```

`--force-fresh` bypasses that protection. Use it only after independently
confirming that all sandbox work is recoverable from the online repository or
another backup:

```bash
create-bipp-sandbox.sh --force-fresh
```

## Image and native build dependencies

`image/build-image.sh` performs a deliberately fresh build with:

```text
docker build --pull --no-cache
```

The image contains Codex, Rust, Cargo, rustfmt, Clippy, pkg-config, GLib, Cairo,
Poppler GLib, QPDF, and libclang. A missing sandbox and an explicit fresh
rebuild invoke this image build; an ordinary resume does not rebuild the image.

The Boomaga-IPP repository is not baked into the image. It is cloned from the
online repository when the sandbox is created, keeping `origin/main` as the
authoritative source.

## Verification

Run the self-cleaning verification sandbox from the repository root:

```bash
BIPP_VERIFY=1 ./openshell/codex/create-bipp-sandbox.sh
```

Verification performs a fresh uncached image build, creates
`bipp-codex-verify`, clones the current online repository, checks Codex, Rust,
and the native dependencies, and runs `cargo check`.

Success is indicated by:

```text
GIT_OK
CODEX_OK
RUST_OK
CARGO_CHECK_OK
```

The temporary verification sandbox is deleted afterward, including when a
post-creation verification command fails.

## GitHub authentication

The gateway provider is named `github-BIPP`. `GITHUB_TOKEN` appears as a
placeholder inside the sandbox. This is intentional: OpenShell substitutes the
real fine-grained PAT at its gateway. Never print or persist the token value.

The PAT has repository data and push access but intentionally lacks
user-profile privileges. Consequently, `gh auth status` and REST requests such
as `GET /user` are not valid authentication tests.

## Git transport

Keep the `.git` suffix on the remote URL so the enforced `github_git` policy
rules match:

```text
https://github.com/GaryScottMartin/Boomaga-IPP.git
```

For non-interactive authenticated Git operations—including `fetch`, `pull`,
`push`, and remote-branch deletion—pass the injected token through
`GIT_ASKPASS` or a command-scoped credential helper. Do not place it in the
remote URL or commit it to disk.

`git ls-remote` is the authoritative read-only transport test.

## REST API

Use `gh api` with explicit REST paths under:

```text
/repos/GaryScottMartin/Boomaga-IPP/**
```

The policy pattern permits nested repository endpoints but may reject the bare
repository-root path. GraphQL is not allowed.

A response containing the `X-Openshell-Policy` header or JSON fields such as
`"error":"policy_denied"` and `"rule_missing"` is a policy mismatch at the
OpenShell gateway, not evidence of an invalid PAT.

When a required REST path is missing, update the live sandbox policy with the
host-side OpenShell CLI or perform a safe fresh rebuild. Editing the policy file
inside the sandbox does not change the policy currently enforced by the
gateway.

## Gateway requirements

The local Docker-backed gateway must provide:

- OpenShell 0.0.116;
- the `github-BIPP` provider;
- gateway-minted sandbox JWT authentication;
- a callback binding reachable from the `openshell-docker` network; and
- persistent gateway state and JWT signing keys.

For this single-user local gateway, sandbox JWTs use `ttl_secs = 0` so a
restarted sandbox supervisor can reuse its bootstrap credential. The gateway's
host-facing API remains bound to loopback, while port 8080 is also bound to the
specific Docker-network gateway address used by `host.openshell.internal`.

Do not commit gateway JWT signing keys or provider credentials to this
repository.
