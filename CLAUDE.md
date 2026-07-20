# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

See @README.md for project overview and architecture

See @docs/HANDOFF.md for current session state and active threads

### Key Inter-crate Patterns

**Configuration Management** (boomaga-config):

- Separate modules for BackendConfig (IPP service settings) and PreviewConfig
  (GUI preferences)
- Uses serde for JSON serialization with config crate
- ConfigManager provides load/save operations with default configuration

**IPP Architecture**:

- Backend service receives jobs via CUPS IPP protocol
- Job queue persists jobs with ID management (Uuid-based JobId)
- JobProcessor handles concurrent job processing with worker threads
- DNS-SD service registration for auto-discovery

**IPC Communication**:

- Unix domain sockets (`/tmp/boomaga-ipp.sock`)
- JSON message protocol for state synchronization
- Message types include job status, document info, user actions

**Layout Engine**:

- N-up algorithms (1, 2, 4, 8 pages per sheet)
- Booklet creation with page ordering
- Custom page positioning with f64 coordinates
- Transform operations for page layout

## Build System

### Standard Commands

```bash
# Build entire workspace
cargo build
# Build release (optimized)
cargo build --release
# Run all tests
cargo test
# Test specific crate
cargo test -p boomaga-core
# Check compilation without building
cargo check
# Format code
cargo fmt
# Lint with clippy
cargo clippy
```
### Development Workflow

```bash
# Edit a specific crate
cd crates/boomaga-core
cargo build
# Test core types only
cargo test -p boomaga-core
# Build preview application
cargo build -p boomaga-preview
# Build IPP backend
cargo build -p boomaga-ipp-backend
```
## Current State

The Xilem migration’s Phases A and B are complete. On Denali,
`boomaga-preview` builds, its tests pass, and the application runs; Phase C (the
Masonry PDF canvas) is next. The workspace as a whole is still not green because
`boomaga-ipp-backend` and `boomaga-ipc` have independent stub/compile gaps. See
`docs/HANDOFF.md` for the current, verified session state rather than relying on
old workspace-wide error counts.

Preview verification commands used on Denali:

```bash
cargo check -p boomaga-preview
cargo test -p boomaga-preview
cargo run -p boomaga-preview
```

## Development Prerequisites

### Rust Toolchain

- Rust 1.70+ (currently 1.87.0)
- Cargo workspace with resolver v2

### System Dependencies

- libpoppler-dev (PDF rendering)
- libpoppler-cpp-dev (PDF API)
- CUPS development libraries
- Wayland development libraries (for GUI)

## Common Development Tasks

### Adding New Types to boomaga_core

1.  Define type in `crates/boomaga-core/src/`
2.  Export at crate level in `crates/boomaga-core/src/lib.rs`
3.  Re-export in workspace root if needed
4.  Update dependencies in relevant Cargo.toml files

### Fixing Compilation Errors

1.  Check error message for crate location
2.  Verify imports from boomaga_core exist and are public
3.  Ensure Eq derives removed from f64 fields (PagePosition, TransformOperation)
4.  Check workspace dependency specifications (use direct versions, not
    workspace refs)
5.  Fix module organization and imports

### IPC Development

- Socket path defaults to `/tmp/boomaga-ipp.sock`
- Messages follow JSON serialization protocol
- Handle async I/O with tokio runtime

## Important Files

### Core Infrastructure

- `crates/boomaga-core/src/lib.rs` \- Main exports and module declarations
- `crates/boomaga-core/src/job.rs` \- Job types and queue management
- `crates/boomaga-core/src/document.rs` \- Document and page types
- `Cargo.toml` \- Workspace configuration

### Backend Service

- `crates/boomaga-ipp-backend/src/main.rs` \- Entry point and configuration
- `crates/boomaga-ipp-backend/src/server.rs` \- IPP server implementation
- `crates/boomaga-ipp-backend/src/job_processor.rs` \- Async job processing
- `crates/boomaga-ipp-backend/src/job_queue.rs` \- Job persistence and management

### GUI Application

- `crates/boomaga-preview/src/main.rs` \- Xilem application entry
- `crates/boomaga-preview/src/app.rs` \- Main application state
- `crates/boomaga-preview/src/document_renderer.rs` \- Dormant poppler/cairo renderer for Phase C integration

### Configuration

- `crates/boomaga-config/src/lib.rs` \- Configuration module exports
- `crates/boomaga-config/src/backend_config.rs` \- IPP service configuration
- `crates/boomaga-config/src/preview_config.rs` \- GUI preferences

### Layout Engine

- `crates/boomaga-layout-engine/src/n_up.rs` \- N-up layout calculations
- `crates/boomaga-layout-engine/src/booklet.rs` \- Booklet creation algorithms
- `crates/boomaga-layout-engine/src/transforms.rs` \- Page transformations

## Known Issues

- `cargo check --workspace` remains red because of backend/IPC stub and compile
  gaps; consult current compiler output before recording specific counts or causes.
- `boomaga-preview` is green through Phase B; Phase C is not yet implemented.
- `FileType` in `boomaga-core` still needs to match the accepted PDF/PWG
  Raster/JPEG formats.

Historical note: earlier reports of roughly 82 workspace errors and roughly 38
`boomaga-config` errors predate the crate-by-crate repairs and must not be treated
as current diagnostics.

## Reference Material

- Software Requirements Specification: `./docs/SW-Reqrmnts-Spec_v0.1.0_draft.pdf`
- Project plan: `./docs/PROJECT_PLAN.md`
- User manual: `README.md`
- Original Boomaga: <https://github.com/Boomaga/boomaga>
- Xilem Migration Plan: `./docs/XILEM_MIGRATION.md`
- Xilem GUI: [https://github.com/linebender/xilem](https://github.com/linebender/xilem)
- Xilem GUI Docs: [https://docs.rs/xilem/latest/xilem](https://docs.rs/xilem/latest/xilem)
- Poppler: <https://crates.io/crates/poppler>
- CUPS IPP: <https://www.cups.org/doc/spec-ipp.html>

## Session Context Management

- Handoff summaries stored in `memory/handoff-session.md`
- Auto-memory in `memory/MEMORY.md` for stable patterns and decisions
- Use `/handoff` command before session end to save context

### Claude Code setup (shipped in the repo)

Project-level Claude Code configuration lives in `.claude/` and is **version
controlled**, so a fresh clone picks it up automatically:

- `.claude/settings.json` — enabled tools and **hooks**:
  - `SessionStart` → `.claude/hooks/session-start.sh` (loads handoff context at start)
  - `PreCompact` → `.claude/hooks/pre-compact.sh` (re-injects context before autocompaction)
- `.claude/hooks/` — the hook scripts above (keep them executable: `chmod +x`)
- `.claude/commands/` — custom slash commands (e.g. `/handoff`)
- `.claude/skills/` — custom project skills

**Not** tracked (stays local per developer): `.claude/settings.local.json`
(personal permission grants) and the session-scratch files
(`context.md`, `current-task.md`, `task-history.md`, `current-bug.md`, `mode`).
See `.gitignore` for the exact split. The session-context automation is therefore
provided by the repo — no per-machine setup is required beyond the executable bit.

## Editing files in the OpenShell sandbox

- The normal patch helper may fail before reading a file with `bwrap: No
  permissions to create a new namespace`. This is an OpenShell/kernel sandbox
  limitation, not a bad path or malformed patch; retrying the same helper through
  an escalated shell does not fix it.
- When that happens, use an approved host-execution command and apply a
  well-formed unified diff with `git apply`. Run `git apply --check` first, then
  apply the identical patch. Keep paths relative to `/sandbox/BIPP` and keep the
  diff limited to the requested files.
- Do not improvise broad file rewrites. If `git apply` reports a corrupt or
  non-applicable patch, regenerate the hunk with accurate context and validate it
  again rather than forcing it.
- After editing, run `git diff --check`, inspect `git diff`, and scan for any stale
  text the edit was meant to remove.

## GitHub access from OpenShell

- `GITHUB_TOKEN` appears as a placeholder inside the sandbox. This is expected:
  OpenShell substitutes the real fine-grained PAT at its gateway. Never print or
  persist the token value.
- Do not use `gh auth status` or the `/user` endpoint to judge repository access.
  The PAT intentionally lacks user-profile privileges; test an explicit permitted
  repository subpath instead.
- Git HTTPS remotes must retain the `.git` suffix:
  `https://github.com/GaryScottMartin/Boomaga-IPP.git`.
- Use `gh api` with explicit REST subpaths for repository metadata, issues, pull
  requests, refs, and verification. GraphQL-backed `gh` commands are blocked.
  API calls must use subpaths allowed by
  `openshell/codex/BIPP-project-policy--Codex.yaml`. The repository-root path
  can be denied even when nested Git Data endpoints are permitted.
- Do not assume `jq` is installed. Prefer `gh api --jq` for response filtering and
  native `-f`/`-F` fields for simple request bodies.
- For publishing working-tree changes, make a normal local commit and use the Git
  HTTPS transport. Do not build commits through the Git Data REST API: multi-step
  blob/tree/commit requests have repeatedly returned transient HTTP 503 responses
  in this sandbox.
- A fresh sandbox has no Git identity. Before the first commit, set it repo-locally:

  ```bash
  git config --local user.name "Gary S. Martin"
  git config --local user.email "gmartin@martin-fam.net"
  ```

- A plain non-interactive `git push` cannot prompt for a username. Pass the
  gateway-injected token through a command-scoped credential helper; this neither
  prints nor persists the token:

  ```bash
  git -c 'credential.helper=!f() { echo username=x-access-token; echo password=$GITHUB_TOKEN; }; f' \
    push origin main
  ```

- Before committing, use `git status --short` and stage only the intended files.
  After pushing, verify the remote ref through REST and confirm the working tree is
  clean:

  ```bash
  gh api repos/GaryScottMartin/Boomaga-IPP/git/ref/heads/main \
    --jq '{ref: .ref, sha: .object.sha}'
  git status --short
  ```

- A 403 response containing `X-Openshell-Policy`, `policy_denied`, or
  `rule_missing` is an OpenShell routing-policy failure, not proof that the PAT
  is invalid.

See `openshell/codex/README.md` for commands and diagnostics.

## Behavioral Guidelines

1. Think before coding. State your assumptions out loud. If the request is ambiguous, ask. If a simpler approach exists, push back. Stop when you are confused, name what is unclear, do not just pick one interpretation and run.
2. Simplicity first. Write the minimum code that solves the problem. No speculative abstractions. No flexibility nobody asked for. The test: would a senior engineer call this overcomplicated.
3. Surgical changes. Touch only what the task requires. Do not improve neighboring code. Do not refactor what is not broken. Every changed line should trace back to the request.
4. Goal-driven execution. Turn vague instructions into verifiable targets before writing a line. “Add validation” becomes “write tests for invalid inputs, then make them pass.”
