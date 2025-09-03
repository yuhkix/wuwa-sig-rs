use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::{execute, queue};
use std::io::{self, Write, stdout};

pub struct Logger;

impl Logger {
    fn log_with_color(color: Color, prefix: &str, msg: &str) {
        let _ = execute!(
            stdout(),
            SetForegroundColor(color),
            Print(format!("[{}] ", prefix)),
            ResetColor,
            Print(format!("{}\n", msg))
        );
    }

    fn log_error_with_color(color: Color, prefix: &str, msg: &str) {
        let mut stderr = io::stderr();
        let _ = queue!(
            stderr,
            SetForegroundColor(color),
            Print(format!("[{}] ", prefix)),
            ResetColor,
            Print(format!("{}\n", msg))
        );
        let _ = stderr.flush();
    }

    pub fn info(msg: &str) {
        Self::log_with_color(Color::Cyan, "INFO", msg);
    }

    pub fn success(msg: &str) {
        Self::log_with_color(Color::Green, "SUCCESS", msg);
    }

    pub fn warning(msg: &str) {
        Self::log_with_color(Color::Yellow, "WARNING", msg);
    }

    pub fn error(msg: &str) {
        Self::log_error_with_color(Color::Red, "ERROR", msg);
    }

    pub fn scan(msg: &str) {
        Self::log_with_color(Color::Yellow, "SCAN", msg);
    }

    pub fn hook(msg: &str) {
        Self::log_with_color(Color::Magenta, "HOOK", msg);
    }

    pub fn bypass(msg: &str) {
        Self::log_with_color(Color::Green, "BYPASS", msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colored_output() {
        // this test just ensures the functions don't panic
        Logger::info("Test info message");
        Logger::success("Test success message");
        Logger::warning("Test warning message");
        Logger::error("Test error message");
        Logger::scan("Test scan message");
        Logger::hook("Test hook message");
        Logger::bypass("Test bypass message");
    }
}
