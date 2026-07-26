# Codex OpenShell notes

This directory contains the tracked policy and bootstrap scripts used to create a
fresh Codex sandbox for Boomaga-IPP.

## Sandbox creation and native build dependencies

`BIPP-codex-start.sh` is the tracked host entry point (and may be symlinked from
`~/bin`). It invokes `create-bipp-sandbox--Codex.sh`, which recreates the selected
sandbox, installs Codex and Rust, clones the repository, and provisions the
native dependencies required by `cargo check --workspace`.

Native packages are downloaded without sandbox root and extracted into
`/sandbox/.local/bipp-native-root`: pkg-config, GLib, Cairo, Poppler GLib, QPDF,
and libclang development packages, including dependencies resolved by apt. The
script persists the pkg-config sysroot, library path, and libclang path in
`/sandbox/.bashrc`.

Run the destructive, self-cleaning smoke test from the repository root on the
OpenShell host:

```bash
BIPP_VERIFY=1 ./openshell/codex/create-bipp-sandbox--Codex.sh
```

A successful provisioning run prints `NATIVE_DEPS_OK`. Follow it in a retained
sandbox with `cargo check --workspace` to establish the compiler baseline.

## GitHub authentication

`GITHUB_TOKEN` appears as a placeholder inside the sandbox. This is intentional:
OpenShell substitutes the real fine-grained PAT at its gateway. Never print or
persist the token value. The PAT has repository data/push access but intentionally
does not have user-profile privileges. Consequently, `gh auth status` and REST
requests such as `GET /user` are not valid authentication tests.

## Git transport

Keep the `.git` suffix on the remote URL so the enforced `github_git` rules match:

```text
https://github.com/GaryScottMartin/Boomaga-IPP.git
```

For non-interactive authenticated Git operations—including `fetch`, `pull`,
`push`, and remote-branch deletion—pass the injected token through `GIT_ASKPASS`
or a command-scoped credential helper. Do not place it in the remote URL or
commit it to disk.
`git ls-remote` is the authoritative read-only transport check.

## REST API

Use `gh api` with explicit REST paths under:

```text
/repos/GaryScottMartin/Boomaga-IPP/**
```

The policy pattern permits nested repository endpoints but may reject the bare
repository-root path. GraphQL is not allowed. A response containing the
`X-Openshell-Policy` header or JSON fields such as `"error":"policy_denied"` and
`"rule_missing"` is a policy mismatch at the OpenShell gateway, not evidence of
an invalid PAT. When a required REST path is missing, update the active policy from the
originating OpenShell host, outside the sandbox. Host-side policy changes can
take effect for the running sandbox; recreating it is not inherently required.
Editing a repository copy from inside the sandbox does not update the host's
active policy.
