# Xilem Migration Plan

> **Last reviewed against code:** 2026-07-22.
> **Phase C status:** **DONE and host-verified on Denali (2026-07-20)** — the
> Masonry canvas displays real Poppler/Cairo-rendered PDF pages; navigation and
> zoom work as expected, and all seven preview tests pass.
> **Status:** Phases A/B/C are complete on `main`; Phase D is implemented on
> `xilem-phase-d` and awaits dependency resolution plus host compile/runtime verification.

## Overview
This document tracks replacing Druid with Xilem for the `boomaga-preview` GUI.
Target architecture: SRS/UIS **v0.2.2** Appendix C and [`docs/uml/`](./uml/)
(`AppData`, `DocumentRenderer`).

## Motivation

### Why Xilem?
- **Native Rust GUI**: full Rust ecosystem, no C++ toolkit FFI
- **Wayland-optimised**: built for modern Linux compositors (via winit + Vello/wgpu)
- **Future-proof**: active Linebender development; Druid is unmaintained
- **Reactive/declarative**: a view tree rebuilt from state, diffed each update

### Why not Druid?
- Druid is **unmaintained** since mid-2023
- The Linebender community moved to Xilem (and its Masonry widget layer)
- Long-term viability concerns for production software

## Current Status

Phases A, B, and C are complete. Phase D is implemented on the
`xilem-phase-d` feature branch: a native PDF chooser and command-line paths feed
one background renderer thread through Xilem 0.4's `worker`/`MessageProxy`
mechanism. `AppData` holds loading/error/progress state and a sparse on-demand
page cache. Host compile and Wayland runtime verification are still pending.

**Dependencies:**
- Workspace `Cargo.toml` and `crates/boomaga-preview/Cargo.toml` declare
  `xilem = "0.4"`, `kurbo = "0.11"`, `winit = "0.30"`, plus `cairo-rs = "0.20"`
  and `poppler = "0.6"` for rendering.
- Phase D adds `rfd = "0.17.2"` for the Linux native/portal file chooser.
- **`druid` is not a dependency in any manifest.**

**Code:**
- `src/main.rs` contains the Xilem view tree, file-open control, status display,
  and persistent renderer worker.
- `src/app.rs` contains navigation/zoom transitions plus generation-safe
  loading, error, progress, worker-channel, and sparse page-cache state.
- `src/render_worker.rs` owns the command/result protocol. It creates and keeps
  `DocumentRenderer` on one dedicated OS thread because Poppler documents are
  neither `Send` nor `Sync`.
- `src/document_renderer.rs` retains the Poppler/Cairo load and rasterization
  implementation; only core metadata and completed `CanvasImage` values cross
  back to Xilem's UI thread.
  surface bytes now feed the Xilem/Masonry canvas through `CanvasImage`.
- The domain types in `boomaga-core` (`Document`, `Page`, `PrintOptions`) and
  `boomaga-config` are framework-independent and stay as-is.

## The correct Xilem 0.4 mental model

Xilem is **not** an immediate-mode / retained-widget framework where you implement a
`Widget` trait with `paint`/`event`/`lifecycle`. That is the *Druid/Masonry* shape.
In Xilem you:

1. Hold your app state in a plain value (our `AppData`).
2. Write an **`app_logic`** function `fn(&mut AppData) -> impl WidgetView<AppData>`
   that returns a **view tree** built from view constructors in `xilem::view::*`
   (e.g. `flex`, `button`, `label`, `sized_box`, `prose`, `textbox`). Interactions
   are wired with callbacks that receive `&mut AppData`.
3. On each state change Xilem re-runs `app_logic`, **diffs** the new view tree against
   the old, and mutates the underlying **Masonry** widget tree accordingly.
4. Run the app roughly as:
   `Xilem::new(initial_state, app_logic).run_windowed(event_loop, window_attributes)`
   (Xilem re-exports `winit`; a window is a winit window).

**Custom drawing** (the rendered PDF page canvas) is done by implementing a
**Masonry `Widget`** (that's where `paint`/`layout`/pointer-event methods live, via
`masonry::…`) and exposing it to the Xilem view tree through a small `View` wrapper —
*not* by implementing a `xilem::Widget`.

**Async / external events** (a page finishing rendering on a worker, or a job arriving
over the Unix-socket IPC) are delivered back into state through Xilem's message/proxy
mechanism (`xilem::core` — e.g. a worker view or a `MessageProxy`), which then triggers
a rebuild. This is the hook the backend→GUI IPC notification (see `docs/uml/C3-sequence.puml`)
will use.

### Verified xilem 0.4.0 API (confirmed by `cargo check`, Phase A)

The shape below is **not a guess** — it's what actually compiles against the pinned
`xilem 0.4.0` (see `crates/boomaga-preview/src/main.rs`). Key facts that differ from
older Xilem and from naive expectations:

- **`button` takes a child *view*, not a string.** `button(label("Prev"), |d| …)` —
  a bare `&str` is not a `WidgetView`.
- **`flex` takes an `Axis` first:** `flex(Axis::Vertical, (child1, child2, …))`, where
  `Axis` is `xilem::view::Axis` and the children are a tuple (a `FlexSequence`).
- **Single-window apps use `Xilem::new_simple` + `WindowOptions` + `run_in`**, not
  `new(...).run_windowed(...)`. `new_simple` wraps the state in `ExitOnClose`
  (→ `AppState`) and the returned view into one window. Plain `Xilem::new` is the
  *multi-window* API (logic must return `impl Iterator<Item = WindowView<State>>`).
- Callbacks are `Fn(&mut State) -> Action`; returning `()` is fine.
- `impl WidgetView<AppData> + use<>` works (Rust 1.88).

```rust
use xilem::view::{button, flex, label, Axis};
use xilem::{EventLoop, WidgetView, WindowOptions, Xilem};

fn app_logic(data: &mut AppData) -> impl WidgetView<AppData> + use<> {
    flex(
        Axis::Vertical,
        (
            label(format!("page {}", data.current_page + 1)),
            button(label("< prev"), |d: &mut AppData| d.previous_page()),
            button(label("next >"), |d: &mut AppData| d.next_page()),
        ),
    )
}

fn main() -> anyhow::Result<()> {
    let app = Xilem::new_simple(AppData::default(), app_logic, WindowOptions::new("Boomaga-IPP"));
    app.run_in(EventLoop::with_user_event())?;
    Ok(())
}
```

> Custom drawing uses a **Masonry `Widget`** wrapped as a Xilem `View`; the
> Phase C implementation is host-verified against `masonry 0.4`.

## Migration Phases (revised for the actual remaining work)

### Phase A: Get to a compiling Xilem skeleton — ✅ DONE (2026-07-13)
1. ✅ Deleted the dangling Druid modules and the incorrect Xilem scaffolds
   (`main_xilem.rs`, `app_xilem.rs`, `document_renderer_xilem.rs`, the Druid-shaped
   `widgets/`, `handlers/`, `window.rs`, `menu_bar.rs`, `toolbar.rs`,
   `document_view.rs`, `viewer/`). Kept `document_renderer.rs` (dormant).
2. ✅ Rewrote `app.rs` as a plain `AppData` value (no `Data`/`Lens` derives).
3. ✅ Rewrote `main.rs` with `app_logic` + `Xilem::new_simple(...).run_in(...)`
   (see the verified API above).
4. ✅ `cargo check -p boomaga-preview` clean (warnings only); merged to `main`.

### Phase B: Core view tree — ✅ DONE (2026-07-19; host-verified on Denali)
- ✅ `flex` layout: toolbar row + page canvas placeholder + status row.
- ✅ Navigation (prev/next/first/last) and zoom (in/out/reset) as `button` callbacks on `AppData`.
- ✅ Page counter / status via `label`.
- ✅ Focused unit tests for navigation bounds and zoom clamping/reset.

### Phase C: PDF page canvas (Masonry custom widget)
- ✅ Implemented a Masonry `PdfCanvasWidget` that paints a rendered page image.
- ✅ Wrapped it as a reactive Xilem `View` and placed it in the Phase B tree.
- ✅ Added validated Cairo-compatible premultiplied-BGRA page image state.
- ✅ Repaired and re-enabled `document_renderer`; its Cairo surface bytes feed
  the canvas through validated premultiplied-BGRA image state.
- ✅ Host-verified on Denali: `cargo check -p boomaga-preview`, all seven tests,
  real multi-page PDF display, navigation, and zoom. Evidence: [`Bommaga-IPP-Preview-Screenshot_2026-07-19_185221.png`](screenshots/Bommaga-IPP-Preview-Screenshot_2026-07-19_185221.png).

### Phase D: Document loading & async rendering — 🚧 IMPLEMENTED, VERIFICATION PENDING
- ✅ Native PDF file chooser plus asynchronous command-line path loading.
- ✅ Dedicated renderer thread owns each non-`Send`/`Sync` Poppler document.
- ✅ Xilem `worker` + `MessageProxy` safely deliver metadata and page images to `AppData`.
- ✅ Idle/loading/ready/error status, generation-based stale-result rejection,
  render progress, and an on-demand page cache.
- ⏳ Regenerate `Cargo.lock`, run `cargo fmt`, `cargo check -p boomaga-preview`,
  and `cargo test -p boomaga-preview` on Denali; then visually verify file-open,
  responsiveness on a large PDF, navigation/cache behavior, and error display.

### Phase E: Imposition & IPC wiring
- Wire `boomaga-layout-engine` (N-up / booklet / transforms) into preview state
  (`docs/uml/C2-class.puml` marks `AppData → NUpCalculator` as `<<planned>>`).
- Wire the Unix-socket IPC (`boomaga-ipc`) so backend job notifications update `AppData`
  (`docs/uml/C3-sequence.puml`).

### Phase F: Print dialog & downstream submit
- Print options dialog bound to `PrintOptions`.
- Downstream printer selection + submit (CUPS/IPP client — no `cups` dep yet).

### Phase G: Testing, polish, docs
- Unit tests for navigation/zoom/state; keyboard shortcuts; accessibility.
- Update README/CLAUDE; remove any lingering Druid references.

## Druid → Xilem concept mapping (corrected)

| Druid concept | Xilem 0.4 equivalent |
|---------------|----------------------|
| `Data` / `Lens` traits | Plain state value; access via closures in `app_logic` — no traits |
| `Widget` trait (`paint`/`event`/`lifecycle`) | **No user `Widget` trait.** Build **views** (`xilem::view::*`); for custom drawing implement a **Masonry** `Widget` |
| `WidgetPod` tree (retained) | View tree rebuilt from state each update, diffed onto the Masonry tree |
| `EventCtx` / `LifeCycle` / `PaintCtx` | Live in **Masonry** (only when writing a custom widget), not at the `xilem` root |
| `WindowDesc` / `AppLauncher` / `run_app` | `Xilem::new(state, app_logic).run_windowed(event_loop, attrs)` |
| `Env` / theme | Xilem/Masonry styling on views (`.style`-style modifiers) |
| Async via `ExtEventSink` | `xilem::core` worker / `MessageProxy` message-into-state |

## Success Criteria

### Functional
- [x] Load and display PDF documents
- [x] Navigate pages (next, previous, first, last)
- [x] Zoom in/out/reset
- [ ] Apply imposition (N-up, booklet) in preview
- [ ] Select downstream printer and submit
- [ ] Toolbar + menu
- [ ] Keyboard shortcuts (Space, N, P, +/-, 0)

### Technical
- [x] `cargo build` succeeds with **no Druid references** and a coherent Xilem tree
- [x] Runs on Linux/Wayland (winit)
- [ ] Page load < 100 ms typical; smooth zoom
- [ ] Reasonable memory footprint

### Code Quality
- [x] Uses the real Xilem 0.4 view-based API (verified against docs.rs), not renamed Druid
- [ ] Test coverage > 70% for state logic
- [ ] Clear, documented modules; no dead `_xilem.rs` duplicates

## Risk Assessment

### High Risk
- **API drift**: Xilem is pre-1.0; symbol names/signatures change between releases.
  Pin `xilem = 0.4` and verify against that exact version.

### Medium Risk
- **State/reactivity model**: declarative rebuild-and-diff is a different mental model
  from Druid's retained widgets.
- **Async rendering + IPC**: feeding worker/IPC results back into state correctly.

### Low Risk
- **Custom PDF canvas**: the Poppler/Cairo-to-Masonry/Vello bridge is implemented
  and host-verified through Phase C.
- **Document loading** (`document_renderer.rs`) — reusable as-is.
- **Configuration / domain types** — framework-independent.

## Next Steps
1. ✅ Correct this plan to reflect the real (broken, mid-migration) state.
2. ✅ **Phase A** — deleted Druid + pseudo-Xilem code; compiling Xilem skeleton on `main`.
3. ✅ **Phase B** — core view tree (toolbar row, navigation, zoom, status).
4. ✅ **Phase C** — custom canvas + Poppler/Cairo renderer handoff, host-verified.
5. 🚧 Phase D — document loading & async rendering.
6. 🚧 Phase E — imposition + IPC wiring.
7. 🚧 Phase F — print dialog & downstream submit.
8. 🚧 Phase G — testing, polish, docs.
