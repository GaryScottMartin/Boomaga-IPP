//! Default configurations

pub const DEFAULT_IPP_PORT: u16 = 631;
pub const DEFAULT_IPC_SOCKET: &str = "/tmp/boomaga-ipp.sock";
pub const DEFAULT_DBUS_SERVICE: &str = "org.boomaga.IPP";
pub const DEFAULT_DBUS_PATH: &str = "/org/boomaga/IPP";
pub const DEFAULT_MAX_CONCURRENT_JOBS: usize = 4;
pub const DEFAULT_WORKER_THREADS: usize = 2;
pub const DEFAULT_JOB_QUEUE_SIZE: usize = 100;
pub const DEFAULT_QUEUE_TIMEOUT: u64 = 30;
pub const DEFAULT_JOB_TIMEOUT: u64 = 300;

pub const DEFAULT_WINDOW_WIDTH: u32 = 1200;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 800;

pub const DEFAULT_ZOOM_LEVEL: f64 = 1.0;
pub const DEFAULT_AUTO_ZOOM: bool = true;
pub const DEFAULT_AUTO_ZOOM_THRESHOLD: f64 = 0.95;

pub const DEFAULT_COPIES: u32 = 1;
pub const DEFAULT_COLLATE: bool = false;
pub const DEFAULT_DUPLEX: &'static str = "none";
pub const DEFAULT_ORIENTATION: &'static str = "portrait";
pub const DEFAULT_PAGES_PER_SHEET: u8 = 1;
pub const DEFAULT_MARGINS: &'static str = "normal";
pub const DEFAULT_SCALE: f64 = 1.0;

pub const DEFAULT_CACHE_SIZE_MB: u64 = 256;
pub const DEFAULT_CACHE_DIR: &str = ".cache/boomaga/pages";
pub const DEFAULT_CACHE_DIR_ABSOLUTE: &str = "/var/cache/boomaga/pages";

pub const DEFAULT_THUMBNAIL_SIZE: usize = 120;
pub const DEFAULT_PREVIEW_ZOOM_LEVELS: &[f64; 6] = [0.25, 0.5, 0.75, 1.0, 1.5, 2.0];
pub const DEFAULT_MAX_JOB_HISTORY: usize = 100;
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default configuration constants that can be accessed from other crates
pub mod constants {
    pub const APP_NAME: &str = "Boomaga-IPP";
    pub const APP_VERSION: &str = "0.1.0";
    pub const APP_DESCRIPTION: &str = "Modern virtual printer for Linux with native Wayland GUI and IPP Everywhere support";

    pub const CONFIG_DIR: &str = ".config/boomaga";
    pub const CACHE_DIR: &str = ".cache/boomaga";
    pub const STATE_DIR: &str = ".local/share/boomaga";

    pub const IPC_SOCKET_PATH: &str = DEFAULT_IPC_SOCKET;
    pub const DBUS_SERVICE_NAME: &str = DEFAULT_DBUS_SERVICE;
    pub const DBUS_OBJECT_PATH: &str = DEFAULT_DBUS_PATH;

    pub const IPP_PORT: u16 = DEFAULT_IPP_PORT;

    pub const MAX_JOB_HISTORY: usize = DEFAULT_MAX_JOB_HISTORY;
    pub const DEFAULT_TIMEOUT: std::time::Duration =
        std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS);

    pub const MAX_CONCURRENT_JOBS: usize = DEFAULT_MAX_CONCURRENT_JOBS;
    pub const WORKER_THREADS: usize = DEFAULT_WORKER_THREADS;
    pub const JOB_QUEUE_SIZE: usize = DEFAULT_JOB_QUEUE_SIZE;

    pub const THUMBNAIL_SIZE: (usize, usize) = (DEFAULT_THUMBNAIL_SIZE, DEFAULT_THUMBNAIL_SIZE);
    pub const ZOOM_LEVELS: [f64; 6] = *DEFAULT_PREVIEW_ZOOM_LEVELS;
}
