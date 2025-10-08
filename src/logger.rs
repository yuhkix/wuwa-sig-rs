use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::{execute, queue};
use std::io::{self, Write, stdout};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Log levels for structured logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 0,
    Warning = 1,
    Info = 2,
    Success = 3,
    Scan = 4,
    Hook = 5,
    Bypass = 6,
}

/// Configuration for the logger
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    pub min_level: LogLevel,
    pub show_timestamps: bool,
    pub show_thread_ids: bool,
    pub colored_output: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            min_level: LogLevel::Info,
            show_timestamps: false,
            show_thread_ids: false,
            colored_output: true,
        }
    }
}

/// High-performance structured logger with thread safety
pub struct Logger {
    config: Arc<Mutex<LoggerConfig>>,
    stdout: Arc<Mutex<std::io::Stdout>>,
    stderr: Arc<Mutex<std::io::Stderr>>,
}

impl Logger {
    /// Create a new logger with default configuration
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(LoggerConfig::default())),
            stdout: Arc::new(Mutex::new(stdout())),
            stderr: Arc::new(Mutex::new(io::stderr())),
        }
    }

    /// Create a new logger with custom configuration
    pub fn with_config(config: LoggerConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            stdout: Arc::new(Mutex::new(stdout())),
            stderr: Arc::new(Mutex::new(io::stderr())),
        }
    }

    /// Update the logger configuration
    pub fn set_config(&self, config: LoggerConfig) {
        if let Ok(mut current_config) = self.config.lock() {
            *current_config = config;
        }
    }

    /// Log a message with the specified level
    pub fn log(&self, level: LogLevel, msg: &str) {
        let config = match self.config.lock() {
            Ok(config) => config.clone(),
            Err(_) => return, // If we can't get the config, skip logging
        };

        if level > config.min_level {
            return;
        }

        let formatted_msg = self.format_message(level, msg, &config);

        match level {
            LogLevel::Error => self.log_to_stderr(&formatted_msg, &config),
            _ => self.log_to_stdout(&formatted_msg, level, &config),
        }
    }

    /// Format a log message with timestamp and level information
    fn format_message(&self, level: LogLevel, msg: &str, config: &LoggerConfig) -> String {
        let mut formatted = String::new();

        if config.show_timestamps {
            if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
                let timestamp = duration.as_secs();
                formatted.push_str(&format!("[{}] ", timestamp));
            }
        }

        if config.show_thread_ids {
            let thread_id = std::thread::current().id();
            formatted.push_str(&format!("[T{:?}] ", thread_id));
        }

        let level_str = match level {
            LogLevel::Error => "ERROR",
            LogLevel::Warning => "WARNING",
            LogLevel::Info => "INFO",
            LogLevel::Success => "SUCCESS",
            LogLevel::Scan => "SCAN",
            LogLevel::Hook => "HOOK",
            LogLevel::Bypass => "BYPASS",
        };

        formatted.push_str(&format!("[{}] {}", level_str, msg));
        formatted
    }

    /// Log to stdout with optional coloring
    fn log_to_stdout(&self, msg: &str, level: LogLevel, config: &LoggerConfig) {
        let mut stdout_guard = match self.stdout.lock() {
            Ok(guard) => guard,
            Err(_) => {
                // If we can't get the lock, just print to stdout directly
                println!("{}", msg);
                return;
            }
        };

        if config.colored_output {
            let color = self.get_color_for_level(level);
            let _ = execute!(
                *stdout_guard,
                SetForegroundColor(color),
                Print(msg),
                Print("\n"),
                ResetColor
            );
        } else {
            let _ = writeln!(*stdout_guard, "{}", msg);
        }
    }

    /// Log to stderr with optional coloring
    fn log_to_stderr(&self, msg: &str, config: &LoggerConfig) {
        let mut stderr_guard = match self.stderr.lock() {
            Ok(guard) => guard,
            Err(_) => {
                // If we can't get the lock, just print to stderr directly
                eprintln!("{}", msg);
                return;
            }
        };

        if config.colored_output {
            let _ = queue!(
                *stderr_guard,
                SetForegroundColor(Color::Red),
                Print(msg),
                Print("\n"),
                ResetColor
            );
            let _ = stderr_guard.flush();
        } else {
            let _ = writeln!(*stderr_guard, "{}", msg);
        }
    }

    /// Get the appropriate color for a log level
    fn get_color_for_level(&self, level: LogLevel) -> Color {
        match level {
            LogLevel::Error => Color::Red,
            LogLevel::Warning => Color::Yellow,
            LogLevel::Info => Color::Cyan,
            LogLevel::Success => Color::Green,
            LogLevel::Scan => Color::Yellow,
            LogLevel::Hook => Color::Magenta,
            LogLevel::Bypass => Color::Green,
        }
    }

    // Convenience methods for different log levels
    pub fn info_instance(&self, msg: &str) {
        self.log(LogLevel::Info, msg);
    }

    pub fn success_instance(&self, msg: &str) {
        self.log(LogLevel::Success, msg);
    }

    pub fn warning_instance(&self, msg: &str) {
        self.log(LogLevel::Warning, msg);
    }

    pub fn error_instance(&self, msg: &str) {
        self.log(LogLevel::Error, msg);
    }

    pub fn scan_instance(&self, msg: &str) {
        self.log(LogLevel::Scan, msg);
    }

    pub fn hook_instance(&self, msg: &str) {
        self.log(LogLevel::Hook, msg);
    }

    pub fn bypass_instance(&self, msg: &str) {
        self.log(LogLevel::Bypass, msg);
    }
}

// Global logger instance
static GLOBAL_LOGGER: std::sync::OnceLock<Logger> = std::sync::OnceLock::new();

/// Initialize the global logger
pub fn init_global_logger() {
    GLOBAL_LOGGER.set(Logger::new()).ok();
}

/// Initialize the global logger with custom configuration
pub fn init_global_logger_with_config(config: LoggerConfig) {
    GLOBAL_LOGGER.set(Logger::with_config(config)).ok();
}

/// Get the global logger instance
fn get_global_logger() -> &'static Logger {
    GLOBAL_LOGGER.get_or_init(|| Logger::new())
}

// Static convenience methods that use the global logger
impl Logger {
    pub fn info(msg: &str) {
        get_global_logger().log(LogLevel::Info, msg);
    }

    pub fn success(msg: &str) {
        get_global_logger().log(LogLevel::Success, msg);
    }

    pub fn warning(msg: &str) {
        get_global_logger().log(LogLevel::Warning, msg);
    }

    pub fn error(msg: &str) {
        get_global_logger().log(LogLevel::Error, msg);
    }

    pub fn scan(msg: &str) {
        get_global_logger().log(LogLevel::Scan, msg);
    }

    pub fn hook(msg: &str) {
        get_global_logger().log(LogLevel::Hook, msg);
    }

    pub fn bypass(msg: &str) {
        get_global_logger().log(LogLevel::Bypass, msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_levels() {
        assert!(LogLevel::Error < LogLevel::Warning);
        assert!(LogLevel::Info < LogLevel::Success);
        assert!(LogLevel::Scan < LogLevel::Hook);
    }

    #[test]
    fn test_logger_config_default() {
        let config = LoggerConfig::default();
        assert_eq!(config.min_level, LogLevel::Info);
        assert!(config.show_timestamps);
        assert!(!config.show_thread_ids);
        assert!(config.colored_output);
    }

    #[test]
    fn test_logger_creation() {
        let _logger = Logger::new();
        // Test that we can create a logger without panicking
        assert!(true);
    }

    #[test]
    fn test_logger_with_config() {
        let config = LoggerConfig {
            min_level: LogLevel::Error,
            show_timestamps: false,
            show_thread_ids: true,
            colored_output: false,
        };
        let _logger = Logger::with_config(config);
        // Test that we can create a logger with custom config
        assert!(true);
    }

    #[test]
    fn test_global_logger() {
        init_global_logger();
        // Test that we can initialize the global logger
        assert!(true);
    }

    #[test]
    fn test_static_logging_methods() {
        // These tests just ensure the functions don't panic
        Logger::info("Test info message");
        Logger::success("Test success message");
        Logger::warning("Test warning message");
        Logger::error("Test error message");
        Logger::scan("Test scan message");
        Logger::hook("Test hook message");
        Logger::bypass("Test bypass message");
    }

    #[test]
    fn test_log_level_filtering() {
        let config = LoggerConfig {
            min_level: LogLevel::Warning,
            show_timestamps: false,
            show_thread_ids: false,
            colored_output: false,
        };
        let logger = Logger::with_config(config);

        // These should not output anything due to level filtering
        logger.info_instance("This should not appear");
        logger.success_instance("This should not appear");

        // These should output
        logger.warning_instance("This should appear");
        logger.error_instance("This should appear");
    }
}
