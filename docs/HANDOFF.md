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
> **Session focus:** completed, host-verified, and merged Xilem Phase E to `main`;
> N-up imposition and backend-to-preview job-status IPC are green. Phase F is next.

---

## 1. TL;DR — where things stand
<!-- One short paragraph. What just happened, what's the single most important next step. -->
Xilem migration Phases A through E are complete, host-verified, and merged to `main`.
The preview supports native PDF loading, asynchronous sparse rendering, navigation/zoom,
and 1/2/4/6/8-up imposition with horizontal/vertical fill and the intended sheet
orientations. Versioned Unix-socket JSON IPC now carries backend job lifecycle messages into
typed, deduplicated preview state. Focused Denali checks pass; tests are 7 layout-engine,
3 IPC, 1 backend, and 19 preview. The next implementation thread is Phase F: print options,
printer selection, and downstream submission. Booklet UI remains a deliberate follow-up.

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
      Phase D followed and is now also on `main`.
- [x] **XILEM Phase D — DONE and host-verified (2026-07-22).** Added native PDF selection,
      asynchronous CLI-path loading, a dedicated renderer thread retaining the non-`Send`/`Sync`
      Poppler document, Xilem `worker`/`MessageProxy` delivery, explicit loading/error/progress
      state, generation-safe stale-result rejection, and on-demand caching. Denali regenerated
      `Cargo.lock`; `cargo check -p boomaga-preview` passed and all ten tests passed. Runtime
      evidence: `docs/screenshots/Boomaga-IPP-Preview-Screenshot_2026-07-21_232928.png`.
      The focused Phase D baseline remains useful historical evidence; Phase E superseded its
      test count and removed the independent config lint blocker.
- [x] **XILEM Phase E — DONE, host-verified, and merged to `main` (2026-07-22).** Added
      1/2/4/6/8-up preview imposition, horizontal/vertical fill, correct sheet orientation,
      sheet navigation, and the finalized toolbar/footer layout. Added protocol-v1 framed JSON
      IPC, backend lifecycle notifications, and a Xilem IPC worker that updates deduplicated
      `AppData` job status. Denali passed 7 layout, 3 IPC, 1 backend, and 19 preview tests.
      The feature branch was deleted after the fast-forward merge. Booklet controls were
      deferred; Phase F print options and downstream submission are next.

## 3. Open questions / waiting on
<!-- Decisions or inputs owned by the human, or external events being awaited. -->
- A fresh workspace-wide `cargo check --workspace` has not been recorded. The focused Phase E
  crates are green on Denali; establish the full-workspace baseline before broader claims.
- Real IPP request parsing/response generation, captured-document handoff, and downstream
  printer submission remain incomplete.
- Booklet controls were outside the accepted Phase E N-up scope and remain open.

## 4. Key decisions & rationale (durable — don't re-litigate)
<!-- Settled calls a future session should honor unless explicitly revisited. -->
- **IPC = Unix domain sockets + versioned JSON** (systemd socket activation), *not* D-Bus.
  `zbus_systemd` is scoped to systemd lifecycle only. (Issue #3, SRS Option B.)
- **Capture side exposes an IPP Everywhere print *service*** (driverless queue CUPS forwards to);
  downstream *may* act as an IPP client. (Issue #1.)
- **Accepted formats: PDF, PWG Raster, JPEG.** PostScript & **Ghostscript dropped** — poppler-rs +
  qpdf + Boomaga-IPP code cover the residuals. (Issue #4.) `FileType` now matches this set.
- **Imposition (N-up/booklet/scale/rotate/margins/gutter) computed in `boomaga-layout-engine`**;
  qpdf-rs assembles/applies content-preserving transforms; poppler-rs renders preview. (Issue #11.)
- **No plugin system.** `boomaga-plugins` deleted; specs stay silent. (Issue #10.) Code fully clean.
- **GUI = Xilem** (Druid deprecated). Phases A/B/C/D/E are complete and host-verified on
  `main`; Phase F is next.
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
- **Focused Phase E crates are green; full-workspace status is unrecorded.** Do not convert
  focused results into a workspace-wide claim without running the workspace command.
- **Sandbox persistence:** container is `restart=unless-stopped` (files survive reboot), BUT
  networking + gateway JWT are fragile — a live session isn't reliably restorable. Back up to git.
- **The sandbox now has a Rust toolchain.** Native libraries, cached dependencies, or network
  policy can still differ from Denali, so record where verification was performed.
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
- `docs/XILEM_MIGRATION.md` — GUI migration plan (Phase F is next).
- `docs/SW-Reqrmnts-Spec--latest.pdf`, `docs/User-Interface-Spec--latest.pdf` — current specs (v0.2.2).
- `docs/uml/*.puml` — code-conformant PlantUML (now also embedded in spec Appendix C).
- `openshell/create-bipp-sandbox.sh` + `openshell/README.md` — host-side sandbox
  provisioning (auto-clone + launch claude); `BIPP_VERIFY=1` for a smoke test.
- GitHub issues: <https://github.com/GaryScottMartin/Boomaga-IPP/issues>

---
<!-- When you finish a session: update §1–§3, add any new §4 decisions/§5 gotchas, prune the stale. -->
