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

> **Last updated:** 2026-07-14 · **By:** Claude (Opus 4.8, 1M) + @GaryScottMartin
> **This session's focus:** sandbox/tooling hygiene (no code changes) — relocated the OpenShell
> policy file into `openshell/`; clarified the SessionStart hook's fresh-clone message and made
> `BIPP_VERIFY` exercise it; deleted the redundant `xilem-phase-a` branch (already on `main`). All
> host-verified on Denali. `main` @ `26a6a1d`.

---

## 1. TL;DR — where things stand
<!-- One short paragraph. What just happened, what's the single most important next step. -->
Specs **v0.2.2** now embed the code-conformant Appendix C UML (`c471a71`), so that thread is
closed. This session was tooling/hygiene, not code: reconciled `PROJECT_PLAN.md` +
`XILEM_MIGRATION.md` with the real code (both were badly stale), **vendored** the `.claude/`
handoff config so it's portable and version-controlled, and added **host-verified** OpenShell
provisioning — `openshell/create-bipp-sandbox.sh` auto-clones the repo and launches claude
(the `--from` image route was tried and abandoned; see §2). **GUI migration:** `boomaga-preview` was a broken Druid→Xilem
half-migration; **Phase A is done and already on `main`** (skeleton commits `37445fd`/`8f47a5c` +
`d785e66`) — both broken trees deleted, a minimal Xilem 0.4 skeleton compiles
(`cargo check -p boomaga-preview` clean). **Next up:** Phase B (real view tree). Separately,
`boomaga-ipp-backend` / `boomaga-ipc` still don't compile (their own stub/bug issues, independent
of the GUI).

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
      `document_renderer.rs` kept dormant. **Next: Phase B** (real view tree/layout), then Phase C
      (Masonry PDF canvas). NB: this compiles the GUI + core + config only — `boomaga-ipp-backend`
      and `boomaga-ipc` still have their own errors (separate from the GUI migration).
      **Branch note (corrected 2026-07-14):** the Phase A commits (`37445fd`/`8f47a5c` + `d785e66`)
      have been on `main` all along — `main` is linear and 6+ commits *ahead* of tag `8f47a5c`. The
      `xilem-phase-a` branch just pointed at that ancestor (fully contained in `main`,
      `git merge-base --is-ancestor` = true) — **redundant, so deleted 2026-07-14** (remote + Denali
      local + pruned; `origin` now has `main` only). There was never a "merge to main" pending or a
      backup risk — an earlier note claiming the branch was unbacked-up was wrong (it keyed off "not
      on remote" without checking ancestry).

## 3. Open questions / waiting on
<!-- Decisions or inputs owned by the human, or external events being awaited. -->
- **Workspace still not compile-verified** here (no toolchain). Phase A has landed, so
  `cargo check -p boomaga-preview` is green on the host; **`cargo check --workspace` is still red**
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
- **GUI = Xilem** (Druid deprecated). See `docs/XILEM_MIGRATION.md`. NOTE: currently a broken
  half-migration — this is *the* thing blocking a green build.
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
- **Pushing over HTTPS:** `GITHUB_TOKEN` is gateway-injected and can't be cleared, so `gh auth login`
  refuses. It's fine-grained (403s on some REST like `/user`) but has git push/data access. It's not
  wired into git's credential flow — feed it via `GIT_ASKPASS` (a script echoing `x-access-token` /
  `$GITHUB_TOKEN`). Raw `curl` to github is egress-blocked; use `gh api` or git-with-askpass.
  `git ls-remote` is authoritative (API reads can lag a fresh push).
- **No git identity in a fresh sandbox** — the first `git commit` fails with "Author identity
  unknown". Set it repo-locally to match the owner: `git config user.name "Gary S. Martin"` /
  `git config user.email "gmartin@martin-fam.net"` (matches prior commit authorship; Claude stays
  a co-author via the trailer). Also note `git commit` only stages what's already staged — after a
  `git mv` plus separate edits, `git add -A` (or `--amend` afterward) so all files land in one commit.
- **Workspace won't compile** — `boomaga-preview` half-migration (see §1). Don't trust "80% done"
  language in older docs; `PROJECT_PLAN.md` now has the honest per-crate status.
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
- `docs/XILEM_MIGRATION.md` — GUI migration plan (Phase A is the next code step).
- `docs/SW-Reqrmnts-Spec--latest.pdf`, `docs/User-Interface-Spec--latest.pdf` — current specs (v0.2.2).
- `docs/uml/*.puml` — code-conformant PlantUML (now also embedded in spec Appendix C).
- `openshell/create-bipp-sandbox.sh` + `openshell/README.md` — host-side sandbox
  provisioning (auto-clone + launch claude); `BIPP_VERIFY=1` for a smoke test.
- GitHub issues: <https://github.com/GaryScottMartin/Boomaga-IPP/issues>

---
<!-- When you finish a session: update §1–§3, add any new §4 decisions/§5 gotchas, prune the stale. -->
