<!--
  docs/HANDOFF.md — session-to-session continuity for Claude Code (and humans).

  PURPOSE: give a cold session enough to be productive in ~30 seconds of reading.
  RULES:
    1. NO SECRETS. This file is committed. No tokens, keys, JWTs, private URLs,
       or raw command output that may contain them. (See CLAUDE.md security notes.)
    2. Curated, not raw. Summaries and decisions — never transcript dumps.
    3. Keep it CURRENT. Prune stale threads; correct anything the code has moved past.
    4. Keep it SHORT. If a section grows past a screen, it belongs in a linked doc.
    5. Verify before trusting. If this file names a file/fn/flag, confirm it still
       exists before acting on it — handoff notes go stale.
  Sections marked with <!-- ... --> comments are guidance; leave them in place.
-->

# Boomaga-IPP — Session Handoff

> **Last updated:** 2026-07-22 · **By:** Codex + Gary Scott Martin
> **Session focus:** implemented Xilem Phase D on `xilem-phase-d`; dependency,
> compile/test, and Wayland runtime verification remain pending on Denali.

---

## 1. TL;DR — where things stand
<!-- One short paragraph. What just happened, what's the single most important next step. -->
Specs **v0.2.2** now embed the code-conformant Appendix C UML (`c471a71`), so that thread is
closed. This session was tooling/hygiene, not code: reconciled `PROJECT_PLAN.md` +
`XILEM_MIGRATION.md` with the real code (both were badly stale), **vendored** the `.claude/`
handoff config so it's portable and version-controlled, and added **host-verified** OpenShell
provisioning — `openshell/create-bipp-sandbox.sh` auto-clones the repo and launches claude
(the `--from` image route was tried and abandoned; see §2). **GUI migration:** Phases A/B/C
are complete on `main`. Phase D is implemented on `xilem-phase-d`: native PDF selection,
thread-confined Poppler loading/rendering through Xilem's worker/message path, loading/error/
progress state, and an on-demand page cache. It is not yet host-verified. Separately,
`boomaga-ipp-backend` / `boomaga-ipc` still don't compile (their own stub/bug issues,
independent of the GUI).

## 2. Active threads / in progress
<!-- The heart of the file. Each item: what, state, concrete next action. Delete when done. -->
- [x] **Appendix C ↔ code (#13/#14) — CLOSED.** Specs bumped to **v0.2.2** embedding the
      code-conformant Appendix C (`c471a71`); `docs/uml/*.puml` is the living source of truth.
- [x] **Planning docs reconciled with code (`6cd8024`).** `PROJECT_PLAN.md` + `XILEM_MIGRATION.md`
      rewritten: dropped plugins / Ghostscript / D-Bus-as-IPC; formats PDF/PWG/JPEG; honest
      per-crate status. `XILEM_MIGRATION.md` now describes the *actual* broken half-migration and
      the correct Xilem 0.4 view-based model (was full of a fabricated API).
- [x] **`.claude/` config made shareable + portable (`99e608f`..`f85094b`).** Un-ignored
      settings.json/hooks/commands/skills; **vendored** the handoff hooks + `/handoff` command from
      symlinks → real files; removed the dangling `.gitmodules`; normalized handoff.md CRLF→LF;
      fixed SKILL.md (machine path + "C++"→Rust). `settings.local.json` stays local. See CLAUDE.md.
      **Update 2026-07-14 (`d82420f`):** clarified the SessionStart hook message. It was printing
      "No handoff context found" whenever the gitignored local scratch (`.claude/context.md` etc.)
      was absent — i.e. *always* in a fresh clone — even though `docs/HANDOFF.md` loads via CLAUDE.md's
      `@`-import. Not a path bug (the hook `cd`s to `git rev-parse --show-toplevel`); the two handoff
      channels are just separate by design. Now it points at the shared handoff instead of implying
      nothing loaded. **`BIPP_VERIFY=1` now also runs the hook** (`b7286c1`) so the smoke test shows
      the exact fresh-clone context message. **Host-verified on Denali 2026-07-14** — the new message
      renders correctly in a fresh-clone sandbox.
- [x] **OpenShell auto-clone provisioning — DONE & host-verified 2026-07-13.** Final form is a
      host-side script `openshell/create-bipp-sandbox.sh` that runs `openshell sandbox create …
      -- bash -lc '<clone-if-absent; cd; exec claude>'`. `BIPP_VERIFY=1 ./openshell/create-bipp-sandbox.sh`
      passed on Denali (`PWD=/sandbox/BIPP`, `GIT_OK`, `handoff.md`). **The `--from` image approach
      was abandoned:** OpenShell does not serve a `--from` image's `/usr/local/bin` at runtime (a
      baked script is absent in the running sandbox); PATH is fine, and a long inline `--` one-liner
      corrupts on paste — hence the script. See `openshell/README.md` for the diagnostics.
      **Update 2026-07-14 (`0e62911`):** `BIPP-project-policy.yaml` moved to `openshell/` (next to
      the script that consumes it); `--policy` path + README/PROJECT_PLAN refs updated. Filename
      unchanged so the enforced policy matching is unaffected. Re-verified on Denali with
      `BIPP_VERIFY=1` (all three markers pass; sandbox auto-deleted).
- [x] **XILEM Phase A — DONE and merged to `main` (2026-07-13).** Deleted both broken GUI trees;
      `app.rs` = plain `AppData`, `main.rs` = minimal Xilem 0.4 app. `cargo check -p boomaga-preview`
      is clean (warnings only) on Denali. Verified xilem 0.4.0 API recorded in `XILEM_MIGRATION.md`
      (button takes a child view; `flex(Axis, seq)`; `Xilem::new_simple(...).run_in(...)`).
      `document_renderer.rs` was kept dormant at that milestone; it is active as of Phase C. NB: this compiles the GUI + core + config only — `boomaga-ipp-backend`
      and `boomaga-ipc` still have their own errors (separate from the GUI migration).
      **Branch note (corrected 2026-07-14):** the Phase A commits (`37445fd`/`8f47a5c` + `d785e66`)
      have been on `main` all along — `main` is linear and 6+ commits *ahead* of tag `8f47a5c`. The
      `xilem-phase-a` branch just pointed at that ancestor (fully contained in `main`,
      `git merge-base --is-ancestor` = true) — **redundant, so deleted 2026-07-14** (remote + Denali
      local + pruned; `origin` now has `main` only). There was never a "merge to main" pending or a
      backup risk — an earlier note claiming the branch was unbacked-up was wrong (it keyed off "not
      on remote" without checking ancestry).
- [x] **XILEM Phase B — DONE, merged, and host-verified (2026-07-19, `4b761e6`).** Added the
      horizontal first/previous/next/last and zoom toolbar, preview-canvas placeholder, and status
      row. Added three `AppData` tests covering navigation bounds, empty-document navigation, and
      zoom clamp/reset. On Denali, `cargo check -p boomaga-preview` and all three tests passed; the
      Wayland preview window rendered correctly. The temporary `xilem-phase-b` branch was deleted
      locally and remotely after its fast-forward merge to `main`.
- [x] **XILEM Phase C — DONE and host-verified (2026-07-20).** Added the custom Masonry
      PDF canvas, repaired the Poppler/Cairo renderer handoff, and wired an optional PDF path
      into startup with cached rendered pages. On Denali, `cargo check -p boomaga-preview`
      passed, all seven tests passed, and a real multi-page PDF rendered with working navigation
      and zoom. Evidence: `docs/screenshots/Bommaga-IPP-Preview-Screenshot_2026-07-19_185221.png`.
      Phase C was fast-forwarded to `main`; its feature branch was deleted locally and remotely.
      Phase D follows on the `xilem-phase-d` branch.
- [ ] **XILEM Phase D — IMPLEMENTED, HOST VERIFICATION PENDING (2026-07-22).** Added native
      PDF selection, asynchronous CLI-path loading, a dedicated renderer thread retaining the
      non-`Send`/`Sync` Poppler document, Xilem `worker`/`MessageProxy` delivery, explicit
      loading/error/progress state, generation-safe stale-result rejection, and on-demand caching.
      This sandbox has no Rust toolchain. On Denali, regenerate `Cargo.lock` for `rfd`, run
      `cargo fmt`, `cargo check -p boomaga-preview`, and `cargo test -p boomaga-preview`,
      then visually verify file-open and responsiveness with a large PDF.

## 3. Open questions / waiting on
<!-- Decisions or inputs owned by the human, or external events being awaited. -->
- **Phase D needs Denali verification.** The branch adds `rfd = "0.17.2"`, but this sandbox
  has no Cargo, so `Cargo.lock` is not regenerated and compile/tests/runtime are unverified.
- **Workspace still not compile-verified** here (no toolchain). Phase C is green on the host;
  **`cargo check --workspace` is still red**
  because `boomaga-ipp-backend` / `boomaga-ipc` have their own errors (see §2). Re-run both on the host.
- Also fix `FileType` in `boomaga-core` to match decision #4 (still lists `PostScript`/`Ps`; needs
  `Pdf`/`PwgRaster`/`Jpeg`).

## 4. Key decisions & rationale (durable — don't re-litigate)
<!-- Settled calls a future session should honor unless explicitly revisited. -->
- **IPC = Unix domain sockets + versioned JSON** (systemd socket activation), *not* D-Bus.
  `zbus_systemd` is scoped to systemd lifecycle only. (Issue #3, SRS Option B.)
- **Capture side exposes an IPP Everywhere print *service*** (driverless queue CUPS forwards to);
  downstream *may* act as an IPP client. (Issue #1.)
- **Accepted formats: PDF, PWG Raster, JPEG.** PostScript & **Ghostscript dropped** — poppler-rs +
  qpdf + Boomaga-IPP code cover the residuals. (Issue #4.) *Code still lags — see §3.*
- **Imposition (N-up/booklet/scale/rotate/margins/gutter) computed in `boomaga-layout-engine`**;
  qpdf-rs assembles/applies content-preserving transforms; poppler-rs renders preview. (Issue #11.)
- **No plugin system.** `boomaga-plugins` deleted; specs stay silent. (Issue #10.) Code fully clean.
- **GUI = Xilem** (Druid deprecated). Phases A/B/C are complete; Phase D is implemented on
  `xilem-phase-d` and awaits host verification.
- **SRS/UIS v0.2.2 Appendix C now conforms to code** (`c471a71`); `docs/uml/*.puml` is the
  maintained source. (Supersedes the earlier "Appendix C is an unreconciled Perplexity model" note.)
- **`.claude/` handoff config is repo-shipped and vendored** (real files — no symlinks, no
  submodule). Portable across clones. See CLAUDE.md "Claude Code setup".
- **Sandboxes are provisioned via `openshell/create-bipp-sandbox.sh`** (host-side): an
  `openshell sandbox create … -- bash -lc '<clone; cd; exec claude>'` that auto-clones into
  `/sandbox/BIPP` and launches claude with cwd = the repo (so `/handoff` resolves). The `--from`
  image route was abandoned (baked files not served at runtime — see `openshell/README.md`).

## 5. Gotchas / environment quirks
<!-- The stuff that wastes an hour if you don't know it. -->
- **Repo lives in a SUBDIR `/sandbox/BIPP`, not `/sandbox`.** Claude Code discovers project config
  (`.claude/commands` → `/handoff`; `.claude/settings.json` → hooks) from its launch cwd. Launch
  claude *inside* the repo or `/handoff` is "unknown". The `--from` bootstrap `cd`s there for you.
- **Git URL MUST keep the `.git` suffix.** The enforced `BIPP-project-policy.yaml` matches
  `/GaryScottMartin/Boomaga-IPP*/...`; a bare `.../Boomaga-IPP` URL 403s at the proxy.
- **There is a real `.git` here with a working `origin`** — commit/push work in-sandbox. (`--upload`
  provisioning would give NO `.git`; the clone-based `--from` image is why push works.)
- **Default sandbox execution can fail before a command starts.** A `bwrap: No permissions to
  create a new namespace` error can affect patching and ordinary read-only shell commands. Rerun
  only the necessary, narrowly scoped command through approved host execution; do not diagnose it
  as a bad command or path.
- **GitHub authentication is gateway-mediated.** The visible `GITHUB_TOKEN` is an intentional
  placeholder; OpenShell substitutes the real fine-grained PAT at the gateway. Do not use
  `gh auth status` or `/user` to validate it because the PAT lacks user-profile privileges. Git
  remotes must keep the `.git` suffix; authenticated `fetch`, `pull`, `push`, and remote-branch
  deletion must supply the token through `GIT_ASKPASS` or a command-scoped credential helper.
  REST calls must use explicit policy-allowed repository subpaths. A 403 with
  `X-Openshell-Policy`, `policy_denied`, or `rule_missing` means the OpenShell allow rule did not
  match; it does not mean the PAT is invalid. See `openshell/codex/README.md`.
- **No git identity in a fresh sandbox** — the first `git commit` fails with "Author identity
  unknown". Set it repo-locally to match the owner: `git config user.name "Gary S. Martin"` /
  `git config user.email "gmartin@martin-fam.net"` (matches prior commit authorship; Claude stays
  a co-author via the trailer). Also note `git commit` only stages what's already staged — after a
  `git mv` plus separate edits, `git add -A` (or `--amend` afterward) so all files land in one commit.
- **Workspace won't compile as a whole** because backend/IPC gaps remain; `boomaga-preview`
  itself is green through Phase C. Don't trust "80% done" language in older docs.
- **Sandbox persistence:** container is `restart=unless-stopped` (files survive reboot), BUT
  networking + gateway JWT are fragile — a live session isn't reliably restorable. Back up to git.
- **No Rust toolchain in the sandbox** — no `cargo`/`rustc`; compile-check on the host.
- **Spec PDFs use subset fonts** — plain text extraction is garbage; decode via embedded ToUnicode
  CMaps. `docs/*--latest.pdf` == **v0.2.2** (byte-identical); v0.1.0/v0.2.0/v0.2.1 baselines kept.

## 6. Repo & workflow
<!-- Conventions so a session doesn't guess. -->
- Repo: `GaryScottMartin/Boomaga-IPP`, default branch **`main`**. Owner pushes directly to `main`.
- Recent milestone: `c471a71` "Update UML … to agree with current code" = the v0.2.2 spec state.
- GitHub access in-sandbox is REST-only (GraphQL blocked) — use `gh api`, not `gh issue list`.
- Commit trailer convention: `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.

## 7. Pointers
<!-- Where the real detail lives. Keep this file thin; link out. -->
- `README.md`, `CLAUDE.md` — project overview, build, inter-crate patterns, Claude Code setup.
- `docs/PROJECT_PLAN.md` — architecture, phases, honest per-crate status.
- `docs/XILEM_MIGRATION.md` — GUI migration plan (Phase D is the next code step).
- `docs/SW-Reqrmnts-Spec--latest.pdf`, `docs/User-Interface-Spec--latest.pdf` — current specs (v0.2.2).
- `docs/uml/*.puml` — code-conformant PlantUML (now also embedded in spec Appendix C).
- `openshell/create-bipp-sandbox.sh` + `openshell/README.md` — host-side sandbox
  provisioning (auto-clone + launch claude); `BIPP_VERIFY=1` for a smoke test.
- GitHub issues: <https://github.com/GaryScottMartin/Boomaga-IPP/issues>

---
<!-- When you finish a session: update §1–§3, add any new §4 decisions/§5 gotchas, prune the stale. -->
