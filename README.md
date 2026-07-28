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

- Rust 1.88 or later
- A C/C++ compiler and `pkg-config`
- `libglib2.0-dev`, `libcairo2-dev`, and `libpoppler-glib-dev`
- `libqpdf-dev` and `libclang-dev`
- Wayland client libraries for running the preview
- CUPS on the host for driverless print ingress
- CUPS client utilities (`lpstat` and `lp`) for downstream discovery/submission

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
# Open the preview without a document
boomaga-preview

# Open and render a PDF
boomaga-preview /path/to/document.pdf
```

### Command-line Options

**IPP Backend:**
```bash
boomaga-ipp-backend --socket /tmp/boomaga.sock --port 631
```

**Preview:**
```bash
boomaga-preview [--debug] [/path/to/document.pdf]
```

## Project Status

The six-crate workspace remains under active development. Xilem preview migration
Phases A through E are complete and host-verified on Denali, including native file
selection, asynchronous on-demand PDF rendering, navigation/zoom, 1/2/4/6/8-up
imposition, and backend job-status IPC. Phase F is in progress: the preview now
discovers CUPS destinations asynchronously, exposes copies/collate/duplex controls
bound to `PrintOptions`, and submits PDFs through `lp` without blocking the UI.
Denali verified real ET-3750 output, persistent submission status, simplex
collation (`123 123 123` versus `111 222 333`), and duplex set preservation.
The focused preview suite now passes 23 tests. A fresh Codex sandbox completed
`cargo check --workspace` with warnings and no errors on 2026-07-26, then
`cargo test --workspace` on 2026-07-27 with 37 tests passed and no failures.
The next Phase F step is a pure `SubmissionPlan`
for deterministic duplex sheet-range batching before the full print dialog. See
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
cargo run -p boomaga-preview -- /path/to/document.pdf
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
- [x] Xilem preview Phases A/B/C/D/E
- [x] Layout engine
- [x] Configuration management system
- [x] PDF rendering foundation (Poppler)
- [ ] Comprehensive error handling
- [x] Wire Unix-socket JSON IPC transport
- [ ] Systemd lifecycle integration
- [ ] Unit tests

### Phase 2: Core Functionality (Weeks 5-8) - 🚧 In progress
- [ ] Complete document rendering pipeline
- [x] Complete backend-to-preview job-status IPC integration
- [x] Phase C Masonry PDF canvas
- [x] Phase D file-open UI and asynchronous rendering
- [x] Phase E N-up imposition and job-status IPC
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
