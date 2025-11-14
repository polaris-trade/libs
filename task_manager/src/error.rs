use std::{borrow::Cow, time::Duration};
use thiserror::Error;

/// Error that occurs when running tasks through the TaskManager.
#[derive(Debug, Error)]
#[error("task '{task_name}' failed: {kind}")]
#[non_exhaustive]
pub struct TaskError {
    pub task_name: String,
    #[source]
    pub kind: TaskErrorKind,
}

impl TaskError {
    pub fn new(task_name: impl Into<String>, kind: TaskErrorKind) -> Self {
        Self {
            task_name: task_name.into(),
            kind,
        }
    }

    pub fn execution<E>(task_name: impl Into<String>, source: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::new(
            task_name,
            TaskErrorKind::Execution {
                source: source.into(),
            },
        )
    }

    pub fn shutdown<E>(task_name: impl Into<String>, source: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::new(
            task_name,
            TaskErrorKind::Shutdown {
                source: source.into(),
            },
        )
    }

    pub fn panic(task_name: impl Into<String>, message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(
            task_name,
            TaskErrorKind::Panic {
                message: message.into(),
            },
        )
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TaskErrorKind {
    #[error("execution error")]
    #[non_exhaustive]
    Execution {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("shutdown handler failed")]
    #[non_exhaustive]
    Shutdown {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("task panicked: {message}")]
    #[non_exhaustive]
    Panic { message: Cow<'static, str> },

    #[error("startup failed: {message}")]
    #[non_exhaustive]
    StartupFailed { message: Cow<'static, str> },
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ShutdownError {
    /// Operation timed out during shutdown.
    #[error("shutdown timed out after {timeout:?}")]
    Timeout { timeout: Duration },

    /// Multiple subsystems failed during shutdown.
    #[error("{} subsystem(s) failed during shutdown", .failures.len())]
    SubsystemsFailed { failures: Vec<TaskError> },

    /// Invalid core allocation configuration.
    #[error("invalid core allocation: {message}")]
    InvalidCoreAllocation { message: String },
}

impl ShutdownError {
    /// Create a timeout error
    pub fn timeout(timeout: Duration) -> Self {
        Self::Timeout { timeout }
    }

    /// Create a subsystems failed error
    pub fn subsystems_failed(failures: Vec<TaskError>) -> Self {
        Self::SubsystemsFailed { failures }
    }

    /// Create an invalid core allocation error
    pub fn invalid_core_allocation(message: impl Into<String>) -> Self {
        Self::InvalidCoreAllocation {
            message: message.into(),
        }
    }
}

impl From<TaskErrorKind> for TaskError {
    fn from(kind: TaskErrorKind) -> Self {
        Self {
            task_name: String::from("unknown"),
            kind,
        }
    }
}

pub type TaskResult<T> = Result<T, TaskError>;

pub type ShutdownResult<T> = Result<T, ShutdownError>;
