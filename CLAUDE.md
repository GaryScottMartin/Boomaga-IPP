# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

See @README.md for project overview and architecture

### Key Inter-crate Patterns

**Configuration Management** (boomaga-config):
- Separate modules for BackendConfig (IPP service settings) and PreviewConfig (GUI preferences)
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

**Plugin System**:
- Dynamic library loading via dyn traits
- Plugin API defines Plugin trait with metadata, initialize, execute, cleanup methods
- PluginRegistry manages plugin lifecycle and type registration

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

The project is in **Phase 1 (Foundation)** at 80% completion. Recent work has focused on:

- Resolving compilation errors (manifest parsing resolved, code compilation ~82 errors remaining)
- Systematic crate-by-crate dependency and import fixes
- Adding missing type exports to boomaga_core
- Fixing Eq trait issues with f64 fields

## Development Prerequisites

### Rust Toolchain
- Rust 1.70+ (currently 1.87.0)
- Cargo workspace with resolver v2

### System Dependencies
- libpoppler-dev (PDF rendering)
- libpoppler-cpp-dev (PDF API)
- libghostscript-dev (PostScript)
- CUPS development libraries
- Wayland development libraries (for GUI)

## Common Development Tasks

### Adding New Types to boomaga_core
1. Define type in `crates/boomaga-core/src/`
2. Export at crate level in `crates/boomaga-core/src/lib.rs`
3. Re-export in workspace root if needed
4. Update dependencies in relevant Cargo.toml files

### Fixing Compilation Errors
1. Check error message for crate location
2. Verify imports from boomaga_core exist and are public
3. Ensure Eq derives removed from f64 fields (PagePosition, TransformOperation)
4. Check workspace dependency specifications (use direct versions, not workspace refs)
5. Fix module organization and imports

### IPC Development
- Socket path defaults to `/tmp/boomaga-ipp.sock`
- Messages follow JSON serialization protocol
- Handle async I/O with tokio runtime

### Plugin Development
- Plugins use dynamic library (.so) format
- Implement Plugin trait from boomaga_plugins::core
- Plugin metadata includes name, version, description, plugin_type

## Important Files

### Core Infrastructure
- `crates/boomaga-core/src/lib.rs` - Main exports and module declarations
- `crates/boomaga-core/src/job.rs` - Job types and queue management
- `crates/boomaga-core/src/document.rs` - Document and page types
- `Cargo.toml` - Workspace configuration

### Backend Service
- `crates/boomaga-ipp-backend/src/main.rs` - Entry point and configuration
- `crates/boomaga-ipp-backend/src/server.rs` - IPP server implementation
- `crates/boomaga-ipp-backend/src/job_processor.rs` - Async job processing
- `crates/boomaga-ipp-backend/src/job_queue.rs` - Job persistence and management

### GUI Application
- `crates/boomaga-preview/src/main.rs` - Druid application entry
- `crates/boomaga-preview/src/app.rs` - Main application state
- `crates/boomaga-preview/src/viewer/document_view.rs` - Document viewer widget
- `crates/boomaga-preview/src/toolbar.rs` - Toolbar implementation

### Configuration
- `crates/boomaga-config/src/lib.rs` - Configuration module exports
- `crates/boomaga-config/src/backend.rs` - IPP service configuration
- `crates/boomaga-config/src/preview.rs` - GUI preferences

### Layout Engine
- `crates/boomaga-layout-engine/src/n_up.rs` - N-up layout calculations
- `crates/boomaga-layout-engine/src/booklet.rs` - Booklet creation algorithms
- `crates/boomaga-layout-engine/src/transforms.rs` - Page transformations

### Plugin System
- `crates/boomaga-plugins/src/core/plugin_api.rs` - Plugin trait definitions
- `crates/boomaga-plugins/src/core/loader.rs` - Dynamic library loading
- `crates/boomaga-plugins/src/loader.rs` - Main plugin system interface

## Known Issues

- Compilation errors remain ~82 across all crates
- boomaga-config has ~38 errors (missing type exports)
- boomaga-plugins has import and type resolution issues
- boomaga-ipp-backend needs Arc wrapping fixes for async patterns
- boomaga-ipc has channel type mismatches and zbus attribute issues

## Reference Material

- Project plan: `PROJECT_PLAN.md`
- User manual: `README.md`
- Original Boomaga: https://github.com/Boomaga/boomaga
- Druid GUI: https://github.com/linebender/druid
- Poppler: https://crates.io/crates/poppler
- CUPS IPP: https://www.cups.org/doc/spec-ipp.html

## Session Context Management

- Handoff summaries stored in `memory/handoff-session.md`
- Auto-memory in `memory/MEMORY.md` for stable patterns and decisions
- Use `/handoff` command before session end to save context
