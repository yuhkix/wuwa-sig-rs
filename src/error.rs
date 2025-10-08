use thiserror::Error;

/// Application error types with automatic Display and Error trait implementations
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Module '{name}' not found")]
    ModuleNotFound { name: String },

    #[error("Failed to get module information: {source}")]
    ModuleInfoFailed {
        #[source]
        source: std::io::Error,
    },

    #[error("Pattern not found in memory (searched {size} bytes)")]
    PatternNotFound { size: usize },

    #[error("String conversion failed: {details}")]
    StringConversion { details: String },

    #[error("Panic recovery in unsafe code: {reason}")]
    PanicRecovery { reason: String },

    #[error("Hook application failed: {message}")]
    HookFailed { message: String },

    #[error("Console initialization failed: {source}")]
    ConsoleInitFailed {
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid configuration: {field} - {reason}")]
    InvalidConfig { field: String, reason: String },

    #[error("Memory access violation at address {address:#x}")]
    MemoryAccessViolation { address: usize },
}

pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = AppError::ModuleNotFound {
            name: "test.exe".to_string(),
        };
        assert_eq!(format!("{}", error), "Module 'test.exe' not found");
    }

    #[test]
    fn test_error_chain() {
        let error = AppError::PatternNotFound { size: 1024 };
        assert!(std::error::Error::source(&error).is_none());
    }

    #[test]
    fn test_error_with_source() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let error = AppError::ModuleInfoFailed { source: io_error };
        assert!(std::error::Error::source(&error).is_some());
    }
}
