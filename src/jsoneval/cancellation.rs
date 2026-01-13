use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::fmt;

/// A thread-safe token that can be used to signal cancellation to running operations
#[derive(Clone, Debug)]
pub struct CancellationToken {
    is_cancelled: Arc<AtomicBool>,
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationToken {
    /// Create a new cancellation token
    pub fn new() -> Self {
        Self {
            is_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Signal cancellation
    pub fn cancel(&self) {
        self.is_cancelled.store(true, Ordering::SeqCst);
    }

    /// Check if cancellation has been requested
    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled.load(Ordering::SeqCst)
    }

    /// Return an error if cancelled
    pub fn check_cancelled(&self) -> Result<(), CancellationError> {
        if self.is_cancelled() {
            Err(CancellationError::Cancelled)
        } else {
            Ok(())
        }
    }
}

/// Error returned when an operation is cancelled
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancellationError {
    Cancelled,
}

impl fmt::Display for CancellationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CancellationError::Cancelled => write!(f, "Operation cancelled"),
        }
    }
}

impl std::error::Error for CancellationError {}

/// Helper type for results that can be cancelled
pub type CancellationResult<T> = Result<T, CancellationError>;
