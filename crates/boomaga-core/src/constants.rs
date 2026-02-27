//! Application constants

pub const APP_NAME: &str = "Boomaga-IPP";
pub const APP_VERSION: &str = "0.1.0";
pub const APP_DESCRIPTION: &str = "Modern virtual printer for Linux with native Wayland GUI and IPP Everywhere support";

/// Default configuration paths
pub const CONFIG_DIR: &str = ".config/boomaga";
pub const CACHE_DIR: &str = ".cache/boomaga";
pub const STATE_DIR: &str = ".local/share/boomaga";

/// IPC socket path
pub const IPC_SOCKET_PATH: &str = "/tmp/boomaga-ipp.sock";

/// D-Bus service name
pub const DBUS_SERVICE_NAME: &str = "org.boomaga.IPP";

/// D-Bus object path
pub const DBUS_OBJECT_PATH: &str = "/org/boomaga/IPP";

/// IPP service port
pub const IPP_PORT: u16 = 631;

/// Default document thumbnail size
pub const THUMBNAIL_SIZE: (usize, usize) = (120, 120);

/// Default preview zoom levels
pub const ZOOM_LEVELS: [f64; 6] = [0.25, 0.5, 0.75, 1.0, 1.5, 2.0];

/// Maximum number of jobs to keep in history
pub const MAX_JOB_HISTORY: usize = 100;

/// Default timeout for operations
pub const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// Max concurrent job processing
pub const MAX_CONCURRENT_JOBS: usize = 4;

/// Worker threads for job processing
pub const WORKER_THREADS: usize = 2;

/// Job queue size
pub const JOB_QUEUE_SIZE: usize = 100;
