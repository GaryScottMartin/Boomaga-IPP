# boomaga-config

Configuration management crate for boomaga-ipp.

## Overview

The `boomaga-config` crate provides centralized configuration management for the boomaga virtual printer system, handling backend service settings, preview application preferences, and user settings.

## Features

- **Configuration Management**: Load, save, and validate configuration files
- **Default Values**: Automatic generation of sensible defaults
- **Environment Variables**: Support for environment-based configuration
- **Cross-platform**: Works on Linux with proper directory detection
- **Type-safe**: Strong typing with compile-time guarantees
- **Extensible**: Easy to add new configuration options

## Configuration Files

### Backend Configuration

Location: `~/.config/boomaga/backend.toml`

Contains settings for the IPP backend service:

```toml
# IPP Backend Configuration
[backend]
# Port number for IPP service
port = 631

# Number of worker threads
worker_threads = 2

# Maximum concurrent jobs
max_concurrent_jobs = 4

# Job queue size
job_queue_size = 100

# Job timeout in seconds
job_timeout = 300

# Enable debug logging
debug = false

# D-Bus service name
dbus_service = "org.boomaga.IPP"

# D-Bus object path
dbus_path = "/org/boomaga/IPP"
```

### Preview Configuration

Location: `~/.config/boomaga/preview.toml`

Contains settings for the preview application:

```toml
# Preview Application Configuration
[preview]
# Default zoom level
default_zoom = 1.0

# Auto-zoom to fit pages
auto_zoom = true

# Auto-zoom threshold
auto_zoom_threshold = 0.95

# Window width
window_width = 1200

# Window height
window_height = 800

# Default paper size
default_size = "A4"

# Default orientation
default_orientation = "portrait"

# Default page margins
default_margins = "normal"

# Default copies
default_copies = 1

# Default collate
default_collate = false

# Default duplex mode
default_duplex = "none"

# Default pages per sheet
default_pages_per_sheet = 1

# Default scale
default_scale = 1.0
```

### User Settings

Location: `~/.local/share/boomaga/settings.json`

Contains persistent user preferences:

```json
{
  "recent_files": [
    "/path/to/document.pdf",
    "/another/document.ps"
  ],
  "default_printer": "My Printer",
  "remember_last_directory": true,
  "toolbar_layout": [
    "file",
    "edit",
    "view",
    "print"
  ],
  "zoom_levels": [0.25, 0.5, 0.75, 1.0, 1.5, 2.0],
  "theme": "dark"
}
```

## API Reference

### `ConfigManager`

Main configuration manager that handles loading and saving configurations.

```rust
use boomaga_config::ConfigManager;

// Create configuration manager
let config = ConfigManager::new()?;

// Load backend configuration
let backend_config = config.load_backend()?;

// Load preview configuration
let preview_config = config.load_preview()?;

// Load user settings
let settings = config.load_settings()?;

// Save configuration
config.save_backend(&backend_config)?;
config.save_preview(&preview_config)?;
config.save_settings(&settings)?;
```

### `BackendConfig`

Backend service configuration.

```rust
use boomaga_config::BackendConfig;

let config = BackendConfig::default();

config.port = 631;
config.worker_threads = 4;
config.max_concurrent_jobs = 8;
config.validate()?;
```

### `PreviewConfig`

Preview application configuration.

```rust
use boomaga_config::PreviewConfig;

let config = PreviewConfig::default();

config.default_zoom = 1.25;
config.auto_zoom = true;
config.window_width = 1400;
config.validate()?;
```

### `Settings`

User settings and preferences.

```rust
use boomaga_config::Settings;

let settings = Settings::default();

settings.remember_last_directory = true;
settings.default_printer = "HP LaserJet";
settings.zoom_levels = vec![0.5, 0.75, 1.0, 1.25, 1.5, 2.0];
```

## Validation

All configuration types are validated when loaded:

```rust
config.validate()?;
```

Validation rules:
- Port must be between 1-65535
- Worker threads must be positive
- Max concurrent jobs must be positive
- Default zoom must be positive
- Window dimensions must be positive
- Paper size must be valid
- Orientation must be valid

## Environment Variables

Configuration can be overridden using environment variables:

```bash
export BOOMAGA_PORT=632
export BOOMAGA_WORKER_THREADS=4
export BOOMAGA_DEBUG=true
```

Variable names use snake_case and prefix with `BOOMAGA_`.

## Directory Structure

```
~/
├── .config/
│   └── boomaga/
│       ├── backend.toml    # Backend configuration
│       └── preview.toml    # Preview configuration
└── .local/
    └── share/
        └── boomaga/
            └── settings.json  # User settings
```

## Examples

### Initialize Configuration

```rust
use boomaga_config::initialize_config;

if let Ok(config) = initialize_config() {
    println!("Configuration initialized successfully");
}
```

### Custom Configuration Paths

```rust
use boomaga_config::ConfigManager;

// Use custom configuration directory
let config = ConfigManager::new()?;

// Customize paths
config.set_config_dir("/custom/config/dir");
config.set_cache_dir("/custom/cache/dir");
config.set_state_dir("/custom/state/dir");
```

### Watch for Configuration Changes

```rust
use std::fs;
use std::time::Duration;
use tokio::time::interval;

let config = ConfigManager::new()?;

// Watch for configuration changes
let mut watcher = fs::watch("/home/user/.config/boomaga")?;
watcher.watch(Interval::new(Duration::from_secs(5)))?;

loop {
    match watcher.recv() {
        Ok(event) => {
            println!("Config changed: {:?}", event);
            // Reload configuration
        }
        Err(e) => eprintln!("Watch error: {}", e),
    }
}
```

## Testing

```bash
cargo test -p boomaga-config

cargo test --lib -- --nocapture
```

## Changelog

### Version 0.1.0

- Initial release
- Configuration management for backend and preview
- User settings support
- Environment variable override
- Validation for all configuration types
- Cross-platform directory detection

## License

GPL-3.0 - See LICENSE file for details

## Contributing

Contributions are welcome! Please see CONTRIBUTING.md for guidelines.
