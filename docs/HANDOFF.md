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

> **Last updated:** 2026-07-10 · **By:** Claude (Opus 4.8) + @GaryScottMartin
> **This session's focus:** verify SRS/UIS v0.2.0 resolved all open GitHub issues.

---

## 1. TL;DR — where things stand
<!-- One short paragraph. What just happened, what's the single most important next step. -->
SRS + UIS **v0.2.0** ("Revised Draft Baseline — Corrected Issues") were reviewed against the
12 open GitHub issues. **All 12 are resolved** (11 via the specs; #10 via deleting the
`boomaga-plugins` crate). The only item not machine-verifiable this session is the Appendix C
UML diagrams (raster images). **Next up:** review the forthcoming **v0.2.1** specs, which will
add the Appendix C `.puml` source as text tables — then diff the PlantUML against the actual
Rust crate layout.

## 2. Active threads / in progress
<!-- The heart of the file. Each item: what, state, concrete next action. Delete when done. -->
- [ ] **Appendix C diagram ↔ code alignment.** Waiting on v0.2.1 (specs bumped to 0.2.1, `.puml`
      embedded as text tables). When it lands: extract the four `.puml` blocks (component, class,
      sequence, use-case) and verify against the 6 shipping crates — confirm **no `boomaga-plugins`**,
      the 4-daemon+GUI architecture, IPP print-*service* role (#1), and Unix-socket IPC (#3) are
      reflected, and class names match real Rust types.
- [ ] **Host repo cleanup (in flight, host-side).** Removing stray top-level `boomaga-config/`
      (README-only duplicate of `crates/boomaga-config/`). `BIPP-project-policy.yaml` was decided
      to be committed (kept in repo).

## 3. Open questions / waiting on
<!-- Decisions or inputs owned by the human, or external events being awaited. -->
- v0.2.1 PDFs not yet uploaded to the sandbox / pushed to the repo.

## 4. Key decisions & rationale (durable — don't re-litigate)
<!-- Settled calls a future session should honor unless explicitly revisited. -->
- **IPC = Unix domain sockets + versioned JSON** (systemd socket activation), *not* D-Bus.
  `zbus_systemd` is scoped to systemd lifecycle only. (Issue #3, SRS Option B.)
- **Capture side exposes an IPP Everywhere print *service*** (driverless queue CUPS forwards to);
  downstream *may* act as an IPP client. (Issue #1.)
- **Accepted formats: PDF, PWG Raster, JPEG.** PostScript support **dropped** (not IPP-mandatory);
  **Ghostscript dropped** — poppler-rs + qpdf + Boomaga-IPP code cover the residual functions.
  (Issue #4; explicit in Chapter 2 of both specs.)
- **Imposition (N-up/booklet/scale/rotate/margins/gutter) is computed in `boomaga-layout-engine`**;
  qpdf-rs only assembles/applies content-preserving transforms; poppler-rs renders preview. (Issue #11.)
- **No plugin system.** `boomaga-plugins` crate deleted and removed from the workspace + README;
  specs intentionally stay silent. (Issue #10.)
- **GUI = Xilem** (Druid is deprecated). See `docs/XILEM_MIGRATION.md`.

## 5. Gotchas / environment quirks
<!-- The stuff that wastes an hour if you don't know it. -->
- **This runs in an `openshell` sandbox = a Docker overlay container.** The source was uploaded
  one-way via `--upload .:/sandbox/project`, and **`--upload` honors `.gitignore`** — so `.git/`,
  `.claude/`, `target/`, and `claude-code-handoff/` are **absent**. Consequence: there is **no
  `.git` here**, so all git operations happen on the **host** source dir, not in the sandbox.
- **Sandbox persistence:** container is `AutoRemove=false` + `restart=unless-stopped` (files survive
  reboot), BUT networking (gateway netns in `/run`, tmpfs) and the gateway JWT are the fragile parts —
  a live session is not reliably restorable across reboot; `docker-compose down` on the gateway is
  more dangerous than a reboot. Back up work to git; don't rely on the sandbox.
- **Spec PDFs use subset fonts** — plain text extraction yields garbage. They embed ToUnicode CMaps;
  decode by mapping 2-byte glyph codes through the CMaps (working approach lived in a throwaway
  `/tmp/cmap_extract.py` this session). No `poppler-utils`/network in the sandbox, so PDF pages
  can't be rendered to images — raster figures (e.g. Appendix C) can't be inspected visually.
- **`docs/*--latest.pdf` == the current `v0.2.0` files** (byte-identical); v0.1.0 baselines kept for diffing.

## 6. Repo & workflow
<!-- Conventions so a session doesn't guess. -->
- Repo: `GaryScottMartin/Boomaga-IPP`, default branch **`main`**. Owner pushes directly to `main`.
- Relevant recent commit: `3e8ad23` "Correct SRS & UIS Citations" (2026-07-08) = the v0.2.0 state.
- GitHub access in-sandbox is REST-only (GraphQL is blocked) — use `gh api`, not `gh issue list`.
- Commit trailer convention: `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.

## 7. Pointers
<!-- Where the real detail lives. Keep this file thin; link out. -->
- `README.md`, `CLAUDE.md` — project overview, build, inter-crate patterns.
- `docs/PROJECT_PLAN.md` — architecture, phases, status.
- `docs/XILEM_MIGRATION.md` — GUI framework migration plan.
- `docs/SW-Reqrmnts-Spec--latest.pdf`, `docs/User-Interface-Spec--latest.pdf` — current specs.
- GitHub issues: <https://github.com/GaryScottMartin/Boomaga-IPP/issues>

---
<!-- When you finish a session: update §1–§3, add any new §4 decisions/§5 gotchas, prune the stale. -->
