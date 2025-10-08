use std::collections::HashMap;
use std::ffi::CStr;
use std::ptr;
use std::sync::{Arc, RwLock};

use winapi::ctypes::c_void;
use winapi::shared::minwindef::{DWORD, HMODULE};
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::psapi::{EnumProcessModules, GetModuleBaseNameA, GetModuleInformation, MODULEINFO};

use crate::error::{AppError, Result};

/// Module scanner with caching for improved performance
pub struct ModuleScanner {
    module_cache: Arc<RwLock<HashMap<String, HMODULE>>>,
}

impl ModuleScanner {
    pub fn new() -> Self {
        Self {
            module_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Find a module by name with caching
    pub fn find_module(&self, name: &str) -> Result<HMODULE> {
        // Check cache first
        {
            let cache = self.module_cache.read().unwrap();
            if let Some(&module) = cache.get(name) {
                return Ok(module);
            }
        }

        // Module not in cache, search for it
        let module = self.find_module_impl(name)?;

        // Cache the result
        {
            let mut cache = self.module_cache.write().unwrap();
            cache.insert(name.to_string(), module);
        }

        Ok(module)
    }

    /// Internal implementation of module finding
    fn find_module_impl(&self, name: &str) -> Result<HMODULE> {
        unsafe {
            let h_process = GetCurrentProcess();
            let mut modules: [HMODULE; 1024] = [ptr::null_mut(); 1024];
            let mut cb_needed: DWORD = 0;

            if EnumProcessModules(
                h_process,
                modules.as_mut_ptr(),
                std::mem::size_of_val(&modules) as DWORD,
                &mut cb_needed,
            ) == 0
            {
                return Err(AppError::ModuleNotFound {
                    name: name.to_string(),
                });
            }

            let count = (cb_needed as usize) / std::mem::size_of::<HMODULE>();
            for &mod_handle in &modules[..count] {
                if let Some(module_name) = self.get_module_name(h_process, mod_handle)? {
                    if module_name.eq_ignore_ascii_case(name) {
                        return Ok(mod_handle);
                    }
                }
            }

            Err(AppError::ModuleNotFound {
                name: name.to_string(),
            })
        }
    }

    /// Get module information with better error handling
    pub fn get_module_info(&self, module_base: HMODULE) -> Result<MODULEINFO> {
        unsafe {
            let mut mod_info = MODULEINFO {
                lpBaseOfDll: ptr::null_mut(),
                SizeOfImage: 0,
                EntryPoint: ptr::null_mut(),
            };

            if GetModuleInformation(
                GetCurrentProcess(),
                module_base,
                &mut mod_info,
                std::mem::size_of::<MODULEINFO>() as DWORD,
            ) == 0
            {
                return Err(AppError::ModuleInfoFailed {
                    source: std::io::Error::last_os_error(),
                });
            }

            Ok(mod_info)
        }
    }

    fn get_module_name(
        &self,
        h_process: *mut c_void,
        mod_handle: HMODULE,
    ) -> Result<Option<String>> {
        unsafe {
            let mut mod_name = [0u8; 256];
            if GetModuleBaseNameA(
                h_process,
                mod_handle,
                mod_name.as_mut_ptr() as *mut i8,
                256 as DWORD,
            ) == 0
            {
                return Ok(None);
            }

            if let Ok(s) = CStr::from_ptr(mod_name.as_ptr() as *const i8).to_str() {
                Ok(Some(s.to_string()))
            } else {
                Ok(None)
            }
        }
    }
}

/// High-performance pattern scanner with optimized algorithms
pub struct PatternScanner {
    cache: HashMap<(usize, usize), *mut u8>,
}

impl PatternScanner {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Scan for a pattern with caching and optimized algorithms
    pub fn scan(
        &mut self,
        base: *mut u8,
        size: usize,
        pattern: &[u8],
        mask: &str,
    ) -> Result<*mut u8> {
        let cache_key = (base as usize, size);

        if let Some(&cached_result) = self.cache.get(&cache_key) {
            if !cached_result.is_null() {
                return Ok(cached_result);
            }
        }

        let result = self.scan_impl(base, size, pattern, mask)?;
        self.cache.insert(cache_key, result);
        Ok(result)
    }

    /// Optimized pattern scanning implementation
    fn scan_impl(&self, base: *mut u8, size: usize, pattern: &[u8], mask: &str) -> Result<*mut u8> {
        if pattern.len() != mask.len() {
            return Err(AppError::PatternNotFound { size });
        }

        if pattern.is_empty() {
            return Err(AppError::PatternNotFound { size });
        }

        // Use Boyer-Moore-like optimization for exact patterns
        if !mask.contains('?') {
            return self.scan_exact_pattern(base, size, pattern);
        }

        // Fall back to brute force for patterns with wildcards
        self.scan_with_wildcards(base, size, pattern, mask)
    }

    /// Optimized scanning for exact patterns (no wildcards)
    fn scan_exact_pattern(&self, base: *mut u8, size: usize, pattern: &[u8]) -> Result<*mut u8> {
        if pattern.len() > size {
            return Err(AppError::PatternNotFound { size });
        }

        // Use memchr for single-byte patterns
        if pattern.len() == 1 {
            return self.scan_single_byte(base, size, pattern[0]);
        }

        // Use optimized multi-byte scanning
        self.scan_multi_byte_optimized(base, size, pattern)
    }

    /// Scan for a single byte pattern using memchr
    fn scan_single_byte(&self, base: *mut u8, size: usize, byte: u8) -> Result<*mut u8> {
        unsafe {
            let slice = std::slice::from_raw_parts(base, size);
            if let Some(pos) = slice.iter().position(|&b| b == byte) {
                Ok(base.add(pos))
            } else {
                Err(AppError::PatternNotFound { size })
            }
        }
    }

    /// Optimized multi-byte pattern scanning
    fn scan_multi_byte_optimized(
        &self,
        base: *mut u8,
        size: usize,
        pattern: &[u8],
    ) -> Result<*mut u8> {
        unsafe {
            let slice = std::slice::from_raw_parts(base, size);
            let pattern_len = pattern.len();

            // Use sliding window approach
            for i in 0..=size.saturating_sub(pattern_len) {
                if &slice[i..i + pattern_len] == pattern {
                    return Ok(base.add(i));
                }
            }

            Err(AppError::PatternNotFound { size })
        }
    }

    /// Scan patterns with wildcards using brute force
    fn scan_with_wildcards(
        &self,
        base: *mut u8,
        size: usize,
        pattern: &[u8],
        mask: &str,
    ) -> Result<*mut u8> {
        for i in 0..=size.saturating_sub(pattern.len()) {
            if self.matches_pattern(base, i, pattern, mask) {
                return Ok(unsafe { base.add(i) });
            }
        }

        Err(AppError::PatternNotFound { size })
    }

    /// Check if pattern matches at given offset
    fn matches_pattern(&self, base: *mut u8, offset: usize, pattern: &[u8], mask: &str) -> bool {
        unsafe {
            for (j, &b) in pattern.iter().enumerate() {
                let mask_char = mask.as_bytes()[j];
                if mask_char == b'x' && *base.add(offset + j) != b {
                    return false;
                }
                // '?' matches any byte, so we don't need to check it
            }
            true
        }
    }

    /// Clear the pattern cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        let total_entries = self.cache.len();
        let non_null_entries = self.cache.values().filter(|&&ptr| !ptr.is_null()).count();
        (total_entries, non_null_entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_scanner_matches() {
        let scanner = PatternScanner::new();
        let mut buffer = vec![0x55, 0x53, 0x56, 0x41, 0x54];
        let pattern = [0x55, 0x53];
        let mask = "xx";

        assert!(scanner.matches_pattern(buffer.as_mut_ptr(), 0, &pattern, mask));
    }

    #[test]
    fn test_pattern_scanner_no_match() {
        let scanner = PatternScanner::new();
        let mut buffer = vec![0x55, 0x54, 0x56, 0x41, 0x54];
        let pattern = [0x55, 0x53];
        let mask = "xx";

        assert!(!scanner.matches_pattern(buffer.as_mut_ptr(), 0, &pattern, mask));
    }

    #[test]
    fn test_pattern_scanner_with_wildcard() {
        let scanner = PatternScanner::new();
        let mut buffer = vec![0x55, 0x00, 0x56, 0x41, 0x54];
        let pattern = [0x55, 0x00];
        let mask = "x?"; // ? should match any byte

        assert!(scanner.matches_pattern(buffer.as_mut_ptr(), 0, &pattern, mask));
    }

    #[test]
    fn test_single_byte_pattern() {
        let mut scanner = PatternScanner::new();
        let mut buffer = vec![0x55, 0x53, 0x56, 0x41, 0x54];
        let pattern = [0x53];
        let mask = "x";

        let result = scanner.scan(buffer.as_mut_ptr(), buffer.len(), &pattern, mask);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), unsafe { buffer.as_mut_ptr().add(1) });
    }

    #[test]
    fn test_multi_byte_pattern() {
        let mut scanner = PatternScanner::new();
        let mut buffer = vec![0x55, 0x53, 0x56, 0x41, 0x54];
        let pattern = [0x55, 0x53];
        let mask = "xx";

        let result = scanner.scan(buffer.as_mut_ptr(), buffer.len(), &pattern, mask);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), buffer.as_mut_ptr());
    }

    #[test]
    fn test_pattern_not_found() {
        let mut scanner = PatternScanner::new();
        let mut buffer = vec![0x55, 0x53, 0x56, 0x41, 0x54];
        let pattern = [0xFF, 0xFF];
        let mask = "xx";

        let result = scanner.scan(buffer.as_mut_ptr(), buffer.len(), &pattern, mask);
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_functionality() {
        let mut scanner = PatternScanner::new();
        let mut buffer = vec![0x55, 0x53, 0x56, 0x41, 0x54];
        let pattern = [0x55, 0x53];
        let mask = "xx";

        // First scan
        let result1 = scanner.scan(buffer.as_mut_ptr(), buffer.len(), &pattern, mask);
        assert!(result1.is_ok());

        // Second scan should use cache
        let result2 = scanner.scan(buffer.as_mut_ptr(), buffer.len(), &pattern, mask);
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());

        // Check cache stats
        let (total, non_null) = scanner.cache_stats();
        assert!(total > 0);
        assert!(non_null > 0);
    }

    #[test]
    fn test_clear_cache() {
        let mut scanner = PatternScanner::new();
        let mut buffer = vec![0x55, 0x53, 0x56, 0x41, 0x54];
        let pattern = [0x55, 0x53];
        let mask = "xx";

        // Add something to cache
        let _ = scanner.scan(buffer.as_mut_ptr(), buffer.len(), &pattern, mask);
        assert!(scanner.cache_stats().0 > 0);

        // Clear cache
        scanner.clear_cache();
        assert_eq!(scanner.cache_stats().0, 0);
    }
}
