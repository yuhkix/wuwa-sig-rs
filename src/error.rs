use std::fmt;

#[derive(Debug, Clone)]
pub enum AppError {
    ModuleNotFound(String),
    ModuleInfoFailed,
    PatternNotFound,
    StringConversion,
    PanicRecovery,
    HookFailed(String),
    ConsoleInitFailed,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ModuleNotFound(name) => write!(f, "Module '{}' not found", name),
            AppError::ModuleInfoFailed => write!(f, "Failed to get module information"),
            AppError::PatternNotFound => write!(f, "Pattern not found in memory"),
            AppError::StringConversion => write!(f, "String conversion failed"),
            AppError::PanicRecovery => write!(f, "Panic recovery in unsafe code"),
            AppError::HookFailed(msg) => write!(f, "Hook application failed: {}", msg),
            AppError::ConsoleInitFailed => write!(f, "Console initialization failed"),
        }
    }
}

impl std::error::Error for AppError {}

pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = AppError::ModuleNotFound("test.exe".to_string());
        assert_eq!(format!("{}", error), "Module 'test.exe' not found");
    }

    #[test]
    fn test_error_chain() {
        let error = AppError::PatternNotFound;
        assert!(std::error::Error::source(&error).is_none());
    }
}
