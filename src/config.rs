pub struct Config<'a> {
    pub target_module: &'a str,
    pub pattern: &'a [u8],
    pub mask: &'a str,
}

impl<'a> Config<'a> {
    pub fn new(target_module: &'a str, pattern: &'a [u8], mask: &'a str) -> Self {
        Self {
            target_module,
            pattern,
            mask,
        }
    }

    pub fn validate(&self) -> bool {
        !self.target_module.is_empty()
            && !self.pattern.is_empty()
            && !self.mask.is_empty()
            && self.pattern.len() == self.mask.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation_valid() {
        let config = Config::new("test.exe", &[0x55, 0x53], "xx");
        assert!(config.validate());
    }

    #[test]
    fn test_config_validation_empty_module() {
        let config = Config::new("", &[0x55, 0x53], "xx");
        assert!(!config.validate());
    }

    #[test]
    fn test_config_validation_mismatched_lengths() {
        let config = Config::new("test.exe", &[0x55, 0x53], "x");
        assert!(!config.validate());
    }

    #[test]
    fn test_config_validation_empty_pattern() {
        let config = Config::new("test.exe", &[], "xx");
        assert!(!config.validate());
    }
}
