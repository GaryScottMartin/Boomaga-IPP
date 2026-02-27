# Boomaga-IPP

**A modern virtual printer for Linux with native Wayland GUI and IPP Everywhere support**

Boomaga-IPP is a rewrite of the classic Boomaga (Booklet Manager) application, reimagined with modern Rust technology and native Wayland integration.

## Features

- **IPP Everywhere Support**: Full CUPS IPP protocol implementation
- **Native Wayland GUI**: Built with Druid for maximum performance and integration
- **Modern Document Rendering**: PDF and PostScript support using Poppler and Ghostscript
- **Advanced Layout Engine**: N-up printing, booklet creation, custom page layouts
- **Plugin System**: Extensible architecture for custom functionality
- **Systemd Integration**: Fully managed as a systemd service
- **Memory Safety**: Rust guarantees memory safety and thread safety

## Architecture

```
boomaga-ipp/
├── boomaga-core/          # Core shared logic
├── boomaga-ipp-backend/   # IPP server service
├── boomaga-preview/       # Wayland GUI application
├── boomaga-layout-engine/ # Page layout algorithms
├── boomaga-ipc/           # Inter-process communication
└── boomaga-plugins/       # Plugin system
```

## Building

### Prerequisites

- Rust 1.70 or later
- CUPS development libraries
- Poppler development libraries
- Ghostscript
- Wayland development libraries

### Building from source

```bash
# Clone the repository
git clone https://github.com/Boomaga/boomaga-ipp.git
cd boomaga-ipp

# Build all components
cargo build --release

# Install systemd service
sudo cp scripts/boomaga-ipp-backend.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable boomaga-ipp-backend
sudo systemctl start boomaga-ipp-backend
```

## Usage

### As a Virtual Printer

1. Install the backend service:
   ```bash
   sudo systemctl enable --now boomaga-ipp-backend
   ```

2. Use any application to print - the system will send jobs to boomaga

3. The preview application will automatically open for each job

### Manual Preview

```bash
# Run the preview application
boomaga-preview
```

### Command-line Options

**IPP Backend:**
```bash
boomaga-ipp-backend --socket /tmp/boomaga.sock --port 631
```

**Preview:**
```bash
boomaga-preview --window 1200x900
```

## Plugins

Boomaga-IPP supports a plugin system for extending functionality:

```rust
// Create a simple plugin
use boomaga_plugins::Plugin, PluginMetadata, PluginType;

struct MyPlugin;

impl Plugin for MyPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "MyPlugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A custom plugin".to_string(),
            plugin_type: PluginType::Utility,
            ..Default::default()
        }
    }

    fn initialize(&mut self, context: &PluginContext) -> Result<(), PluginError> {
        // Initialize plugin
        Ok(())
    }

    // ... other trait methods
}
```

## Development

### Project Structure
- `/docs`: Detailed documentation
- `/examples`: Example code and tests
- `/scripts`: Installation and packaging scripts

### Running Tests
```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p boomaga-core
cargo test -p boomaga-ipp-backend
```

### Code Style
The project follows the Rust community guidelines:
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Target Rust 2021 edition

## Licensing

GPL-3.0 License - See LICENSE file for details

## Contributing

Contributions are welcome! Please see CONTRIBUTING.md for guidelines.

## Roadmap

- [x] Project foundation
- [x] Core infrastructure
- [x] IPP server implementation
- [x] Preview application UI
- [x] Layout engine
- [x] Plugin system
- [ ] Comprehensive testing
- [ ] Documentation completion
- [ ] Distribution packages (.deb, .rpm, Flatpak)
- [ ] Plugin examples
- [ ] Advanced features (watermarks, PDF export)
- [ ] Performance optimization

## Troubleshooting

### Backend service not starting
```bash
# Check logs
journalctl -u boomaga-ipp-backend -f

# Verify socket path
ls -la /tmp/boomaga-ipp.sock
```

### Preview not opening
```bash
# Check IPC socket permissions
chmod 666 /tmp/boomaga-ipp.sock

# Test IPP endpoint
curl -v http://localhost:631
```

## Resources

- [Original Boomaga](https://github.com/Boomaga/boomaga)
- [Druid GUI Framework](https://github.com/linebender/druid)
- [Poppler Documentation](https://poppler.freedesktop.org/)
- [CUPS IPP Protocol](https://www.cups.org/doc/spec-ipp.html)

## Credits

Developed with ❤️ by the Boomaga-IPP Team

## Support

- Issues: https://github.com/Boomaga/boomaga-ipp/issues
- Discussions: https://github.com/Boomaga/boomaga-ipp/discussions
