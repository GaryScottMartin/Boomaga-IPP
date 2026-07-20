# Boomaga-IPP

**A virtual printer for Linux with native Wayland and IPP Everywhere support**

Boomaga-IPP is a rewrite of the classic Boomaga (Booklet Manager) application, reimagined with modern Rust technology and native Wayland integration.

## Features

- **IPP Everywhere Direction**: Driverless print-service ingress (implementation incomplete)
- **Native Wayland GUI**: Built with Xilem for maximum performance and integration
- **Modern Document Rendering**: PDF preview foundation using Poppler
- **Advanced Layout Engine**: N-up printing, booklet creation, custom page layouts
- **Systemd Integration**: Fully managed as a systemd service
- **Memory Safety**: Rust guarantees memory safety and thread safety

## Architecture

```
boomaga-ipp/
├── boomaga-core/          # Core shared logic
├── boomaga-ipp-backend/   # IPP server service
├── boomaga-preview/       # Wayland GUI application
├── boomaga-layout-engine/ # Page layout algorithms
├── boomaga-config/        # Configuration management
└── boomaga-ipc/           # Inter-process communication
```

## Building

### Prerequisites

- Rust 1.70 or later
- CUPS development libraries
- Poppler development libraries
- Wayland development libraries

### Building from source

```bash
# Clone the repository
git clone https://github.com/GaryScottMartin/Boomaga-IPP.git
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

## Project Status

The six-crate workspace remains under active development and does not yet build
as a whole because the backend and IPC crates retain stub/compile gaps. The
Xilem preview migration’s Phases A and B are complete and host-verified on
Denali; Phase C (the Masonry PDF canvas) is next. See
[`docs/HANDOFF.md`](docs/HANDOFF.md) for current session state and
[`docs/PROJECT_PLAN.md`](docs/PROJECT_PLAN.md) for detailed status.

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

# Host-verified preview checks
cargo check -p boomaga-preview
cargo test -p boomaga-preview
cargo run -p boomaga-preview
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

### Phase 1: Foundation (Weeks 1-4) - 🚧 In progress
- [x] Project foundation
- [x] Core infrastructure
- [ ] Complete IPP server implementation
- [x] Xilem preview Phases A/B
- [x] Layout engine
- [x] Configuration management system
- [x] PDF rendering foundation (Poppler)
- [ ] Comprehensive error handling
- [ ] Wire Unix-socket JSON IPC transport
- [ ] Systemd lifecycle integration
- [ ] Unit tests

### Phase 2: Core Functionality (Weeks 5-8) - 🚧 In progress
- [ ] Complete document rendering pipeline
- [ ] Complete Unix-socket JSON IPC integration
- [ ] Phase C Masonry PDF canvas
- [ ] Complete document viewer implementation
- [x] Navigation and zoom controls
- [ ] Print dialog UI

### Phase 3: Advanced Features (Weeks 9-12) - 📋 Planned
- [ ] Systemd integration
- [ ] Printer management
- [ ] User experience enhancements
- [ ] Watermarks, headers/footers
- [ ] PDF export

### Phase 4: Testing & Quality (Weeks 13-16) - 📋 Planned
- [ ] Unit testing (>90% coverage)
- [ ] Integration testing
- [ ] Performance optimization
- [ ] Security audit

### Phase 5: Deployment & Documentation (Weeks 17-20) - 📋 Planned
- [ ] Distribution packages (.deb, .rpm, Flatpak)
- [ ] Documentation completion
- [ ] Release preparation
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
- [Xilem GUI Framework](https://github.com/linebender/xilem)
- [Poppler Documentation](https://poppler.freedesktop.org/)
- [CUPS IPP Protocol](https://www.cups.org/doc/spec-ipp.html)

## Credits

Developed with ❤️ by the Boomaga-IPP Team
- @GaryScottMartin
- Claude Code / GLM-4.7-Flash

## Support

- Issues: https://github.com/GaryScottMartin/Boomaga-IPP/issues
- Discussions: https://github.com/GaryScottMartin/Boomaga-IPP/discussions
