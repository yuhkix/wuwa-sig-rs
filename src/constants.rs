/// Application constants for configuration and behavior
pub mod constants {
    /// Target module name for scanning
    pub const TARGET_MODULE: &str = "Client-Win64-Shipping.exe";

    /// Byte pattern to search for in memory
    pub const TARGET_PATTERN: [u8; 7] = [0x49, 0x81, 0xC3, 0x9A, 0x0B, 0xFB, 0xFF];

    /// Pattern mask for the target pattern (x = exact match, ? = wildcard)
    pub const PATTERN_MASK: &str = "xxxxxxx";

    /// DLL process attach reason code
    pub const DLL_PROCESS_ATTACH: u32 = 1;

    /// Success return value for bypass function
    pub const BYPASS_SUCCESS: usize = 1;

    /// Memory access constants
    pub mod memory {
        /// Offset for v4 pointer in the pak file check structure
        pub const V4_POINTER_OFFSET: usize = 16;

        /// Offset for parent pointer in the structure
        pub const PARENT_POINTER_OFFSET: usize = 8;
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
