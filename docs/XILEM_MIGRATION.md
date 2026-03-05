# Xilem Migration Plan

## Overview
This document outlines the plan to replace Druid with Xilem for the Boomaga-IPP project.

## Motivation

### Why Xilem?
Xilem is chosen over Druid because:
- **Native Rust GUI**: Zero FFI, full Rust ecosystem integration
- **Wayland-optimized**: Built for modern Linux compositors
- **Future-proof**: Active development by Rust community
- **Performance**: Zero-cost abstractions
- **Clean API**: Declarative UI similar to React
- **Active Maintenance**: Druid is considered unmaintained

### Why Not Druid?
- Druid is **unmaintained** since mid-2023
- Community has moved to Xilem as the modern replacement
- Long-term viability concerns for production software
- Xilem provides better performance and modern features

## Current Status

**Framework in Use**: Druid 0.8 (to be replaced)
**Target Framework**: Xilem
**Date**: March 2026

The current preview application (`boomaga-preview`) uses Druid:
- **Location**: `crates/boomaga-preview/src/`
- **Entry Point**: `main.rs` (Druid-based)
- **Key Components**:
  - `app.rs` - Druid application state
  - `document_renderer.rs` - Poppler document rendering
  - `document_view.rs` - Document viewer widget
  - `menu_bar.rs` - Menu bar (Druid-based)
  - `toolbar.rs` - Toolbar (Druid-based)
  - `print_dialog.rs` - Print dialog
  - `settings_dialog.rs` - Settings dialog
  - `viewer/` - Document view components

**Druid Dependencies** (in workspace Cargo.toml):
```toml
druid = "0.8"
cairo-rs = "0.18"
```

## Migration Approach

**Note**: This is a **complete rewrite**, not a migration. The project will be rebuilt using Xilem from the start, as Druid is unmaintained and not suitable for long-term development.

## Migration Phases

### Phase 1: Project Foundation Setup (1-2 days)

#### 1.1 Update Workspace Dependencies

Update `Cargo.toml` workspace dependencies:

```toml
[workspace.dependencies]
# GUI - Replace Druid with Xilem
xilem = "0.4"
kurbo = { version = "0.11", features = ["use-glow"] }
winit = "0.30"

# Document Rendering - Keep Poppler
poppler = "0.6"

# Other dependencies - Keep as is
tokio = { workspace = true, features = ["full"] }
druid = "0.8"  # Keep for now, will remove after migration
cairo-rs = "0.18"
```

#### 1.2 Update Preview Crate Dependencies

Update `crates/boomaga-preview/Cargo.toml`:

```toml
[dependencies]
boomaga-core = { path = "../boomaga-core" }
boomaga-config = { path = "../boomaga-config" }

# Remove Druid dependencies
# Remove cairo-rs (if not needed for other purposes)

# Add Xilem dependencies
xilem = { workspace = true }
kurbo = { workspace = true }
winit = { workspace = true }

# Keep for now during transition
druid = { workspace = true }
cairo-rs = { workspace = true }

# Document and async dependencies
poppler = { workspace = true }
tokio = { workspace = true }
anyhow = { workspace = true }
serde = { workspace = true, features = ["derive"] }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

#### 1.3 Create Xilem Project Structure

Create new Xilem-based project structure:

```
crates/boomaga-preview/src/
├── main.rs              # Xilem entry point
├── app.rs               # Application state
├── document_renderer.rs # Poppler document rendering
├── widgets/             # Xilem widgets
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
├── util/                # Helper functions
│   ├── mod.rs
│   └── layout.rs
├── window.rs            # Window management
└── window_config.rs     # Window configuration
```

#### 1.4 Create Application State

Create new `app.rs` with Xilem-compatible state:

```rust
// crates/boomaga-preview/src/app.rs
use std::path::PathBuf;
use boomaga_core::{Document, Page, PageSize, Orientation};

#[derive(Clone)]
pub struct AppData {
    pub document: Option<Document>,
    pub current_page: usize,
    pub num_pages: usize,
    pub zoom: f64,
    pub is_loading: bool,
    pub error_message: Option<String>,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            document: None,
            current_page: 0,
            num_pages: 0,
            zoom: 1.0,
            is_loading: false,
            error_message: None,
        }
    }
}
```

### Phase 2: Core UI Components (3-4 days)

#### 2.1 Window Management

Create window setup:

```rust
// crates/boomaga-preview/src/window.rs
use xilem::WindowConfig;

pub fn create_window() -> WindowConfig<AppData> {
    WindowConfig::new()
        .title("Boomaga Preview")
        .size((1200, 800))
        .build()
}
```

#### 2.2 Document Renderer

Port Poppler integration to Xilem:

```rust
// crates/boomaga-preview/src/document_renderer.rs
use poppler::Document as PopplerDocument;
use boomaga_core::Document;

pub struct DocumentRenderer {
    // Poppler document reference
    // Rendering state
    // Cache management
}

impl DocumentRenderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load_document(&mut self, path: &PathBuf) -> Result<Document, Error> {
        // Implement Poppler document loading
        // Create boomaga_core::Document
        Ok(document)
    }

    pub fn render_page(&self, page_num: usize, zoom: f64) -> Result<ImageData, Error> {
        // Render page to image
        Ok(image_data)
    }
}
```

#### 2.3 Page Container Widget

Create single page widget:

```rust
// crates/boomaga-preview/src/widgets/page_container.rs
use kurbo::Vec2;
use xilem::{Widget, WidgetId, PaintCtx};

pub struct PageContainer {
    page_image: Option<ImageData>,
    transform: kurbo::Affine,
}

impl PageContainer {
    pub fn new(page_image: Option<ImageData>, zoom: f64) -> Self {
        // Calculate transform based on zoom
        Self {
            page_image,
            transform: calculate_transform(zoom),
        }
    }
}

impl Widget for PageContainer {
    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, env: &Env) {
        if let Some(image) = &self.page_image {
            ctx.draw_image(
                image.to_img_ctx(),
                self.transform,
            );
        }
    }
}
```

#### 2.4 Document Viewer Widget

Create main container:

```rust
// crates/boomaga-preview/src/widgets/document_viewer.rs
use xilem::{Widget, Flex, Label};
use crate::app::AppData;

pub struct DocumentViewer {
    page_container: PageContainer,
    page_info: Label,
    navigation_buttons: Flex,
    zoom_controls: Flex,
}

impl Widget for DocumentViewer {
    fn event(
        &mut self,
        event: &Event,
        ctx: &mut EventCtx,
        data: &mut AppData,
        env: &Env,
    ) -> bool {
        match event {
            Event::Click(button) => {
                match button {
                    NavigationButton::Prev => navigate_prev(data),
                    NavigationButton::Next => navigate_next(data),
                    ZoomButton::In => zoom_in(data),
                    ZoomButton::Out => zoom_out(data),
                }
                ctx.request_paint();
                true
            }
            _ => false,
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, env: &Env) {
        // Render document viewer
    }
}
```

### Phase 3: Toolbar & Menu (2-3 days)

#### 3.1 Toolbar Widget

Create toolbar with Xilem:

```rust
// crates/boomaga-preview/src/widgets/toolbar.rs
use xilem::{Widget, Flex, Button, Label};
use crate::app::AppData;

pub enum ToolbarEvent {
    OpenDocument,
    Print,
    ZoomIn,
    ZoomOut,
}

pub struct Toolbar {
    open_button: Button,
    print_button: Button,
    zoom_controls: Flex,
    page_info: Label,
}

impl Widget for Toolbar {
    fn event(
        &mut self,
        event: &Event,
        ctx: &mut EventCtx,
        data: &mut AppData,
        env: &Env,
    ) -> bool {
        match event {
            Event::Click(button) => match button {
                ToolbarEvent::OpenDocument => handle_open_document(ctx, data),
                ToolbarEvent::Print => handle_print(ctx, data),
                ToolbarEvent::ZoomIn => zoom_in(data),
                ToolbarEvent::ZoomOut => zoom_out(data),
            },
            _ => false,
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, env: &Env) {
        // Render toolbar
    }
}
```

#### 3.2 Menu Bar Widget

Create menu bar:

```rust
// crates/boomaga-preview/src/widgets/menu_bar.rs
use xilem::{Widget, Flex};
use crate::app::AppData;

pub struct MenuBar {
    file_menu: Flex,
    view_menu: Flex,
    help_menu: Flex,
}

impl Widget for MenuBar {
    fn event(
        &mut self,
        event: &Event,
        ctx: &mut EventCtx,
        data: &mut AppData,
        env: &Env,
    ) -> bool {
        match event {
            Event::MenuAction(action) => match action {
                MenuItem::OpenDocument => handle_open_document(ctx, data),
                MenuItem::Print => handle_print(ctx, data),
                MenuItem::Exit => ctx.close_window(),
                // ... other menu items
            },
            _ => false,
        }
    }
}
```

### Phase 4: Navigation & Zoom Controls (1-2 days)

#### 4.1 Navigation Widget

```rust
// crates/boomaga-preview/src/widgets/navigation.rs
pub enum NavigationButton {
    First,
    Prev,
    Next,
    Last,
}

pub struct NavigationWidget {
    prev_button: Button,
    next_button: Button,
    page_label: Label,
    jump_input: TextInput,
}
```

#### 4.2 Zoom Controls

```rust
// crates/boomaga-preview/src/widgets/zoom_controls.rs
pub enum ZoomButton {
    In,
    Out,
    Reset,
}

pub struct ZoomWidget {
    zoom_label: Label,
    zoom_in_button: Button,
    zoom_out_button: Button,
    zoom_reset_button: Button,
}
```

### Phase 5: Event Handling (2 days)

#### 5.1 Handler Module

```rust
// crates/boomaga-preview/src/handlers/document.rs
use crate::app::AppData;

pub fn handle_open_document(ctx: &mut EventCtx, data: &mut AppData) {
    // File dialog handling
}

pub fn handle_print(ctx: &mut EventCtx, data: &AppData) {
    // Print dialog handling
}

pub fn handle_document_loaded(result: Result<Document, Error>, data: &mut AppData) {
    // Update state on document load
}
```

#### 5.2 Navigation Handler

```rust
// crates/boomaga-preview/src/handlers/navigation.rs
pub fn navigate_prev(data: &mut AppData) {
    if data.current_page > 0 {
        data.current_page -= 1;
    }
}

pub fn navigate_next(data: &mut AppData) {
    if data.current_page < data.num_pages - 1 {
        data.current_page += 1;
    }
}
```

#### 5.3 Zoom Handler

```rust
// crates/boomaga-preview/src/handlers/zoom.rs
pub fn zoom_in(data: &mut AppData) {
    data.zoom *= 1.2;
}

pub fn zoom_out(data: &mut AppData) {
    data.zoom /= 1.2;
}

pub fn reset_zoom(data: &mut AppData) {
    data.zoom = 1.0;
}
```

### Phase 6: Document Loading Pipeline (2 days)

#### 6.1 Load PDF File

```rust
// crates/boomaga-preview/src/document_renderer.rs
pub fn load_document_from_file(path: &PathBuf) -> Result<Document, DocumentError> {
    let poppler_doc = PopplerDocument::new_from_file(path)?;
    let mut pages = Vec::new();

    for i in 0..poppler_doc.n_pages() {
        let page = poppler_doc.page(i)?;
        pages.push(extract_page(page)?);
    }

    Ok(Document {
        file_path: path.clone(),
        pages,
        file_type: FileType::Pdf,
    })
}
```

#### 6.2 Render to Image

```rust
// crates/boomaga-preview/src/document_renderer.rs
pub fn render_page_to_image(
    poppler_page: &PopplerPage,
    zoom: f64,
) -> Result<ImageData, RenderError> {
    let width = poppler_page.get_page_width() * zoom as f64;
    let height = poppler_page.get_page_height() * zoom as f64;

    let surface = CairoImageSurface::create(width as i32, height as i32)?;
    let context = CairoContext::new(&surface)?;

    // Draw page
    // Render to ImageData
    Ok(ImageData::new(surface))
}
```

### Phase 7: Error Handling (1-2 days)

#### 7.1 Error Types

```rust
// crates/boomaga-preview/src/errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DocumentError {
    #[error("Failed to open document: {0}")]
    OpenError(String),

    #[error("Invalid document format")]
    InvalidFormat,

    #[error("Page {0} not found")]
    PageNotFound(usize),

    #[error("Rendering error: {0}")]
    RenderError(String),
}
```

#### 7.2 Error Handling

Implement comprehensive error handling throughout the application.

### Phase 8: Testing (2-3 days)

#### 8.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_loading() {
        // Test document loading
    }

    #[test]
    fn test_page_navigation() {
        // Test navigation logic
    }

    #[test]
    fn test_zoom_operations() {
        // Test zoom functionality
    }
}
```

#### 8.2 Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    // Test full document loading workflow
    // Test UI interactions
}
```

### Phase 9: Polish & Optimization (2 days)

#### 9.1 Performance Optimization

- Implement rendering caching
- Optimize image loading
- Optimize zoom operations

#### 9.2 Keyboard Shortcuts

```rust
// Global event handler
Event::KeyDown(key) => match key {
    KeyCode::Space => navigate_next(data),
    KeyCode::Back => navigate_prev(data),
    KeyCode::Plus => zoom_in(data),
    KeyCode::Minus => zoom_out(data),
    KeyCode::Key0 => reset_zoom(data),
    // ...
}
```

#### 9.3 Accessibility

- Focus management
- Keyboard navigation
- Screen reader support

### Phase 10: Documentation & Cleanup (1-2 days)

#### 10.1 Update Documentation

- Update README.md
- Update CLAUDE.md
- Add migration documentation

#### 10.2 Remove Druid Code

- Remove all Druid dependencies
- Remove all Druid-specific code
- Update build scripts

#### 10.3 Update Architecture Diagrams

## File Structure

### Final Xilem Structure

```
crates/boomaga-preview/src/
├── main.rs              # Xilem entry point with window setup
├── app.rs               # Application state (AppData)
├── errors.rs            # Error types
├── window.rs            # Window configuration
├── document_renderer.rs # Poppler integration
├── widgets/
│   ├── mod.rs
│   ├── document_viewer.rs  # Main container
│   ├── page_container.rs   # Single page widget
│   ├── toolbar.rs           # Toolbar widget
│   ├── menu_bar.rs          # Menu bar widget
│   ├── navigation.rs        # Navigation controls
│   └── zoom_controls.rs     # Zoom controls
├── handlers/
│   ├── mod.rs
│   ├── document.rs
│   ├── navigation.rs
│   └── zoom.rs
├── styles/
│   ├── mod.rs
│   └── colors.rs
└── util/
    ├── mod.rs
    └── layout.rs
```

## Key Technical Details

### Xilem vs Druid

| Druid Concept | Xilem Equivalent |
|---------------|------------------|
| `Data` trait | Value type (no trait needed) |
| `Lens` trait | Function-based access (no trait needed) |
| `Widget` trait | `Widget` trait (similar but simpler) |
| `EventCtx` | `EventCtx` (slightly different API) |
| `LifeCycle` | `LifeCycle` (similar) |
| `PaintCtx` | `PaintCtx` (similar) |
| `WindowDesc` | `WindowConfig` (function-based) |
| `AppLauncher` | `Window` builder pattern |

### Event Handling

```rust
fn event(
    &mut self,
    event: &Event,
    ctx: &mut EventCtx,
    data: &mut AppData,
    env: &Env,
) -> bool {
    // Return true to consume event
    true
}
```

### Painting

```rust
fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, env: &Env) {
    // Use Xilem's drawing APIs
    ctx.draw_text(...);
    ctx.draw_image(...);
}
```

### Layout

Xilem uses a simpler, more intuitive layout system based on Flexbox concepts.

## Success Criteria

### Functional Requirements
- [ ] Load and display PDF documents
- [ ] Navigate between pages (next, previous, first, last)
- [ ] Zoom in/out and reset
- [ ] Print documents
- [ ] Use toolbar and menu bar
- [ ] Keyboard shortcuts (Space, N, P, +/-, 0)

### Technical Requirements
- [ ] Compile without errors
- [ ] Run on Linux with Wayland
- [ ] Acceptable performance (< 100ms for page load)
- [ ] Minimal memory overhead (< 50MB for typical documents)
- [ ] Smooth zoom operations
- [ ] Responsive UI

### Code Quality Requirements
- [ ] Follow Xilem patterns and conventions
- [ ] Good test coverage (> 70%)
- [ ] Clear, maintainable code
- [ ] Comprehensive error handling
- [ ] Documentation comments
- [ ] No Druid dependencies or code

## Timeline Estimate

| Phase | Duration | Description |
|-------|----------|-------------|
| Phase 1: Foundation | 1-2 days | Setup dependencies, structure, app state |
| Phase 2: Core UI | 3-4 days | Document renderer, page container, viewer |
| Phase 3: Toolbar & Menu | 2-3 days | Toolbar widget, menu bar widget |
| Phase 4: Navigation & Zoom | 1-2 days | Navigation controls, zoom controls |
| Phase 5: Event Handling | 2 days | Handler modules for all events |
| Phase 6: Document Loading | 2 days | PDF loading, rendering pipeline |
| Phase 7: Error Handling | 1-2 days | Error types, error handling |
| Phase 8: Testing | 2-3 days | Unit tests, integration tests |
| Phase 9: Polish | 2 days | Performance, keyboard shortcuts, accessibility |
| Phase 10: Documentation | 1-2 days | Update docs, remove Druid code |
| **Total** | **17-24 days** | Approx. 3-4 weeks |

## Risk Assessment

### High Risk
- **Complete rewrite**: This is not a migration but a complete rebuild
- **Learning curve**: Team needs to learn Xilem patterns
- **Poppler integration**: Must ensure rendering quality and performance
- **State management**: New approach to handle application state

### Medium Risk
- **Performance**: Must ensure smooth rendering with Xilem
- **Widget behavior**: Xilem widgets have different interaction patterns
- **Event handling**: New event system to master
- **Testing**: Ensuring comprehensive test coverage

### Low Risk
- **Document loading**: Core logic can be reused
- **Configuration**: Not GUI-specific
- **Error handling**: Can be implemented from scratch

## Next Steps

1. ✅ Review and approve this migration plan
2. 🚧 Remove Druid dependencies from workspace Cargo.toml
3. 🚧 Create new Xilem-based project structure
4. 🚧 Implement Phase 1: Foundation setup
5. 🚧 Implement Phase 2: Core UI components
6. 🚧 Implement Phase 3: Toolbar & Menu
7. 🚧 Implement Phase 4: Navigation & Zoom
8. 🚧 Implement Phase 5: Event Handling
9. 🚧 Implement Phase 6: Document Loading
10. 🚧 Implement Phase 7: Error Handling
11. 🚧 Implement Phase 8: Testing
12. 🚧 Implement Phase 9: Polish
13. 🚧 Implement Phase 10: Documentation & Cleanup
14. ✅ Test and deploy Xilem-based preview application
