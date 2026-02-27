//! IPC protocol messages

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use boomaga_core::{JobId, PrintOptions, PageSize, Error, Result};

/// Message type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Request message
    Request,
    /// Response message
    Response,
    /// Notification message
    Notification,
    /// Acknowledgment message
    Ack,
    /// Error message
    Error,
}

/// IPC message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message ID
    pub message_id: u64,
    /// Message type
    pub message_type: MessageType,
    /// Source
    pub source: MessageSource,
    /// Destination
    pub destination: MessageDestination,
    /// Payload
    pub payload: MessagePayload,
    /// Timestamp
    pub timestamp: i64,
}

/// Message source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageSource {
    /// Backend service
    Backend,
    /// Preview application
    Preview,
    /// IPC layer
    Ipc,
}

/// Message destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageDestination {
    /// Backend service
    Backend,
    /// Preview application
    Preview,
    /// All receivers
    Broadcast,
}

/// Message payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    /// Print job request
    PrintJobRequest {
        job_id: JobId,
        file_path: String,
        file_type: String,
        options: PrintOptions,
    },
    /// Print job status
    PrintJobStatus {
        job_id: JobId,
        status: String,
    },
    /// Document ready
    DocumentReady {
        document_id: String,
        page_count: usize,
    },
    /// Page rendered
    PageRendered {
        page_number: usize,
        image_data: Vec<u8>,
    },
    /// Printer info
    PrinterInfo {
        name: String,
        description: String,
        status: String,
    },
    /// Job queue update
    JobQueueUpdate {
        queue_size: usize,
        active_jobs: usize,
    },
    /// Configuration update
    ConfigUpdate {
        key: String,
        value: String,
    },
    /// Custom data
    Custom {
        data_type: String,
        data: Vec<u8>,
    },
}

impl Message {
    /// Create a new request message
    pub fn new_request(
        source: MessageSource,
        destination: MessageDestination,
        payload: MessagePayload,
    ) -> Self {
        let message_id = Self::generate_message_id();

        Self {
            message_id,
            message_type: MessageType::Request,
            source,
            destination,
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        }
    }

    /// Create a new response message
    pub fn new_response(
        message_id: u64,
        source: MessageSource,
        payload: MessagePayload,
    ) -> Self {
        Self {
            message_id,
            message_type: MessageType::Response,
            source,
            destination: MessageDestination::Preview, // Default
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        }
    }

    /// Create a new notification message
    pub fn new_notification(
        source: MessageSource,
        destination: MessageDestination,
        payload: MessagePayload,
    ) -> Self {
        let message_id = Self::generate_message_id();

        Self {
            message_id,
            message_type: MessageType::Notification,
            source,
            destination,
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        }
    }

    /// Generate unique message ID
    fn generate_message_id() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    /// Get message age
    pub fn age(&self) -> i64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        now - self.timestamp
    }

    /// Check if message is expired
    pub fn is_expired(&self, ttl: i64) -> bool {
        self.age() > ttl
    }
}

/// Request wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Request ID
    pub request_id: u64,
    /// Request type
    pub request_type: RequestType,
    /// Parameters
    pub parameters: HashMap<String, String>,
}

/// Request type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestType {
    /// Get printer info
    GetPrinterInfo,
    /// Get job queue
    GetJobQueue,
    /// Get job status
    GetJobStatus,
    /// Create job
    CreateJob,
    /// Cancel job
    CancelJob,
    /// Send document
    SendDocument,
    /// Close job
    CloseJob,
    /// Print document
    PrintDocument,
}

/// Response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Request ID
    pub request_id: u64,
    /// Response type
    pub response_type: ResponseType,
    /// Success flag
    pub success: bool,
    /// Data
    pub data: HashMap<String, String>,
    /// Error message
    pub error: Option<String>,
}

/// Response type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseType {
    /// Success
    Success,
    /// Error
    Error,
    /// Partial success
    Partial,
}

impl Response {
    /// Create a successful response
    pub fn success(request_id: u64, data: HashMap<String, String>) -> Self {
        Self {
            request_id,
            response_type: ResponseType::Success,
            success: true,
            data,
            error: None,
        }
    }

    /// Create an error response
    pub fn error(request_id: u64, error: String) -> Self {
        Self {
            request_id,
            response_type: ResponseType::Error,
            success: false,
            data: HashMap::new(),
            error: Some(error),
        }
    }
}
