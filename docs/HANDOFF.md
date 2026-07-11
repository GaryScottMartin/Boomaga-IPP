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
> **This session's focus:** diff v0.2.1 Appendix C UML against the actual Rust code.

---

## 1. TL;DR — where things stand
<!-- One short paragraph. What just happened, what's the single most important next step. -->
**v0.2.1 specs landed** (`--latest.pdf` now == v0.2.1). Extracted all six Appendix C UML source
tables (CMap decode) and diffed them against the real 6-crate code. Result is **mixed**: the
Appendix C diagrams are the earlier Perplexity-generated models and were **not reconciled** with
the v0.2.0/v0.2.1 corrections — they fail 3 of the 4 alignment checks (see §2). Separately, the
"no plugins" decision (#10) had a **build-breaking dangling `boomaga-plugins` reference** in
`boomaga-config`, now **fixed** this session. **Next up:** decide whether to correct Appendix C
in the spec (regenerate diagrams to match code + issue #1 service role), or treat it as
aspirational and annotate it.

## 2. Active threads / in progress
<!-- The heart of the file. Each item: what, state, concrete next action. Delete when done. -->
- [x] **Appendix C diagram ↔ code alignment.** DONE. All six Appendix C UML tables extracted from
      v0.2.1 and diffed vs the 6 crates. Verdict per check: **no `boomaga-plugins`** in the diagrams
      ✅ (but see plugin residue below); **IPP print-*service* role (#1)** ❌ — Appendix C models an
      IPP *client-pull* (`IppEverywhereClient` w/ discover/query/subscribe/fetch; `IPP --> "IPP
      Everywhere Printer"` cloud), contradicting §2.2/Req 1 and the real `IppServer`; **Unix-socket
      IPC (#3)** ❌ — no IPC transport depicted at all (direct arrows); **class names match code** ❌
      — only `Page` and `PrintOptions` of 13 exist by name, both with different fields. Real vs UML
      names: `AppController`→`BoomagaApp/AppData`, `CaptureService`→(none; in `IppServer`+`JobQueue`),
      `IppEverywhereClient`→`IppServer`, `JobStore`→`JobQueue`, `PrintJob`→`PrintJobRequest`,
      `DocumentModel`→`Document`, `PreviewRenderer`→`DocumentRenderer`, `DownstreamPrintService`/
      `CupsRsAdapter`→(none, **no cups dep at all**), `PrinterProfile`→`PrinterInfo/PrinterCapabilities`.
      `JobStatus` variants differ (UML Captured/Draft/Ready/Submitted/Printing/… vs real Queued/
      Processing/Held/Aborted/…). Also: "4-daemon+GUI" wording is off — SRS §2.2 names **3** daemons
      + GUI, and the code ships only **2 binaries** (`boomaga-ipp-backend`, `boomaga-preview`).
- [ ] **Appendix C is unreconciled with the corrected design.** Decide: (a) regenerate the four
      diagrams from the real crate/type layout + #1 service role and re-embed in a spec bump, or
      (b) keep them as an explicitly-labelled "initial analysis model" caveat. Owner call.
- [x] **Plugin residue / build-breaker (#10).** DONE (this session): removed `boomaga-plugins`
      dep from `crates/boomaga-config/Cargo.toml`, the `use boomaga_plugins::core::PluginMetadata`
      import, and the `PluginSettings` struct + `Settings.plugins` field + its `Default`. **Not
      compile-verified** (no Rust toolchain/registry in sandbox — verify host-side with
      `cargo check -p boomaga-config`).
- [x] **Remaining plugin residue.** DONE (this session): removed `enable_plugins`/`plugin_dirs`/
      `with_plugins()` (+ their `Default` init incl. the `.../plugins` path strings) from
      `preview_config.rs`, and the `Error::Plugin` variant (+ its `severity()` arm) from
      `boomaga-core`. Grep confirms **zero** `plugin` references left in `crates/`. #10 fully
      realized in code now. Still not compile-verified in-sandbox (no toolchain).
- [x] **Host repo cleanup.** DONE — stray top-level `boomaga-config/` is gone; only
      `crates/boomaga-config/` remains. `BIPP-project-policy.yaml` kept in repo.

## 3. Open questions / waiting on
<!-- Decisions or inputs owned by the human, or external events being awaited. -->
- **Appendix C reconciliation** (see §2): regenerate to match code, or annotate as aspirational? Owner call.
- The plugin removals in `boomaga-config` **and** `boomaga-core` are **not compile-verified** here —
  needs a host-side `cargo check --workspace`.

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
  specs intentionally stay silent. (Issue #10.) Code fully cleaned 2026-07-10: `boomaga-config`
  dep/import + `PluginSettings`, plus `preview_config` (`enable_plugins`/`plugin_dirs`/`with_plugins`)
  and `boomaga-core` `Error::Plugin` all removed — grep shows zero `plugin` refs in `crates/`.
- **Appendix C UML is the initial (Perplexity-generated) analysis model, NOT ground truth.** It
  diverges from the code and from the #1/#3 corrections (§2). Trust the crate layout + real types
  over the diagrams until the spec is regenerated.
- **GUI = Xilem** (Druid is deprecated). See `docs/XILEM_MIGRATION.md`.

## 5. Gotchas / environment quirks
<!-- The stuff that wastes an hour if you don't know it. -->
- **This runs in an `openshell` sandbox reached over SSH; the repo was `git clone`d directly into
  `/sandbox/BIPP`.** So there **is** a real `.git` here with a working `origin`
  (`GaryScottMartin/Boomaga-IPP`) — git add/commit/push all work in-sandbox. (Earlier sessions used
  a different `--upload .:/sandbox/project` flow that honored `.gitignore` and had **no** `.git`,
  forcing host-side git; that no longer applies to this clone.)
- **Pushing over HTTPS:** `GITHUB_TOKEN` is gateway-injected (an `openshell:resolve:env:…` placeholder
  resolved at exec time) and **can't be cleared** from a shell, so `gh auth login` refuses. The token
  is fine-grained: it 403s on some REST endpoints (e.g. `/user`, repo metadata) but **has git
  push + git-data access**. It isn't wired into git's credential flow by default (`git push` fails
  with "could not read Username"). Fix: feed it via `GIT_ASKPASS` — a tiny script echoing
  `x-access-token` for Username and `$GITHUB_TOKEN` for Password — then `git push` works. Raw `curl`
  to `github.com`/`api.github.com` is egress-blocked (proxy 403 on CONNECT); use `gh api` (REST) or
  git-with-askpass instead. Note API reads (`gh api …/git/ref/heads/main`) can lag a fresh push —
  `git ls-remote` is authoritative.
- **Sandbox persistence:** container is `AutoRemove=false` + `restart=unless-stopped` (files survive
  reboot), BUT networking (gateway netns in `/run`, tmpfs) and the gateway JWT are the fragile parts —
  a live session is not reliably restorable across reboot; `docker-compose down` on the gateway is
  more dangerous than a reboot. Back up work to git; don't rely on the sandbox.
- **Spec PDFs use subset fonts** — plain text extraction yields garbage. They embed ToUnicode CMaps;
  decode by mapping 2-byte glyph codes through the CMaps. No `poppler-utils`/network in the sandbox,
  so PDF pages can't be rendered to images — raster figures can't be inspected visually, but **the
  Appendix C `.puml` text tables extract fine** via CMap decode. Working extractor rebuilt this
  session at `/sandbox/work/extract.py` (parses ObjStm/XRef streams → per-font ToUnicode → decodes
  Tj/TJ); `/sandbox/work/*` is scratch and not persisted to the repo.
- **No Rust toolchain in the sandbox** — no `cargo`/`rustc` and no crates.io registry cache (and raw
  egress is proxy-restricted, so a fresh crate fetch is unlikely to work even if installed). Code
  changes here can't be compile-checked; run `cargo check`/`cargo build` on the host.
- **`docs/*--latest.pdf` == the current `v0.2.1` files** (byte-identical); v0.1.0 + v0.2.0 baselines kept for diffing.

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
