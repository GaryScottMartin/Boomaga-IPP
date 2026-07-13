# Xilem Migration Plan

> **Last reviewed against code:** 2026-07-13.
> **Status:** **Phase A DONE** — `boomaga-preview` now compiles as a minimal Xilem
> 0.4 skeleton (`cargo check -p boomaga-preview` clean, warnings only, on the
> `xilem-phase-a` branch). Both broken GUI trees (dangling Druid modules +
> fabricated-Xilem scaffolds) were deleted; `app.rs` is a plain `AppData` and
> `main.rs` is a real Xilem app. `document_renderer.rs` is retained but dormant
> (re-wired in Phase C). Next: **Phase B** (real view tree / layout) then **Phase C**
> (Masonry PDF canvas). The verified xilem 0.4.0 API is recorded below — use it,
> not the pre-Phase-A guesses.

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

## Current Status — the real picture

**This is a half-finished migration, not a fresh start.** Concretely:

**Dependencies (already changed):**
- Workspace `Cargo.toml` and `crates/boomaga-preview/Cargo.toml` declare
  `xilem = "0.4"`, `kurbo = "0.11"`, `winit = "0.30"`, plus `cairo-rs = "0.18"`
  and `poppler = "0.6"` for rendering.
- **`druid` is not a dependency in any manifest.** (Verified: no `druid` entry in
  any `Cargo.toml`.)

**Code (inconsistent — this is why it won't build):**
- The **active** entry point and several modules still call Druid, with no `druid`
  crate available:
  - `src/main.rs` — `druid::WindowDesc::new()`, `druid::run_app(...)`
  - `src/app.rs` — `use druid::{AppLauncher, Data, Env, Lens}`, `#[derive(Clone, Data, Lens)]`
  - `src/menu_bar.rs`, `src/toolbar.rs` — Druid `Widget` impls (`paint`/`layout`/`event`/`lifecycle`)
  - `src/widgets/page_container.rs` — `use druid::kurbo::Vec2`, `druid::ImageData`
  - `src/document_view.rs`, `src/viewer/` — Druid custom widgets
  → every one of these is a dangling reference to a crate that isn't linked.
- Parallel **Xilem scaffolds** exist but were written against a **non-existent API**:
  - `src/main_xilem.rs` — `use xilem::{App, Color, Event, EventCtx, LifeCycle, LifeCycleCtx, PaintCtx, Env, Widget}`
  - `src/widgets/{document_viewer,page_container}.rs` — `use xilem::{Widget, Flex, Label, PaintCtx, ...}` with `paint`/`event`/`lifecycle` methods
  - `src/window.rs` — `use xilem::WindowConfig`
  - `src/handlers/{document,navigation}.rs` — `use xilem::{Event, EventCtx, WindowCtx}`
  → **Xilem 0.4 exports none of these** (`xilem::Widget`, `Flex`, `Label`, `App`,
  `WindowConfig`, `PaintCtx`, `EventCtx`, `LifeCycle` at the crate root). These files
  are essentially Druid code with the import path renamed to `xilem::`. They must be
  rewritten, not patched.

**What is salvageable:**
- `src/document_renderer.rs` — **real poppler + cairo rendering** (loads a PDF via
  `poppler::Document`, extracts metadata, renders pages to a Cairo surface). This is
  framework-agnostic and should be **kept**; only the surface→GPU-image handoff needs
  adapting to Xilem/Masonry paint.
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

> Custom drawing (the PDF page canvas, Phase C) is still a **Masonry** `Widget`
> wrapped as a Xilem `View` — that layer's exact method set should be checked against
> `masonry 0.4` when we get there.

## Migration Phases (revised for the actual remaining work)

### Phase A: Get to a compiling Xilem skeleton — ✅ DONE (2026-07-13)
1. ✅ Deleted the dangling Druid modules and the incorrect Xilem scaffolds
   (`main_xilem.rs`, `app_xilem.rs`, `document_renderer_xilem.rs`, the Druid-shaped
   `widgets/`, `handlers/`, `window.rs`, `menu_bar.rs`, `toolbar.rs`,
   `document_view.rs`, `viewer/`). Kept `document_renderer.rs` (dormant).
2. ✅ Rewrote `app.rs` as a plain `AppData` value (no `Data`/`Lens` derives).
3. ✅ Rewrote `main.rs` with `app_logic` + `Xilem::new_simple(...).run_in(...)`
   (see the verified API above).
4. ✅ `cargo check -p boomaga-preview` clean (warnings only) on branch `xilem-phase-a`.

### Phase B: Core view tree
- `flex`/`sized_box` layout: toolbar row + page canvas + status row.
- Navigation (prev/next/first/last) and zoom (in/out/reset) as `button` callbacks on `AppData`.
- Page counter / status via `label`.

### Phase C: PDF page canvas (Masonry custom widget)
- Implement a Masonry `Widget` that paints the Cairo/poppler-rendered page image.
- Feed it the surface produced by `document_renderer::render_page_to_surface`.
- Wrap it as a Xilem `View` and place it in the tree.

### Phase D: Document loading & async rendering
- File-open → load via `DocumentRenderer::load` (keep existing poppler code).
- Render pages off the UI thread; deliver results into state via the worker/proxy hook.

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
- [ ] Load and display PDF documents
- [ ] Navigate pages (next, previous, first, last)
- [ ] Zoom in/out/reset
- [ ] Apply imposition (N-up, booklet) in preview
- [ ] Select downstream printer and submit
- [ ] Toolbar + menu
- [ ] Keyboard shortcuts (Space, N, P, +/-, 0)

### Technical
- [ ] `cargo build` succeeds with **no Druid references** and a coherent Xilem tree
- [ ] Runs on Linux/Wayland (winit)
- [ ] Page load < 100 ms typical; smooth zoom
- [ ] Reasonable memory footprint

### Code Quality
- [ ] Uses the real Xilem 0.4 view-based API (verified against docs.rs), not renamed Druid
- [ ] Test coverage > 70% for state logic
- [ ] Clear, documented modules; no dead `_xilem.rs` duplicates

## Risk Assessment

### High Risk
- **API drift**: Xilem is pre-1.0; symbol names/signatures change between releases.
  Pin `xilem = 0.4` and verify against that exact version.
- **Custom PDF canvas**: bridging poppler/Cairo output into a Masonry paint pass and
  onto Xilem's GPU (Vello) renderer is the hardest integration point.
- **Two dead code paths today**: the repo currently carries *both* broken Druid and
  broken pseudo-Xilem trees; deleting confidently (Phase A) is prerequisite to progress.

### Medium Risk
- **State/reactivity model**: declarative rebuild-and-diff is a different mental model
  from Druid's retained widgets.
- **Async rendering + IPC**: feeding worker/IPC results back into state correctly.

### Low Risk
- **Document loading** (`document_renderer.rs`) — reusable as-is.
- **Configuration / domain types** — framework-independent.

## Next Steps
1. ✅ Correct this plan to reflect the real (broken, mid-migration) state.
2. ✅ **Phase A** — deleted Druid + pseudo-Xilem code; compiling Xilem skeleton (branch `xilem-phase-a`).
3. 🚧 **Phase B (next)** — core view tree (toolbar row, navigation, zoom, status), real layout.
4. 🚧 Phase C — Masonry PDF page canvas.
5. 🚧 Phase D — document loading & async rendering.
6. 🚧 Phase E — imposition + IPC wiring.
7. 🚧 Phase F — print dialog & downstream submit.
8. 🚧 Phase G — testing, polish, docs.
