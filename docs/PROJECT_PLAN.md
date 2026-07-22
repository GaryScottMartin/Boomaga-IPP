# Modern Boomaga Virtual Printer - Implementation Plan

> **Last reviewed against code:** 2026-07-22 (6-crate workspace; 2 binaries).
> **Authoritative architecture:** SRS & UIS **v0.2.2**, Appendix C, and the
> code-conformant PlantUML in [`docs/uml/`](./uml/) (conforms to code @ `34652fa`).
> Where this plan and the specs/UML disagree, the specs/UML win — this document
> is the *implementation* view and is kept in sync with them.

## Context
Need to create a modern version of boomaga (BOOklet MANager), a virtual printer for Linux. The original C++ implementation has good features but is outdated, using CUPS + D-Bus + Ghostscript. Requirements: systemd-managed Linux, Wayland display, IPP Everywhere printing, Rust language implementation, memory-safe and type-safe.

## Requirements Summary
- **Language**: Rust (edition 2021, `rust-version = 1.88`, resolver v2)
- **Display**: Native Wayland
- **Print System**: IPP Everywhere — Boomaga-IPP exposes an IPP Everywhere print
  *service* (driverless queue that host CUPS forwards jobs into); downstream
  submission to a real printer *may* act as an IPP/CUPS client (decision #1)
- **Process Management**: systemd-managed (socket activation)
- **License**: GPLv3
- **UI**: Full-featured document preview & imposition UI. **No plugin system**
  (decision #10 — the `boomaga-plugins` crate and all plugin hooks were removed)
- **Supported input formats**: PDF, PWG Raster, JPEG. **PostScript and Ghostscript
  were dropped** (not IPP-mandatory; decision #4). `FileType` now matches the
  accepted `Pdf`/`PwgRaster`/`Jpeg` set.

## Architecture Overview

See [`docs/uml/C1-component.puml`](./uml/C1-component.puml) for the authoritative
component diagram (solid = present in code; dashed = decided-but-not-yet-wired).

### Components
1. **Backend Service** (`boomaga-ipp-backend`, systemd service)
   - Receives print jobs via an IPP Everywhere print service (`IppServer`, TCP)
   - Queues and processes jobs (`JobQueue`, `JobProcessor`)
   - Notifies the GUI over versioned Unix-socket JSON IPC

2. **Preview Application** (`boomaga-preview`, Wayland GUI · Xilem)
   - Document viewer (`DocumentRenderer` via poppler)
   - Imposition controls (N-up, booklet, rotate, reorder, duplex)
   - Downstream printer selection & submit

3. **Configuration System** (`boomaga-config`)
   - `BackendConfig` (IPP service settings)
   - `PreviewConfig` (GUI preferences)
   - `Settings` + `ConfigManager` (load/save; TOML for configs, JSON for settings)

4. **Shared Components**
   - Domain types + error handling (`boomaga-core`; PDF assembly via `qpdf`)
   - Page layout / imposition engine (`boomaga-layout-engine`: N-up, booklet, transforms)
   - IPC library (`boomaga-ipc`: Unix-domain-socket transport + versioned JSON protocol)
   - PDF rendering for preview (poppler + cairo, in `boomaga-preview`)

### Data Flow
1. User prints → host CUPS
2. Host CUPS sends an IPP Everywhere (driverless) job **into** the Boomaga-IPP `IppServer`
3. Backend queues/processes the job → notifies the GUI over the Unix socket
4. Preview opens the document, renders pages, applies imposition
5. User reviews → selects a downstream printer → submits
6. Downstream job is sent to the selected CUPS/IPP printer

## Technology Stack Decisions

### GUI Framework: Xilem
**Why Xilem over alternatives**:
- Native Wayland rendering with direct compositor integration
- Modern, reactive/declarative Rust GUI (view tree over a Masonry widget layer)
- Good performance for document-centric rendering
- Active development by the Linebender community
- Druid (the original choice) is unmaintained — see [`docs/XILEM_MIGRATION.md`](./XILEM_MIGRATION.md)

**Status:** migration Phases A through E are complete. The Xilem 0.4 preview
builds, all 19 tests pass, and Denali verified native PDF selection,
asynchronous on-demand rendering, navigation, zoom, N-up imposition, and backend
job-status notifications. Phase F (print options and downstream submission) is
next. See the migration plan for the remaining work.

### Display: Native Wayland
- Direct Wayland compositor access (via winit)
- Maximum performance and integration
- Future-proof architecture

### Document Rendering & PDF Assembly: Poppler + qpdf
- **poppler** (0.6) + **cairo-rs** (0.18) for PDF page rendering in the preview
- **qpdf** (0.3.5, in `boomaga-core`) for content-preserving PDF assembly/imposition output
- **Ghostscript dropped** — poppler + qpdf + the layout engine cover the residual
  functions (decision #4)

### IPC: Unix Domain Socket + versioned JSON
**Why**:
- Low-latency, local-only IPC for real-time backend→GUI updates
- Native Linux integration with systemd socket activation
- Better security than network sockets

**Note on D-Bus:** `zbus` / `zbus_systemd` is scoped to **systemd lifecycle only**
(socket activation / service management), **not** to IPC message transport (decision #3).
The message transport is Unix domain sockets carrying versioned JSON `Message`s.

## Complete Crate Architecture

```
boomaga-ipp/
├── Cargo.toml                          # Workspace manifest (resolver v2)
├── README.md
├── LICENSE                             # GPLv3
├── CLAUDE.md
├── docs/
│   ├── PROJECT_PLAN.md
│   ├── XILEM_MIGRATION.md
│   ├── HANDOFF.md
│   ├── SW-Reqrmnts-Spec--*.pdf         # SRS (latest == v0.2.2)
│   ├── User-Interface-Spec--*.pdf      # UIS (latest == v0.2.2)
│   └── uml/                            # code-conformant PlantUML (spec Appendix C)
├── openshell/
│   ├── create-bipp-sandbox.sh          # host-side sandbox provisioning
│   ├── BIPP-project-policy.yaml        # sandbox network/fs policy
│   └── README.md
└── scripts/

# Core crates (6 total; 2 emit binaries)
crates/
├── boomaga-core/                       # Core shared logic (+ qpdf)          [lib]
├── boomaga-ipp-backend/                # IPP Everywhere print service        [bin]
├── boomaga-preview/                    # GUI application (Xilem + poppler)   [bin]
├── boomaga-layout-engine/              # Imposition: N-up, booklet, transforms [lib]
├── boomaga-config/                     # Configuration management            [lib]
└── boomaga-ipc/                        # IPC library (Unix socket + JSON)    [lib]
```

*(There is no `boomaga-plugins` crate — decision #10.)*

## Implementation Phases (20 weeks total)

### Phase 1: Foundation (Weeks 1-4)
- Project setup and CI/CD
- Core infrastructure (config, error handling, IPC)
- IPP service skeleton
- Document processing pipeline

### Phase 2: Core Functionality (Weeks 5-8)
- Print job processing
- Xilem GUI through Phase E (native file-open, asynchronous rendering, N-up, and IPC)
- Layout engine (N-up, booklet)
- Document rendering

### Phase 3: Advanced Features (Weeks 9-12)
- Systemd integration
- Downstream printer management
- User experience enhancements
- Advanced features (watermarks, PDF export)

### Phase 4: Testing & Quality (Weeks 13-16)
- Unit testing (>90% coverage)
- Integration testing
- Performance optimization
- Security audit

### Phase 5: Deployment & Documentation (Weeks 17-20)
- Packaging (.deb, .rpm, Flatpak)
- Documentation completion
- Release preparation
- Launch

## Critical Dependencies

### Rust Crates (versions as pinned in workspace `Cargo.toml`)
- **poppler**: PDF rendering (0.6)
- **cairo-rs**: 2D surface for page rendering (0.18)
- **qpdf**: PDF assembly/imposition (0.3.5, in `boomaga-core`)
- **xilem**: GUI framework (0.4)
- **kurbo**: geometry/vector math (0.11)
- **winit**: windowing (0.30)
- **zbus**: D-Bus / systemd lifecycle only, *not* IPC transport (4.4)
- **nix**: Unix socket syscalls (0.29, in `boomaga-ipc` + backend)
- **tokio**: async runtime (1.35)
- **serde** / **serde_json**: serialization (1.0)
- **uuid**: `JobId` (1.0)
- **config** / **toml** / **directories**: configuration (`boomaga-config`)

### System Libraries
- libpoppler-dev / libpoppler-glib-dev (PDF rendering)
- libcairo2-dev (rendering surface)
- Wayland client/compositor libraries
- CUPS (host side, for driverless job ingress)

*(No Ghostscript / libghostscript-dev — decision #4.)*

## Critical Files

### Backend (`boomaga-ipp-backend`)
- `src/main.rs` - service entry, CLI parsing, init
- `src/server.rs` - `IppServer` (IPP operations, TCP listener)
- `src/job_processor.rs` - `JobProcessor` (worker loop)
- `src/job_queue.rs` - `JobQueue` (tokio mpsc + atomic size)

### GUI (`boomaga-preview`)
- `src/main.rs` - Xilem 0.4 GUI entry point and view tree
- `src/app.rs` - application state (`AppData`), imposition, and job-status handling
- `src/document_renderer.rs` - poppler + cairo rendering (real)
- `src/ipc_worker.rs` - Unix-socket notification worker and Xilem delivery

### Layout Engine (`boomaga-layout-engine`)
- `src/n_up.rs` - N-up layout (`NUpCalculator`)
- `src/booklet.rs` - booklet creation (`BookletCalculator`)
- `src/transforms.rs` - page transforms (`PageTransformer`)
- `src/imposition/layout_template.rs` - page templates

### Configuration (`boomaga-config`)
- `src/lib.rs` - `ConfigManager`
- `src/backend_config.rs` - backend/IPP settings
- `src/preview_config.rs` - GUI preferences
- `src/settings.rs` / `src/defaults.rs`

### IPC (`boomaga-ipc`)
- `src/protocol.rs` - `Message` types / JSON protocol
- `src/transport.rs` - framed, versioned Unix-socket JSON transport
- `src/d_bus.rs` - zbus lifecycle skeleton (systemd only)

## Step-by-Step Implementation

### Week 1: Project Setup
- Initialize Cargo workspace
- Create CI/CD pipeline
- Set up systemd templates
- Configure testing infrastructure

### Week 2: Core Infrastructure
- Configuration management (`boomaga-config`)
- Error types and handling (`boomaga-core`)
- IPC protocol messages + Unix-socket transport
- systemd lifecycle hooks (zbus_systemd)

### Week 3: IPP Foundation
- IPP service skeleton (receive-side)
- DNS-SD service registration
- Print job queue management
- Job status tracking

### Week 4: Document Processing
- PDF rendering with poppler
- PWG Raster / JPEG ingestion
- Document metadata extraction
- Page rendering pipeline

### Week 5: Print Job Processing
- IPP job reception (Create-Job / Send-Document)
- Job validation and error handling
- Queue persistence
- Cancellation support

### Week 6: GUI Foundation
- Xilem migration Phases A/B/C/D/E complete; Phase F print workflow next
  (see `XILEM_MIGRATION.md`)
- Main window (winit)
- Preview rendering
- Zoom and navigation

### Week 7: Layout Engine
- Page layout algorithms
- Booklet calculations
- Multi-page layouts
- Duplex printing

### Week 8: Document Rendering
- PDF pipeline completion
- High-quality preview
- Document merging (qpdf)

### Week 9-12: Advanced Features
- Systemd integration
- Downstream printer management (CUPS/IPP client — no `cups` dep yet)
- Print settings dialog
- Job queue UI
- Drag-and-drop support
- Advanced options (watermarks, headers/footers)

### Week 13-16: Testing
- Unit test coverage
- Integration testing
- Performance optimization
- Security audit

### Week 17-20: Deployment
- Packaging scripts
- Documentation
- Release preparation
- Launch

## Verification

### Testing Strategy
1. **Unit Tests**: Focus on core algorithms and rendering
2. **Integration Tests**: End-to-end IPP job flow
3. **Performance Tests**: Large document rendering
4. **Compatibility Tests**: Different compositors, paper sizes
5. **Security Tests**: IPC vulnerability assessment

### Manual Testing Checklist
- [ ] PDF rendering and preview
- [ ] PWG Raster / JPEG ingestion
- [ ] N-up printing (1, 2, 4, 8 pages/sheet)
- [ ] Booklet creation (A4, Letter)
- [ ] Multiple document merging
- [ ] Print settings dialog
- [ ] Systemd service lifecycle
- [ ] Downstream printer selection & submit

### Environment Requirements
- Debian/Ubuntu with systemd
- Wayland compositor (Hyprland, Sway, KDE, GNOME)
- CUPS with IPP support
- Development dependencies installed

## Success Criteria
1. Complete IPP Everywhere compliance (print-service ingress)
2. Native Wayland performance comparable to original
3. 90%+ test coverage for core functionality
4. Clean packaging for major distributions
5. Comprehensive documentation

## Implementation Status

> **Reality check (2026-07-22):** focused checks and tests for the four Phase E
> crates pass on Denali. A fresh workspace-wide `cargo check --workspace` has not
> yet been recorded, so this plan does not claim a completely green workspace.
> Percentages below remain estimates of *design + partial implementation*.

### Per-crate state

| Crate | Kind | State | Tests | Notes |
|-------|------|-------|-------|-------|
| `boomaga-core` | lib | Types complete; compiles | 0 | Plugin residue removed. `FileType` matches PDF/PWG Raster/JPEG. `parse_metadata()` is a TODO no-op. |
| `boomaga-config` | lib | Complete | 3 | `ConfigManager` wired; plugin settings removed. |
| `boomaga-layout-engine` | lib | Real & usable | 7 | N-up, booklet, transforms implemented; N-up partial-sheet behavior is tested. |
| `boomaga-preview` | bin | Phases A/B/C/D/E complete | 19 | Native PDF selection, asynchronous rendering, N-up preview, navigation/zoom, and IPC status updates verified on Denali. |
| `boomaga-ipc` | lib | Transport wired | 3 | Versioned newline-delimited JSON framing is used by backend and preview; focused tests pass. |
| `boomaga-ipp-backend` | bin | Scaffolded, partial | 1 | Queue/processor compile and emit ordered lifecycle notifications; real IPP parsing and downstream submission remain incomplete. |

### Phase 1: Foundation (Weeks 1-4) — 🚧 **~65%** (was reported 80%)

#### Completed ✅
- Workspace + 6-crate structure
- `boomaga-core` domain types, error handling
- `boomaga-config` configuration management (BackendConfig, PreviewConfig, Settings, ConfigManager)
- `boomaga-layout-engine` N-up, booklet, transforms (with unit tests)
- poppler + cairo PDF rendering in `boomaga-preview`
- IPP service scaffolding (`IppServer`, `JobQueue`, `JobProcessor`)

#### Remaining Phase 1 Tasks
- Record a fresh workspace-wide compiler/test baseline
- Complete IPP request parsing / response generation
- systemd socket activation (zbus_systemd) — not yet wired
- Expand core, IPC, and backend coverage beyond the focused Phase E tests

---

### Phase 2: Core Functionality (Weeks 5-8) — 🚧 **~35%**

#### Completed ✅
- Layout/imposition algorithms (in `boomaga-layout-engine`)
- PDF rendering pipeline foundation (poppler)
- IPC protocol message types defined

#### Remaining Phase 2 Tasks
- Phase F print-options dialog and downstream printer workflow
- Add booklet controls (deferred from the accepted Phase E N-up scope)
- Complete document-ready IPC and captured-document handoff
- Downstream submit path (CUPS/IPP client)

### Preview host verification (Denali, 2026-07-22)

```bash
cargo check -p boomaga-preview
cargo test -p boomaga-preview
cargo check -p boomaga-ipc
cargo test -p boomaga-ipc
cargo check -p boomaga-ipp-backend
cargo test -p boomaga-ipp-backend
cargo test -p boomaga-layout-engine
```

All focused checks passed. Denali reported 19 preview tests, 3 IPC tests,
1 backend test, and 7 layout-engine tests passing. Phase E preview behavior was
also reviewed interactively; the impossible `u16 > 65535` lint blocker was fixed.

---

### Phase 3: Advanced Features (Weeks 9-12) — 📋 **0%**
- Systemd integration
- Downstream printer management
- User experience enhancements
- Advanced features (watermarks, PDF export)

---

### Phase 4: Testing & Quality (Weeks 13-16) — 📋 **0%**
- Unit testing (>90% coverage)
- Integration testing
- Performance optimization
- Security audit

---

### Phase 5: Deployment & Documentation (Weeks 17-20) — 📋 **0%**
- Packaging (.deb, .rpm, Flatpak)
- Documentation completion
- Release preparation
- Launch

## Sources:
- [Boomaga Original System](https://github.com/Boomaga/boomaga)
- [Xilem - Modern Rust GUI Framework](https://github.com/linebender/xilem)
- [Poppler Rust Bindings](https://crates.io/crates/poppler)
- SRS/UIS **v0.2.2** Appendix C and [`docs/uml/`](./uml/) — authoritative architecture
