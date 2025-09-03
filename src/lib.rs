use std::ptr;
use std::thread;
use std::time::Duration;

use ilhook::x64::Registers;
use windows::core::PCWSTR;

use winapi::shared::minwindef::{BOOL, DWORD, HMODULE, LPVOID, TRUE};
use winapi::um::consoleapi::AllocConsole;
use winapi::um::processthreadsapi::CreateThread;

// modular architecture
pub mod config;
pub mod error;
pub mod hooks;
pub mod logger;
pub mod memory;

use config::Config;
use error::{AppError, Result};
use hooks::PakFileHook;
use logger::Logger;
use memory::{ModuleScanner, PatternScanner};

// configuration
const TARGET_MODULE: &str = "Client-Win64-Shipping.exe";
const TARGET_PATTERN: [u8; 14] = [
    0x55, 0x53, 0x56, 0x41, 0x54, 0x41, 0x57, 0x48, 0x89, 0xE5, 0x48, 0x83, 0xEC, 0x60,
];
const PATTERN_MASK: &str = "xxxxxxxxxxxxxx";
const POLL_INTERVAL_MS: u64 = 1;

// main hook logic
unsafe extern "win64" fn pak_file_check_replacement(
    reg: *mut Registers,
    _: usize,
    _: usize,
) -> usize {
    let pak_name = extract_pak_name(reg);

    match pak_name {
        Ok(name) => {
            Logger::bypass(&format!("Verifying pak: '{}' -> OK", name));
        }
        Err(_) => {
            Logger::bypass("Could not read pak name, but returning true anyway");
        }
    }

    1 // always return success for bypass
}

fn extract_pak_name(reg: *mut Registers) -> Result<String> {
    let result = std::panic::catch_unwind(|| {
        let v4_ptr = unsafe { *(((*reg).rcx + 16) as *const usize) };
        let parent_ptr = unsafe { *(v4_ptr as *const usize) };

        unsafe {
            PCWSTR::from_raw(*((parent_ptr + 8) as *const usize) as *const u16)
                .to_string()
                .map_err(|_| AppError::StringConversion)
        }
    });

    result.unwrap_or(Err(AppError::PanicRecovery))
}

// console initialization
fn init_console() -> Result<()> {
    unsafe {
        AllocConsole();
    }
    Logger::info("Console initialized");
    Ok(())
}

// main hook function
unsafe fn apply_bypass_hook() -> Result<()> {
    init_console()?;

    let config = Config::new(TARGET_MODULE, &TARGET_PATTERN, PATTERN_MASK);
    if !config.validate() {
        return Err(AppError::ModuleNotFound(
            "Invalid configuration".to_string(),
        ));
    }

    let scanner = ModuleScanner::new();

    let module_base = scanner.find_module(&config.target_module)?;
    let module_info = scanner.get_module_info(module_base)?;

    Logger::scan(&format!("Module base address: {:?}", module_base));

    let mut pattern_scanner = PatternScanner::new();
    let target_func = pattern_scanner.scan(
        module_info.lpBaseOfDll as *mut u8,
        module_info.SizeOfImage as usize,
        &config.pattern,
        config.mask,
    )?;

    Logger::scan(&format!("Found target function at: {:p}", target_func));

    let offset = (target_func as usize) - (module_base as usize);
    Logger::scan(&format!("Target function offset: {:#x}", offset));

    let preamble = unsafe { *(target_func as *const u64) };
    Logger::info(&format!(
        "Using dynamic preamble for ACE check: {:#x}",
        preamble
    ));

    wait_for_ace_init(target_func, preamble)?;

    let mut hook = PakFileHook::new();
    hook.apply(target_func as usize, pak_file_check_replacement)?;

    Logger::success("Bypass successfully applied!");

    // infinite loop to maintain the hook
    loop {
        thread::sleep(Duration::from_secs(u64::MAX));
    }
}

fn wait_for_ace_init(target_func: *mut u8, expected_preamble: u64) -> Result<()> {
    Logger::info("Waiting for ACE init...");

    let check_address = target_func as *const u64;
    loop {
        let current_preamble = unsafe { ptr::read_volatile(check_address) };
        if current_preamble == expected_preamble {
            Logger::success("ACE Initialization finished");
            return Ok(());
        }
        thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
    }
}

// thread entry point
unsafe extern "system" fn start_address(_lp_parameter: LPVOID) -> DWORD {
    match unsafe { apply_bypass_hook() } {
        Ok(_) => 0,
        Err(e) => {
            Logger::error(&format!("Hook application failed: {}", e));
            1
        }
    }
}

// dll entry point
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(
    _h_module: HMODULE,
    ul_reason_for_call: DWORD,
    _lp_reserved: LPVOID,
) -> BOOL {
    const DLL_PROCESS_ATTACH: DWORD = 1;

    if ul_reason_for_call == DLL_PROCESS_ATTACH {
        unsafe {
            CreateThread(
                ptr::null_mut(),
                0,
                Some(start_address),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
            );
        }
    }

    TRUE
}
