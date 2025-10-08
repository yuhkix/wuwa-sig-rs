use crate::error::{AppError, Result};

/// Configuration for the memory scanner and hook system
#[derive(Debug, Clone)]
pub struct Config<'a> {
    /// Target module name to scan
    pub target_module: &'a str,
    /// Byte pattern to search for
    pub pattern: &'a [u8],
    /// Mask string where 'x' means exact match, '?' means wildcard
    pub mask: &'a str,
    /// Maximum scan size in bytes (default: 100MB)
    pub max_scan_size: usize,
    /// Timeout for ACE initialization in milliseconds (default: 5000ms)
    pub ace_init_timeout_ms: u64,
}

impl<'a> Config<'a> {
    /// Create a new configuration with default values
    pub fn new(target_module: &'a str, pattern: &'a [u8], mask: &'a str) -> Self {
        Self {
            target_module,
            pattern,
            mask,
            max_scan_size: 100 * 1024 * 1024, // 100MB default
            ace_init_timeout_ms: 5000,        // 5 seconds default
        }
    }

    /// Create a new configuration with custom scan size and timeout
    pub fn with_limits(
        target_module: &'a str,
        pattern: &'a [u8],
        mask: &'a str,
        max_scan_size: usize,
        ace_init_timeout_ms: u64,
    ) -> Self {
        Self {
            target_module,
            pattern,
            mask,
            max_scan_size,
            ace_init_timeout_ms,
        }
    }

    /// Validate the configuration and return detailed error information
    pub fn validate(&self) -> Result<()> {
        if self.target_module.is_empty() {
            return Err(AppError::InvalidConfig {
                field: "target_module".to_string(),
                reason: "Module name cannot be empty".to_string(),
            });
        }

        if self.pattern.is_empty() {
            return Err(AppError::InvalidConfig {
                field: "pattern".to_string(),
                reason: "Pattern cannot be empty".to_string(),
            });
        }

        if self.mask.is_empty() {
            return Err(AppError::InvalidConfig {
                field: "mask".to_string(),
                reason: "Mask cannot be empty".to_string(),
            });
        }

        if self.pattern.len() != self.mask.len() {
            return Err(AppError::InvalidConfig {
                field: "pattern/mask".to_string(),
                reason: format!(
                    "Pattern length ({}) must match mask length ({})",
                    self.pattern.len(),
                    self.mask.len()
                ),
            });
        }

        // Validate mask characters
        for (i, ch) in self.mask.chars().enumerate() {
            if ch != 'x' && ch != '?' {
                return Err(AppError::InvalidConfig {
                    field: "mask".to_string(),
                    reason: format!(
                        "Invalid mask character '{}' at position {} (only 'x' and '?' allowed)",
                        ch, i
                    ),
                });
            }
        }

        if self.max_scan_size == 0 {
            return Err(AppError::InvalidConfig {
                field: "max_scan_size".to_string(),
                reason: "Maximum scan size must be greater than 0".to_string(),
            });
        }

        if self.ace_init_timeout_ms == 0 {
            return Err(AppError::InvalidConfig {
                field: "ace_init_timeout_ms".to_string(),
                reason: "ACE initialization timeout must be greater than 0".to_string(),
            });
        }

        Ok(())
    }

    /// Get the pattern length
    pub fn pattern_len(&self) -> usize {
        self.pattern.len()
    }

    /// Check if the pattern uses wildcards
    pub fn has_wildcards(&self) -> bool {
        self.mask.contains('?')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation_valid() {
        let config = Config::new("test.exe", &[0x55, 0x53], "xx");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_empty_module() {
        let config = Config::new("", &[0x55, 0x53], "xx");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_mismatched_lengths() {
        let config = Config::new("test.exe", &[0x55, 0x53], "x");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_empty_pattern() {
        let config = Config::new("test.exe", &[], "xx");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_mask_character() {
        let config = Config::new("test.exe", &[0x55, 0x53], "xz");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_with_limits() {
        let config = Config::with_limits("test.exe", &[0x55, 0x53], "xx", 1024, 1000);
        assert!(config.validate().is_ok());
        assert_eq!(config.max_scan_size, 1024);
        assert_eq!(config.ace_init_timeout_ms, 1000);
    }

    #[test]
    fn test_config_has_wildcards() {
        let config_with_wildcards = Config::new("test.exe", &[0x55, 0x53], "x?");
        let config_without_wildcards = Config::new("test.exe", &[0x55, 0x53], "xx");

        assert!(config_with_wildcards.has_wildcards());
        assert!(!config_without_wildcards.has_wildcards());
    }

    #[test]
    fn test_config_pattern_len() {
        let config = Config::new("test.exe", &[0x55, 0x53, 0x56], "xxx");
        assert_eq!(config.pattern_len(), 3);
    }
}
