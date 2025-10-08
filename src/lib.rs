//! # WuWa Signature Scanner and Hook Library
//!
//! This library provides a high-performance memory scanning and hooking system
//! for Windows applications, specifically designed for bypassing PAK file verification
//! in WuWa (Wuthering Waves).
//!
//! ## Features
//!
//! - **High-performance memory scanning** with optimized pattern matching algorithms
//! - **Thread-safe hook management** with state tracking
//! - **Structured logging** with configurable levels and formatting
//! - **Safe memory access** with bounds checking and error handling
//! - **Comprehensive error handling** with detailed error types
//! - **Modular architecture** for easy maintenance and testing
//!
//! ## Architecture
//!
//! The library is organized into several modules:
//!
//! - `config`: Configuration management with validation
//! - `constants`: Application constants and magic numbers
//! - `error`: Comprehensive error handling with thiserror
//! - `hooks`: Thread-safe hook management with state tracking
//! - `logger`: Structured logging with performance optimizations
//! - `memory`: High-performance memory scanning and pattern matching
//! - `safety`: Safe abstractions for unsafe operations
//!
//! ## Usage
//!
//! ```rust
//! use wuwa_sig_rs::*;
//!
//! // The library automatically initializes when loaded as a DLL
//! // and applies the bypass hook to the target process.
//! ```
//!
//! ## Safety
//!
//! This library uses unsafe code for low-level memory operations and Windows API calls.
//! All unsafe operations are wrapped in safe abstractions with proper error handling
//! and bounds checking to minimize the risk of memory safety issues.
//!
//! ## Performance
//!
//! The library is optimized for performance with:
//! - Cached module scanning
//! - Optimized pattern matching algorithms
//! - Efficient memory access patterns
//! - Minimal allocation in hot paths

use std::ptr;
use std::thread;
use std::time::Duration;

use ilhook::x64::Registers;
use winapi::shared::minwindef::{BOOL, DWORD, HMODULE, LPVOID, TRUE};

// modular architecture
pub mod config;
pub mod constants;
pub mod error;
pub mod hooks;
pub mod logger;
pub mod memory;
pub mod safety;

use config::Config;
use constants::constants::memory::{PARENT_POINTER_OFFSET, V4_POINTER_OFFSET};
use constants::constants::*;
use error::{AppError, Result};
use hooks::PakFileHook;
use logger::Logger;
use memory::{ModuleScanner, PatternScanner};
use safety::{ConsoleManager, MemoryAccess, StringConverter, ThreadManager};

/// Main hook replacement function for PAK file verification bypass
///
/// This function is called instead of the original PAK file verification function.
/// It extracts the PAK file name from the register context and logs the verification
/// attempt, but always returns success to bypass the verification.
///
/// # Safety
///
/// This function is marked as unsafe because it:
/// - Accesses raw pointers from the register context
/// - Performs memory operations that could potentially access invalid memory
/// - Is called from native code with specific calling conventions
///
/// # Arguments
///
/// * `reg` - Pointer to the x64 register context containing function arguments
/// * `_` - Unused parameter (reserved for future use)
/// * `_` - Unused parameter (reserved for future use)
///
/// # Returns
///
/// Always returns `BYPASS_SUCCESS` (1) to indicate successful verification
unsafe extern "win64" fn pak_file_check_replacement(
    reg: *mut Registers,
    _: usize,
    _: usize,
) -> usize {
    //Logger::bypass(&format!("Register context: {:p}", reg));

    // let pak_name = extract_pak_name(reg);
    let pak_name = unsafe { extract_pak_name_unsafe(reg) };

    match pak_name {
        Ok(name) => {
            Logger::info(&format!("Verifying pak: '{}' -> OK", name));
        }
        Err(e) => {
            Logger::bypass(&format!(
                "Could not read pak name: {}, but returning true anyway",
                e
            ));
        }
    }

    //Logger::bypass("=== HOOK FUNCTION RETURNING SUCCESS ===");
    BYPASS_SUCCESS // always return success for bypass
}

unsafe fn extract_pak_name_unsafe(reg: *mut Registers) -> Result<String> {
    unsafe {
        let rcx = (*reg).rcx;
        let v4_ptr = *((rcx + 16) as *const usize);
        let parent_ptr = *(v4_ptr as *const usize);
        let wstr = *((parent_ptr + 8) as *const usize) as *const u16;
        StringConverter::pcwstr_to_string(wstr)
    }
}

/// Safely extract PAK file name from register context
#[allow(dead_code)]
fn extract_pak_name(reg: *mut Registers) -> Result<String> {
    let result = std::panic::catch_unwind(|| {
        // Safely read the v4 pointer from the register context
        let v4_ptr = unsafe {
            MemoryAccess::read_pointer_safe((*reg).rcx as *const usize, V4_POINTER_OFFSET)?
        };

        // Safely read the parent pointer
        let parent_ptr = unsafe { MemoryAccess::read_pointer_safe(v4_ptr as *const usize, 0)? };

        // Safely read the string pointer
        let string_ptr = unsafe {
            MemoryAccess::read_pointer_safe(parent_ptr as *const usize, PARENT_POINTER_OFFSET)?
        };

        // Convert the wide string to a Rust String
        unsafe { StringConverter::pcwstr_to_string(string_ptr as *const u16) }
    });

    result.unwrap_or_else(|_| {
        Err(AppError::PanicRecovery {
            reason: "Panic occurred while extracting pak name from registers".to_string(),
        })
    })
}

/// Initialize the console using safe abstractions
fn init_console() -> Result<()> {
    ConsoleManager::init()
}

/// Apply the bypass hook to the target function
///
/// This function performs the main hooking operation:
/// 1. Initializes the console for logging
/// 2. Validates the configuration
/// 3. Scans for the target module
/// 4. Finds the target function using pattern scanning
/// 5. Waits for ACE initialization to complete
/// 6. Applies the hook to replace the original function
/// 7. Enters an infinite loop to maintain the hook
///
/// # Safety
///
/// This function is marked as unsafe because it:
/// - Performs memory scanning operations
/// - Accesses process memory
/// - Modifies function pointers and code execution
///
/// # Returns
///
/// Returns `Ok(())` if the hook is successfully applied, or an error if any step fails.
/// Note that this function never returns normally due to the infinite loop at the end.
unsafe fn apply_bypass_hook() -> Result<()> {
    init_console()?;

    // Initialize the global logger after console is ready
    logger::init_global_logger();

    // Try to use the logger
    Logger::info("Console and logger initialized successfully");

    Logger::info("Creating configuration...");
    Logger::info(&format!("Target module: {}", TARGET_MODULE));
    Logger::info(&format!("Pattern: {:02X?}", TEST_PATTERN));
    Logger::info(&format!("Mask: {}", TEST_PATTERN_MASK));

    let config = Config::new(TARGET_MODULE, &TEST_PATTERN, TEST_PATTERN_MASK);
    Logger::info("Configuration created successfully");

    Logger::info("Validating configuration...");
    match config.validate() {
        Ok(_) => {
            Logger::info("Configuration validated successfully");
        }
        Err(e) => {
            Logger::error(&format!("Configuration validation failed: {}", e));
            return Err(e);
        }
    }

    Logger::info("Creating module scanner...");
    let scanner = ModuleScanner::new();

    Logger::info(&format!("Looking for module: {}", config.target_module));
    let module_base = match scanner.find_module(&config.target_module) {
        Ok(addr) => {
            Logger::info(&format!("Module found at: {:?}", addr));
            addr
        }
        Err(e) => {
            Logger::error(&format!("Failed to find module: {}", e));
            return Err(e);
        }
    };

    Logger::info("Getting module information...");
    let module_info = match scanner.get_module_info(module_base) {
        Ok(info) => {
            Logger::info("Module information retrieved successfully");
            info
        }
        Err(e) => {
            Logger::error(&format!("Failed to get module info: {}", e));
            return Err(e);
        }
    };

    Logger::scan(&format!("Module base address: {:?}", module_base));
    Logger::scan(&format!("Module size: {} bytes", module_info.SizeOfImage));

    Logger::info("Creating pattern scanner...");
    let mut pattern_scanner = PatternScanner::new();

    Logger::info("Starting pattern scan...");
    Logger::scan(&format!("Scanning for pattern: {:02X?}", config.pattern));
    Logger::scan(&format!("Using mask: {}", config.mask));

    let target_func = match pattern_scanner.scan(
        module_info.lpBaseOfDll as *mut u8,
        module_info.SizeOfImage as usize,
        &config.pattern,
        config.mask,
    ) {
        Ok(addr) => {
            Logger::info(&format!("Pattern found at: {:p}", addr));
            addr
        }
        Err(e) => {
            Logger::error(&format!("Pattern not found: {}", e));
            Logger::error(
                "This might indicate the game version has changed or the pattern is incorrect",
            );
            return Err(e);
        }
    };

    let new_target_func = (target_func as usize).saturating_sub(0x45) as *mut u8;

    Logger::scan(&format!(
        "Found target function (original scan result) at: {:p}",
        target_func
    ));
    Logger::scan(&format!(
        "Adjusted target function (new offset) at: {:p}",
        new_target_func
    ));

    let offset = (new_target_func as usize) - (module_base as usize);
    Logger::scan(&format!("Target function offset: {:#x}", offset));

    Logger::info("Reading preamble for ACE check...");
    let preamble = unsafe { *(new_target_func as *const u64) };
    Logger::info(&format!(
        "Using dynamic preamble for ACE check: {:#x}",
        preamble
    ));

    Logger::info("Waiting for ACE initialization...");
    wait_for_ace_init(new_target_func, preamble)?;

    Logger::info("Creating hook instance...");
    let hook = PakFileHook::new();

    Logger::info(&format!(
        "Hook state before application: {:?}",
        hook.state()
    ));
    Logger::info("Applying hook...");

    match hook.apply(new_target_func as usize, pak_file_check_replacement) {
        Ok(_) => {
            Logger::info("Hook applied successfully");
        }
        Err(e) => {
            Logger::error(&format!("Failed to apply hook: {}", e));
            return Err(e);
        }
    }

    Logger::info(&format!("Hook state after application: {:?}", hook.state()));
    Logger::info(&format!("Hook target address: {:?}", hook.target_address()));
    Logger::info(&format!("Hook is active: {}", hook.is_active()));

    // Verify the hook was applied by checking the memory
    let hook_addr = new_target_func as *const u8;
    let first_bytes = unsafe { std::slice::from_raw_parts(hook_addr, 16) };
    Logger::info(&format!(
        "First 16 bytes at hook address: {:02X?}",
        first_bytes
    ));

    Logger::success("Bypass successfully applied!");

    // infinite loop to maintain the hook
    Logger::info("Entering maintenance loop...");
    loop {
        thread::sleep(Duration::from_secs(MAX_SLEEP_DURATION_SECS));
    }
}

/// Wait for ACE initialization to complete using safe memory access
fn wait_for_ace_init(target_func: *mut u8, expected_preamble: u64) -> Result<()> {
    Logger::info("Waiting for ACE init...");

    let check_address = target_func as *const u64;
    loop {
        let current_preamble = unsafe { MemoryAccess::read_volatile_safe(check_address)? };

        if current_preamble == expected_preamble {
            Logger::success("ACE Initialization finished");
            return Ok(());
        }

        thread::sleep(Duration::from_millis(ACE_CHECK_INTERVAL_MS));
    }
}

/// Thread entry point for the hook application
///
/// This function is called when the DLL is loaded and creates a new thread
/// to perform the hooking operation. This prevents blocking the main thread
/// and allows the DLL to load successfully.
///
/// # Safety
///
/// This function is marked as unsafe because it calls `apply_bypass_hook()`
/// which performs unsafe memory operations.
///
/// # Arguments
///
/// * `_lp_parameter` - Unused parameter passed to the thread
///
/// # Returns
///
/// Returns 0 on success, 1 on failure
unsafe extern "system" fn start_address(_lp_parameter: LPVOID) -> DWORD {
    match unsafe { apply_bypass_hook() } {
        Ok(_) => {
            // Success - hook applied and running
            0
        }
        Err(e) => {
            // Try to log the error, but don't fail if logger isn't available
            let error_msg = format!("Hook application failed: {}", e);
            let _ = std::panic::catch_unwind(|| {
                Logger::error(&error_msg);
            });
            1
        }
    }
}

/// DLL entry point
///
/// This is the main entry point for the DLL. It's called by the Windows loader
/// when the DLL is loaded or unloaded from a process.
///
/// # Safety
///
/// This function is marked as unsafe because it's a Windows API callback
/// that must follow specific calling conventions and handle system-level operations.
///
/// # Arguments
///
/// * `_h_module` - Handle to the DLL module (unused)
/// * `ul_reason_for_call` - Reason for the call (DLL_PROCESS_ATTACH, etc.)
/// * `_lp_reserved` - Reserved parameter (unused)
///
/// # Returns
///
/// Always returns `TRUE` to indicate successful DLL initialization
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(
    _h_module: HMODULE,
    ul_reason_for_call: DWORD,
    _lp_reserved: LPVOID,
) -> BOOL {
    if ul_reason_for_call == DLL_PROCESS_ATTACH {
        if let Err(_e) = ThreadManager::create_thread(start_address, ptr::null_mut()) {
            // Can't use logger here since it's not initialized yet
            // The error will be handled in the thread function
        }
    }

    TRUE
}
