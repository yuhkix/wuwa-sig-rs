/// Application constants for configuration and behavior
pub mod constants {
    /// Target module name for scanning
    pub const TARGET_MODULE: &str = "Client-Win64-Shipping.exe";

    /// Byte pattern to search for in memory
    pub const TARGET_PATTERN: [u8; 14] = [
        0x55, 0x53, 0x56, 0x41, 0x54, 0x41, 0x57, 0x48, 0x89, 0xE5, 0x48, 0x83, 0xEC, 0x60,
    ];

    pub const TEST_PATTERN: [u8; 7] = [
        0x49, 0x81, 0xC3, 0x9A, 0x0B, 0xFB, 0xFF, // substract 0x45 to get offset
    ];

    /// Pattern mask for the target pattern (x = exact match, ? = wildcard)
    pub const PATTERN_MASK: &str = "xxxxxxxxxxxxxx";
    pub const TEST_PATTERN_MASK: &str = "xxxxxxx";

    /// Maximum number of modules to scan (buffer size)
    pub const MAX_MODULES: usize = 1024;

    /// Maximum module name length
    pub const MAX_MODULE_NAME_LEN: usize = 256;

    /// Default maximum scan size in bytes (100MB)
    pub const DEFAULT_MAX_SCAN_SIZE: usize = 100 * 1024 * 1024;

    /// Default ACE initialization timeout in milliseconds (5 seconds)
    pub const DEFAULT_ACE_INIT_TIMEOUT_MS: u64 = 5000;

    /// Sleep interval for ACE initialization check in milliseconds
    pub const ACE_CHECK_INTERVAL_MS: u64 = 1;

    /// Maximum sleep duration for the main loop (effectively infinite)
    pub const MAX_SLEEP_DURATION_SECS: u64 = u64::MAX;

    /// DLL process attach reason code
    pub const DLL_PROCESS_ATTACH: u32 = 1;

    /// Success return value for bypass function
    pub const BYPASS_SUCCESS: usize = 1;

    /// Thread creation flags
    pub const THREAD_CREATE_FLAGS: u32 = 0;

    /// Memory access constants
    pub mod memory {
        /// Offset for v4 pointer in the pak file check structure
        pub const V4_POINTER_OFFSET: usize = 16;

        /// Offset for parent pointer in the structure
        pub const PARENT_POINTER_OFFSET: usize = 8;

        /// Size of a pointer on x64 systems
        pub const POINTER_SIZE: usize = 8;
    }

    /// Logging constants
    pub mod logging {
        /// Default log level for the application
        pub const DEFAULT_LOG_LEVEL: &str = "BYPASS";

        /// Maximum log message length
        pub const MAX_LOG_MESSAGE_LEN: usize = 1024;
    }

    /// Error handling constants
    pub mod error {
        /// Maximum error message length
        pub const MAX_ERROR_MESSAGE_LEN: usize = 512;

        /// Maximum number of error context items
        pub const MAX_ERROR_CONTEXT_ITEMS: usize = 10;
    }
}

/// Re-export commonly used constants
pub use constants::*;
