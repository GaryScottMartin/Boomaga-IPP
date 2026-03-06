//! Application constants

pub const APP_NAME: &str = "Boomaga-IPP";
pub const APP_VERSION: &str = "0.1.0";
pub const APP_DESCRIPTION: &str = "Modern virtual printer for Linux with native Wayland GUI and IPP Everywhere support";

/// Default configuration paths
pub const CONFIG_DIR: &str = ".config/boomaga";
pub const CACHE_DIR: &str = ".cache/boomaga";
pub const STATE_DIR: &str = ".local/share/boomaga";

/// IPC socket path
pub const DEFAULT_IPC_SOCKET: &str = "/tmp/boomaga-ipp.sock";

/// D-Bus service name
pub const DEFAULT_DBUS_SERVICE: &str = "org.boomaga.IPP";

/// D-Bus object path
pub const DEFAULT_DBUS_PATH: &str = "/org/boomaga/IPP";
pub const DBUS_OBJECT_PATH: &str = DEFAULT_DBUS_PATH;

/// IPP service port
pub const DEFAULT_IPP_PORT: u16 = 631;
pub const IPP_PORT: u16 = DEFAULT_IPP_PORT;

/// Default document thumbnail size
pub const DEFAULT_THUMBNAIL_SIZE: (usize, usize) = (120, 120);

/// Default preview zoom levels
pub const DEFAULT_PREVIEW_ZOOM_LEVELS: [f64; 6] = [0.25, 0.5, 0.75, 1.0, 1.5, 2.0];

/// Maximum number of jobs to keep in history
pub const DEFAULT_MAX_JOB_HISTORY: usize = 100;

/// Default timeout for operations
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Max concurrent job processing
pub const DEFAULT_MAX_CONCURRENT_JOBS: usize = 4;

/// Worker threads for job processing
pub const DEFAULT_WORKER_THREADS: usize = 2;

/// Job queue size
pub const DEFAULT_JOB_QUEUE_SIZE: usize = 100;

// Backward compatibility aliases
pub const IPC_SOCKET_PATH: &str = DEFAULT_IPC_SOCKET;
pub const DBUS_SERVICE_NAME: &str = DEFAULT_DBUS_SERVICE;
pub const MAX_CONCURRENT_JOBS: usize = DEFAULT_MAX_CONCURRENT_JOBS;
pub const WORKER_THREADS: usize = DEFAULT_WORKER_THREADS;
pub const JOB_QUEUE_SIZE: usize = DEFAULT_JOB_QUEUE_SIZE;

// Backward compatibility aliases
pub const THUMBNAIL_SIZE: (usize, usize) = DEFAULT_THUMBNAIL_SIZE;
pub const ZOOM_LEVELS: [f64; 6] = [0.25, 0.5, 0.75, 1.0, 1.5, 2.0];
pub const MAX_JOB_HISTORY: usize = DEFAULT_MAX_JOB_HISTORY;

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub ipc_socket_path: String,
    pub dbus_service_name: String,
    pub dbus_path: String,
    pub ipp_port: u16,
    pub max_job_history: usize,
    pub timeout_secs: u64,
    pub max_concurrent_jobs: usize,
    pub worker_threads: usize,
    pub job_queue_size: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ipc_socket_path: DEFAULT_IPC_SOCKET.to_string(),
            dbus_service_name: DEFAULT_DBUS_SERVICE.to_string(),
            dbus_path: DEFAULT_DBUS_PATH.to_string(),
            ipp_port: DEFAULT_IPP_PORT,
            max_job_history: DEFAULT_MAX_JOB_HISTORY,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            max_concurrent_jobs: DEFAULT_MAX_CONCURRENT_JOBS,
            worker_threads: DEFAULT_WORKER_THREADS,
            job_queue_size: DEFAULT_JOB_QUEUE_SIZE,
        }
    }
}
