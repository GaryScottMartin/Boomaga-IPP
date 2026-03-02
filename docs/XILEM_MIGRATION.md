# Xilem Migration Plan

## Overview
This document outlines the migration from Druid to Xilem for the Boomaga-IPP project.

## Rationale
Xilem is preferred over Druid because:
- **Native Rust GUI**: Zero FFI, full Rust ecosystem integration
- **Wayland-optimized**: Built for modern Linux compositors
- **Future-proof**: Active development by Rust community
- **Performance**: Zero-cost abstractions
- **Clean API**: Declarative UI similar to React

## Migration Strategy

### Phase 1: Documentation and Setup (Current)

#### 1.1 Update Cargo.toml Dependencies
```toml
# Remove Druid dependencies
# Remove cairo-rs (if not needed for other purposes)

# Add Xilem dependencies
[dependencies]
xilem = "0.4"
kurbo = { version = "0.11", features = ["use-glow"] }  # For rendering
winit = "0.30"  # Window management
env_logger = "0.11"  # Logging
```

#### 1.2 Update Project Documentation
- [x] This migration plan document
- [ ] Update README.md with new GUI approach
- [ ] Update architecture diagrams
- [ ] Update development setup instructions

### Phase 2: Core UI Components (2-3 days)

#### 2.1 Document Renderer (Poppler Integration)
**Current Status**: ✅ Complete
**Files to Keep**:
- `crates/boomaga-preview/src/document_renderer.rs`
- `crates/boomaga-preview/src/document_view.rs`

**Changes Needed**:
- Update rendering to work with Xilem's rendering primitives
- Implement Xilem's image rendering
- Ensure compatibility with new widget system

#### 2.2 Create Xilem Widget Structure
**New Files**:
- `crates/boomaga-preview/src/widgets/document_viewer.rs`
- `crates/boomaga-preview/src/widgets/page_renderer.rs`
- `crates/boomaga-preview/src/widgets/toolbar.rs`
- `crates/boomaga-preview/src/widgets/menu_bar.rs`

**Current Widgets to Port**:
- Toolbar (druid → xilem)
- Menu bar (druid → xilem)
- Page container (druid → xilem)
- Navigation buttons (druid → xilem)
- Zoom controls (druid → xilem)

#### 2.3 State Management
**Current**: Druid Data + Lens
**New**: Xilem's internal state handling + event handlers

**Example Transformation**:
```rust
// Druid (Current)
#[derive(Clone, Data, Lens)]
struct BoomagaApp {
    document_path: PathBuf,
    zoom_level: f64,
    // ...
}

// Xilem (Target)
struct BoomagaApp {
    document_path: PathBuf,
    zoom_level: f64,
    // No need for Lens derivation
    // State managed by Xilem's event loop
}

// Event handlers replace Lens-based updates
fn zoom_in_handler(app: &mut BoomagaApp, _ctx: &mut EventCtx) {
    app.zoom_level *= 1.2;
    request_paint();
}
```

### Phase 3: Document Viewer (2-3 days)

#### 3.1 Page Container Widget
**Requirements**:
- Display single page
- Handle zoom transformations
- Support page navigation
- Cached rendering for performance

**Implementation Plan**:
```rust
// New file: widgets/page_container.rs
struct PageContainer {
    page: Page,
    zoom: f64,
    cached_image: Option<ImageData>,
}

impl Widget for PageContainer {
    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, env: &Env) {
        // Implement Xilem rendering
        let transformed = transform_page(data.zoom);
        render_page(ctx, data.page, transformed);
    }
}
```

#### 3.2 Navigation Controls
**Features**:
- Previous/Next page buttons
- Page number display
- Jump to page input
- First/Last page shortcuts

### Phase 4: Toolbar & Menu (1-2 days)

#### 4.1 Toolbar Widget
**Components**:
- Open document button
- Print button
- Page navigation buttons
- Zoom controls (+/- buttons, zoom slider)
- Page info display

**Implementation**:
```rust
// New file: widgets/toolbar.rs
struct Toolbar {
    navigation_buttons: ButtonGroup,
    zoom_controls: Flex,
    page_info: Label,
}

impl Widget for Toolbar {
    fn event(&mut self, event: &Event, ctx: &mut EventCtx) {
        match event {
            Event::Click(button) => {
                handle_button_click(button, &mut self.state);
                ctx.request_paint();
            }
            _ => {}
        }
    }
}
```

#### 4.2 Menu Bar Widget
**Features**:
- File menu (Open, Print, Print Settings, Close)
- View menu (Navigation, Zoom)
- Print menu
- Help menu

### Phase 5: Document Loading & Rendering (1-2 days)

#### 5.1 Poppler Integration
**Current**: Druid-based rendering
**New**: Xilem-compatible rendering pipeline

**Workflow**:
1. Load PDF → PopplerDocument
2. Extract pages → boomaga_core::Page
3. Render to Cairo surface → Cache
4. Display in Xilem widget

#### 5.2 Error Handling
- Document loading errors
- File format errors
- Poppler rendering errors
- User-friendly error messages

### Phase 6: Testing & Refinement (2 days)

#### 6.1 Unit Tests
- Widget behavior tests
- Event handling tests
- Rendering accuracy tests

#### 6.2 Integration Tests
- Document loading workflow
- Page navigation
- Zoom operations
- Toolbar interactions

#### 6.3 Performance Testing
- Rendering speed
- Memory usage
- Cache effectiveness
- Frame rate

### Phase 7: Polish (1 day)

#### 7.1 Styling
- Modern, clean UI
- Consistent spacing
- Good contrast
- Hover effects

#### 7.2 Keyboard Shortcuts
- Page navigation (Space, N, P)
- Zoom controls (+, -, 0)
- File operations (Ctrl+O, Ctrl+P)

#### 7.3 Accessibility
- Keyboard navigation
- Screen reader support
- Focus management

## File Structure Changes

### Current Structure
```
crates/boomaga-preview/src/
├── main.rs              # Druid entry point
├── app.rs               # Druid app state
├── document_renderer.rs # Poppler integration
├── document_view.rs     # Document view widget
├── menu_bar.rs          # Menu bar (druid)
├── toolbar.rs           # Toolbar (druid)
├── print_dialog.rs      # Print dialog
└── settings_dialog.rs   # Settings dialog
```

### Target Xilem Structure
```
crates/boomaga-preview/src/
├── main.rs              # Xilem entry point
├── app.rs               # App state
├── document_renderer.rs # Poppler integration
├── widgets/
│   ├── mod.rs
│   ├── document_viewer.rs  # Main container
│   ├── page_container.rs   # Single page widget
│   ├── toolbar.rs           # Toolbar widget
│   ├── menu_bar.rs          # Menu bar widget
│   ├── navigation.rs        # Navigation controls
│   └── zoom_controls.rs     # Zoom controls
├── handlers/            # Event handlers
│   ├── mod.rs
│   ├── document.rs
│   ├── navigation.rs
│   └── zoom.rs
├── styles/              # Styling utilities
│   ├── mod.rs
│   └── colors.rs
└── util/                # Helper functions
    ├── mod.rs
    └── layout.rs
```

## Key Technical Differences

### Druid → Xilem Mapping

| Druid Concept | Xilem Equivalent |
|---------------|-----------------|
| `Data` trait | Value type (no trait needed) |
| `Lens` trait | Function-based access (no trait needed) |
| `Widget` trait | `Widget` trait (similar but simpler) |
| `EventCtx` | `EventCtx` (slightly different API) |
| `LifeCycle` | `LifeCycle` (similar) |
| `PaintCtx` | `PaintCtx` (similar) |
| `UpdateCtx` | `EventCtx` (events handle updates) |
| `WidgetPod` | Embedded widgets (no Pod needed) |
| `WindowState` | Widget state directly embedded |
| `WindowDesc` | `WindowConfig` (function-based) |
| `AppLauncher` | `Window` builder pattern |

### Event Handling
**Druid**:
```rust
fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Data, env: &Env)
```

**Xilem**:
```rust
fn event(&mut self, event: &Event, ctx: &mut EventCtx, data: &mut Data) -> bool
```

### Painting
**Druid**:
```rust
fn paint(&mut self, ctx: &mut PaintCtx, data: &Data, env: &Env)
```

**Xilem**:
```rust
fn paint(&mut self, ctx: &mut PaintCtx, data: &Data, env: &Env)
```

Similar, but Xilem has more aggressive caching.

### Layout
**Druid**: Complex constraint system
**Xilem**: Simpler, more intuitive layout system

## Risk Assessment

### High Risk
- **Poppler integration**: Existing implementation needs refactoring
- **State management**: Different approach to handle state
- **Performance**: Need to ensure smooth rendering

### Medium Risk
- **Widget behavior**: Xilem widgets may have different interaction patterns
- **Event handling**: New event system to learn
- **Styling**: New styling system to master

### Low Risk
- **Document loading**: Core logic can be reused
- **Configuration**: Not GUI-specific
- **Error handling**: Can be ported directly

## Success Criteria

### Functional
- [ ] Load and display PDF documents
- [ ] Navigate between pages
- [ ] Zoom in/out
- [ ] Print documents
- [ ] Use toolbar and menu bar

### Technical
- [ ] Compile without errors
- [ ] Run on Linux with Wayland
- [ ] Acceptable performance (< 100ms for page load)
- [ ] Minimal memory overhead (< 50MB for typical documents)

### Code Quality
- [ ] Follow Xilem patterns and conventions
- [ ] Good test coverage (> 70%)
- [ ] Clear, maintainable code
- [ ] Comprehensive documentation

## Timeline Estimate

| Phase | Duration | Status |
|-------|----------|--------|
| Phase 1: Setup & Docs | 0.5 days | 🔄 In Progress |
| Phase 2: Core Components | 2-3 days | ⏳ Pending |
| Phase 3: Document Viewer | 2-3 days | ⏳ Pending |
| Phase 4: Toolbar & Menu | 1-2 days | ⏳ Pending |
| Phase 5: Loading & Rendering | 1-2 days | ⏳ Pending |
| Phase 6: Testing | 2 days | ⏳ Pending |
| Phase 7: Polish | 1 day | ⏳ Pending |
| **Total** | **9-13 days** | |

## Next Steps

1. ✅ Complete Poppler integration documentation
2. ⏳ Fix boomaga-core compilation errors
3. ⏳ Create Xilem widget structure
4. ⏳ Port document viewer component
5. ⏳ Port toolbar and menu components
6. ⏳ Integrate with Poppler rendering
7. ⏳ Testing and refinement
