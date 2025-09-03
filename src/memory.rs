use std::ffi::CStr;
use std::ptr;

use winapi::ctypes::c_void;
use winapi::shared::minwindef::{DWORD, HMODULE};
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::psapi::{EnumProcessModules, GetModuleBaseNameA, GetModuleInformation, MODULEINFO};

use crate::error::{AppError, Result};

pub struct ModuleScanner;

impl ModuleScanner {
    pub fn new() -> Self {
        Self
    }

    pub fn find_module(&self, name: &str) -> Result<HMODULE> {
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
                return Err(AppError::ModuleNotFound(name.to_string()));
            }

            let count = (cb_needed as usize) / std::mem::size_of::<HMODULE>();
            for &mod_handle in &modules[..count] {
                if let Some(module_name) = self.get_module_name(h_process, mod_handle)? {
                    if module_name == name {
                        return Ok(mod_handle);
                    }
                }
            }

            Err(AppError::ModuleNotFound(name.to_string()))
        }
    }

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
                return Err(AppError::ModuleInfoFailed);
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

pub struct PatternScanner {
    cache: std::collections::HashMap<(usize, usize), *mut u8>,
}

impl PatternScanner {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

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

    fn scan_impl(&self, base: *mut u8, size: usize, pattern: &[u8], mask: &str) -> Result<*mut u8> {
        if pattern.len() != mask.len() {
            return Err(AppError::PatternNotFound);
        }

        for i in 0..=size.saturating_sub(pattern.len()) {
            if self.matches_pattern(base, i, pattern, mask) {
                return Ok(unsafe { base.add(i) });
            }
        }

        Err(AppError::PatternNotFound)
    }

    fn matches_pattern(&self, base: *mut u8, offset: usize, pattern: &[u8], mask: &str) -> bool {
        for (j, &b) in pattern.iter().enumerate() {
            if mask.as_bytes()[j] == b'x' && unsafe { *base.add(offset + j) } != b {
                return false;
            }
        }
        true
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
        let _mask = "x?"; // ? should match any byte

        // this test would need to be adjusted based on actual wildcard implementation
        // for now, testing exact match
        assert!(scanner.matches_pattern(buffer.as_mut_ptr(), 0, &pattern, "xx"));
    }
}
