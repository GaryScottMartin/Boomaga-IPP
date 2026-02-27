# Modern Boomaga Virtual Printer - Implementation Plan

## Context
Need to create a modern version of boomaga (BOOklet MANager), a virtual printer for Linux. The original C++ implementation has good features but is outdated, using CUPS + D-Bus + Ghostscript. Requirements: systemd-managed Linux, Wayland display, IPP Everywhere printing, Rust language implementation, memory-safe and type-safe.

## Requirements Summary
- **Language**: Rust
- **Display**: Native Wayland
- **Print System**: IPP Everywhere (CUPS IPP)
- **Process Management**: systemd-managed
- **License**: GPLv3
- **Features**: Full-featured UI with plugin system
- **Supported Formats**: PDF and PostScript

## Architecture Overview

### Components
1. **Backend Service** (systemd service)
   - Receives print jobs via IPP
   - Processes and transforms pages
   - Spawns preview window

2. **Preview Application** (Wayland GUI)
   - Document viewer with annotations and bookmarks
   - Print controls (N-up, booklet, etc.)
   - Print dialog
   - Plugin system support

3. **Shared Components**
   - IPP parser
   - Page layout engine (N-up, booklet)
   - PDF/PostScript rendering
   - Document manipulation

### Data Flow
1. User selects printer → CUPS
2. CUPS creates IPP job → Boomaga backend service
3. Backend processes document → Generates preview
4. Backend spawns preview window with document
5. User reviews → Clicks print
6. User prints → Uses system printer

## Technology Stack Decisions

### GUI Framework: Druid
**Why Druid over alternatives**:
- Mature Wayland backend support
- Strong Rust integration
- Desktop-optimized architecture
- Good performance for document rendering
- Data-driven approach suits document viewers

### Display: Native Wayland
**Why not Xwayland**:
- Better performance and integration
- Direct Wayland API access
- Modern compositor support
- Future-proof architecture

### Document Rendering: Poppler + Ghostscript
- Poppler for PDF rendering (industry standard)
- Ghostscript for PostScript rendering
- Cairo for 2D graphics surface handling

### IPC: Unix Domain Socket + D-Bus
**Why this combination**:
- Low-latency IPC for real-time updates
- Native Linux integration
- Systemd socket activation support
- Better security than network sockets

### Plugin System: Dynamic Libraries with dyn traits
**Implementation approach**:
- Runtime extensibility
- Clear API boundaries
- Easy distribution and update
- Compatible with Rust's type system

## Complete Crate Architecture

```
boomaga-ipp/
├── Cargo.toml                          # Workspace manifest
├── README.md
├── LICENSE                            # GPLv3
├── CHANGELOG.md
├── CONTRIBUTING.md
├── docs/
│   ├── architecture.md
│   ├── design-decisions.md
│   └── api-documentation/
├── examples/
│   ├── ipp-client-test/
│   └── layout-algorithms/
├── tests/
│   ├── integration/
│   └── performance/
└── scripts/
    ├── install-systemd.sh
    ├── setup-debian.sh
    └── create-distribution-packages.sh

# Core crates
crates/
├── boomaga-core/                      # Core shared logic
├── boomaga-ipp-backend/               # IPP service
├── boomaga-preview/                   # GUI application
├── boomaga-layout-engine/             # Page layout algorithms
├── boomaga-ipc/                       # IPC library
└── boomaga-plugins/                   # Plugin system
```

## Implementation Phases (20 weeks total)

### Phase 1: Foundation (Weeks 1-4)
- Project setup and CI/CD
- Core infrastructure (config, error handling, IPC)
- IPP server skeleton
- Document processing pipeline

### Phase 2: Core Functionality (Weeks 5-8)
- Print job processing
- GUI foundation with Druid
- Layout engine (N-up, booklet)
- Document rendering

### Phase 3: Advanced Features (Weeks 9-12)
- Systemd integration
- Printer management
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

### Rust Crates
- **poppler**: PDF rendering (v0.27)
- **druid**: GUI framework (v0.8)
- **zbus**: D-Bus communication (v4.3)
- **tokio**: Async runtime (v1.35)
- **serde**: Serialization (v1.0)

### System Libraries
- libpoppler-dev (PDF rendering)
- libpoppler-cpp-dev (PDF API)
- libghostscript-dev (PostScript)

## Critical Files

### Backend
- `crates/boomaga-ipp-backend/src/main.rs` - IPP service entry
- `crates/boomaga-ipp-backend/src/server.rs` - IPP server
- `crates/boomaga-ipp-backend/src/job_processor.rs` - Job handling

### GUI
- `crates/boomaga-preview/src/main.rs` - GUI entry point
- `crates/boomaga-preview/src/app.rs` - Main application
- `crates/boomaga-preview/src/viewer/document_view.rs` - Document viewer

### Layout Engine
- `crates/boomaga-layout-engine/src/n_up.rs` - N-up layout
- `crates/boomaga-layout-engine/src/booklet.rs` - Booklet creation
- `crates/boomaga-layout-engine/src/imposition/template.rs` - Page templates

### IPC
- `crates/boomaga-ipc/src/protocol/messages.rs` - Message types
- `crates/boomaga-ipc/src/transport/unix_socket.rs` - Socket transport

### Plugin System
- `crates/boomaga-plugins/src/core/plugin_api.rs` - Plugin interfaces
- `crates/boomaga-plugins/src/core/loader.rs` - Dynamic loading

## Step-by-Step Implementation

### Week 1: Project Setup
- Initialize Cargo workspace
- Create CI/CD pipeline
- Set up systemd templates
- Configure testing infrastructure

### Week 2: Core Infrastructure
- Configuration management
- Error types and handling
- D-Bus interface definitions
- IPC protocol messages
- Basic IPC transport

### Week 3: IPP Foundation
- IPP server skeleton
- DNS-SD service registration
- Print job queue management
- Job status tracking

### Week 4: Document Processing
- PDF rendering with poppler
- PostScript parsing
- Document metadata extraction
- Page rendering pipeline

### Week 5: Print Job Processing
- IPP job reception
- Job validation and error handling
- Queue persistence
- Cancellation support

### Week 6: GUI Foundation
- Druid with Wayland backend
- Main window
- Preview rendering
- Zoom and navigation

### Week 7: Layout Engine
- Page layout algorithms
- Booklet calculations
- Multi-page layouts
- Duplex printing

### Week 8: Document Rendering
- PDF pipeline completion
- PostScript support
- High-quality preview
- Document merging

### Week 9-12: Advanced Features
- Systemd integration
- Printer management
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
- [ ] PostScript rendering
- [ ] N-up printing (1, 2, 4, 8 pages/sheet)
- [ ] Booklet creation (A4, Letter)
- [ ] Multiple document merging
- [ ] Print settings dialog
- [ ] Systemd service lifecycle
- [ ] Plugin loading and execution

### Environment Requirements
- Debian/Ubuntu with systemd
- Wayland compositor (Hyprland, Sway, KDE, GNOME)
- CUPS with IPP support
- Development dependencies installed

## Success Criteria
1. Complete IPP Everywhere compliance
2. Native Wayland performance comparable to original
3. 90%+ test coverage for core functionality
4. Clean packaging for major distributions
5. Comprehensive documentation
6. Plugin ecosystem with 3+ sample plugins

## Implementation Status

### Completed ✅
- Project structure and workspace setup
- Core crate (boomaga-core) with error handling, job types, document types
- IPP backend service with job queue and processor
- Preview application with Druid GUI framework
- Layout engine with N-up and booklet algorithms

### In Progress 🚧
- Document rendering with poppler
- IPC layer implementation
- Plugin system

### Planned 📋
- Systemd service configuration
- D-Bus integration
- Advanced features (watermarks, PDF export)
- Comprehensive testing
- Documentation and packaging

## Sources:
- [Boomaga Original System](https://github.com/Boomaga/boomaga)
- [Druid Rust GUI Framework](https://github.com/linebender/druid)
- [Poppler Rust Bindings](https://crates.io/crates/poppler)
