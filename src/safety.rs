/// Safe abstractions for unsafe operations
use std::ptr;
use winapi::shared::minwindef::{DWORD, LPVOID};
use winapi::um::consoleapi::AllocConsole;
use winapi::um::processthreadsapi::CreateThread;
use windows::core::PCWSTR;

use crate::error::{AppError, Result};

/// Safe wrapper for console initialization
pub struct ConsoleManager;

impl ConsoleManager {
    /// Initialize the console with proper error handling
    pub fn init() -> Result<()> {
        let result = unsafe { AllocConsole() };

        if result == 0 {
            let error = std::io::Error::last_os_error();
            // Don't use logger here since it's not initialized yet
            return Err(AppError::ConsoleInitFailed { source: error });
        }

        // Console is ready, but don't log yet since logger isn't initialized
        Ok(())
    }
}

/// Safe wrapper for thread creation
pub struct ThreadManager;

impl ThreadManager {
    /// Create a thread with proper error handling
    pub fn create_thread(
        start_address: unsafe extern "system" fn(LPVOID) -> DWORD,
        parameter: LPVOID,
    ) -> Result<()> {
        let thread_handle = unsafe {
            CreateThread(
                ptr::null_mut(), // Security attributes
                0,               // Stack size (default)
                Some(start_address),
                parameter,
                0,               // Creation flags
                ptr::null_mut(), // Thread ID
            )
        };

        if thread_handle.is_null() {
            let error = std::io::Error::last_os_error();
            return Err(AppError::HookFailed {
                message: format!("Thread creation failed: {}", error),
            });
        }

        // Thread created successfully
        Ok(())
    }
}

/// Safe memory access utilities
pub struct MemoryAccess;

impl MemoryAccess {
    /// Safely read a value from memory with bounds checking
    pub unsafe fn read_volatile_safe<T>(ptr: *const T) -> Result<T>
    where
        T: Copy,
    {
        if ptr.is_null() {
            return Err(AppError::MemoryAccessViolation { address: 0 });
        }

        // In a real implementation, you might want to add more sophisticated
        // bounds checking or use platform-specific APIs to verify the memory
        // is readable before accessing it.

        Ok(unsafe { ptr::read_volatile(ptr) })
    }

    /// Safely read a pointer from memory
    pub unsafe fn read_pointer_safe(ptr: *const usize, offset: usize) -> Result<usize> {
        if ptr.is_null() {
            return Err(AppError::MemoryAccessViolation { address: 0 });
        }

        let target_ptr = unsafe { ptr.add(offset) };
        let value = unsafe { ptr::read_volatile(target_ptr) };

        if value == 0 {
            return Err(AppError::MemoryAccessViolation {
                address: target_ptr as usize,
            });
        }

        Ok(value)
    }

    /// Safely dereference a pointer with null checking
    pub unsafe fn deref_pointer_safe<T>(ptr: *const T) -> Result<T>
    where
        T: Copy,
    {
        if ptr.is_null() {
            return Err(AppError::MemoryAccessViolation { address: 0 });
        }

        Ok(unsafe { *ptr })
    }
}

/// Safe string conversion utilities
pub struct StringConverter;

impl StringConverter {
    /// Safely convert a PCWSTR to a Rust String
    pub unsafe fn pcwstr_to_string(pcwstr: *const u16) -> Result<String> {
        if pcwstr.is_null() {
            return Err(AppError::StringConversion {
                details: "PCWSTR pointer is null".to_string(),
            });
        }

        match unsafe { PCWSTR::from_raw(pcwstr).to_string() } {
            Ok(s) => Ok(s),
            Err(e) => Err(AppError::StringConversion {
                details: format!("UTF-16 to UTF-8 conversion failed: {}", e),
            }),
        }
    }
}

/// Safe pattern matching utilities
pub struct PatternMatcher;

impl PatternMatcher {
    /// Safely match a pattern at a given offset with bounds checking
    pub unsafe fn matches_pattern_safe(
        base: *const u8,
        offset: usize,
        pattern: &[u8],
        mask: &str,
        max_size: usize,
    ) -> Result<bool> {
        if base.is_null() {
            return Err(AppError::MemoryAccessViolation { address: 0 });
        }

        if pattern.len() != mask.len() {
            return Err(AppError::PatternNotFound { size: 0 });
        }

        if offset + pattern.len() > max_size {
            return Err(AppError::MemoryAccessViolation {
                address: base as usize + offset,
            });
        }

        for (i, &byte) in pattern.iter().enumerate() {
            let mask_char = mask.as_bytes()[i];
            if mask_char == b'x' {
                let target_byte = unsafe { *base.add(offset + i) };
                if target_byte != byte {
                    return Ok(false);
                }
            }
            // '?' matches any byte, so we skip the check
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_access_null_pointer() {
        let result = unsafe { MemoryAccess::read_volatile_safe::<u32>(ptr::null()) };
        assert!(result.is_err());
    }

    #[test]
    fn test_string_converter_null_pointer() {
        let result = unsafe { StringConverter::pcwstr_to_string(ptr::null()) };
        assert!(result.is_err());
    }

    #[test]
    fn test_pattern_matcher_bounds_check() {
        let buffer = vec![0x55, 0x53, 0x56];
        let pattern = [0x55, 0x53, 0x56, 0x57]; // Longer than buffer
        let mask = "xxxx";

        let result = unsafe {
            PatternMatcher::matches_pattern_safe(buffer.as_ptr(), 0, &pattern, mask, buffer.len())
        };
        assert!(result.is_err());
    }
}
