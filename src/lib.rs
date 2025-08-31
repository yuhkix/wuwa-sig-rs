use std::ffi::CStr;
use std::ptr;
use std::thread;
use std::time::Duration;

use ilhook::x64::Registers;
use interceptor_rs::Interceptor;
use windows::core::PCWSTR;

use winapi::shared::minwindef::{BOOL, DWORD, HMODULE, LPVOID, TRUE};
use winapi::um::consoleapi::AllocConsole;
use winapi::um::processthreadsapi::{CreateThread, GetCurrentProcess};
use winapi::um::psapi::{EnumProcessModules, GetModuleBaseNameA, GetModuleInformation, MODULEINFO};

// ANSI Color Codes for beautified console output
const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const RED: &str = "\x1b[31m";

unsafe extern "win64" fn f_pak_file_check_replacement(
    reg: *mut Registers,
    _: usize,
    _: usize,
) -> usize {
    let pak_name_result = std::panic::catch_unwind(|| {
        let v4_ptr = unsafe { *(((*reg).rcx + 16) as *const usize) };
        let parent_ptr = unsafe { *(v4_ptr as *const usize) };
        unsafe {
            PCWSTR::from_raw(*((parent_ptr + 8) as *const usize) as *const u16)
                .to_string()
                .unwrap_or_else(|_| "Invalid String".to_string())
        }
    });

    if let Ok(pak_name) = pak_name_result {
        println!(
            "{GREEN}[BYPASS]{RESET} Verifying pak: '{}'. -> {GREEN}OK{RESET}",
            pak_name
        );
    } else {
        println!("{GREEN}[BYPASS]{RESET} Could not read pak name, but returning true anyway.");
    }

    1
}

fn init_console() {
    unsafe {
        AllocConsole();
        println!("{CYAN}[INFO]{RESET} Console initialized.");
    }
}

fn get_module_by_name(name: &str) -> Option<HMODULE> {
    unsafe {
        let h_process = GetCurrentProcess();
        let mut modules: [HMODULE; 1024] = [ptr::null_mut(); 1024];
        let mut cb_needed: DWORD = 0;

        if EnumProcessModules(
            h_process,
            modules.as_mut_ptr(),
            std::mem::size_of_val(&modules) as DWORD,
            &mut cb_needed,
        ) != 0
        {
            let count = (cb_needed as usize) / std::mem::size_of::<HMODULE>();
            for &mod_handle in &modules[..count] {
                let mut mod_name = [0u8; 256];
                if GetModuleBaseNameA(
                    h_process,
                    mod_handle,
                    mod_name.as_mut_ptr() as *mut i8,
                    256 as DWORD,
                ) != 0
                {
                    if let Ok(s) = CStr::from_ptr(mod_name.as_ptr() as *const i8).to_str() {
                        if s == name {
                            return Some(mod_handle);
                        }
                    }
                }
            }
        }
        None
    }
}

fn pattern_scan(base: *mut u8, size: usize, pattern: &[u8], mask: &str) -> *mut u8 {
    for i in 0..=size.saturating_sub(pattern.len()) {
        let mut found = true;
        for (j, &b) in pattern.iter().enumerate() {
            if mask.as_bytes()[j] == b'x' && unsafe { *base.add(i + j) } != b {
                found = false;
                break;
            }
        }
        if found {
            return unsafe { base.add(i) };
        }
    }
    ptr::null_mut()
}

unsafe extern "system" fn start_address(_lp_parameter: LPVOID) -> DWORD {
    init_console();

    let module_base = match get_module_by_name("Client-Win64-Shipping.exe") {
        Some(addr) => addr,
        None => {
            eprintln!("{RED}[ERROR]{RESET} Failed to get module base address");
            return 1;
        }
    };

    let mut mod_info = MODULEINFO {
        lpBaseOfDll: ptr::null_mut(),
        SizeOfImage: 0,
        EntryPoint: ptr::null_mut(),
    };

    if unsafe {
        GetModuleInformation(
            GetCurrentProcess(),
            module_base,
            &mut mod_info,
            std::mem::size_of::<MODULEINFO>() as DWORD,
        )
    } == 0
    {
        eprintln!("{RED}[ERROR]{RESET} Failed to get module information");
        return 1;
    }

    println!(
        "{YELLOW}[SCAN]{RESET} Module base address: {YELLOW}{:?}{RESET}",
        module_base
    );

    let pattern: [u8; 14] = [
        0x55, 0x53, 0x56, 0x41, 0x54, 0x41, 0x57, 0x48, 0x89, 0xE5, 0x48, 0x83, 0xEC, 0x60,
    ];
    let mask = "xxxxxxxxxxxxxx";

    let target_func = pattern_scan(
        mod_info.lpBaseOfDll as *mut u8,
        mod_info.SizeOfImage as usize,
        &pattern,
        mask,
    );

    if target_func.is_null() {
        eprintln!("{RED}[ERROR]{RESET} Failed to find target function using pattern scan");
        return 1;
    }

    println!(
        "{YELLOW}[SCAN]{RESET} Found target function at address: {YELLOW}{:p}{RESET}",
        target_func
    );

    let offset = (target_func as usize) - (module_base as usize);
    println!(
        "{YELLOW}[SCAN]{RESET} Target function offset: {YELLOW}{:#x}{RESET}",
        offset
    );

    let f_pak_file_check_preamble = unsafe { *(target_func as *const u64) };
    println!(
        "{CYAN}[INFO]{RESET} Using dynamic f_pak_file_check_preamble for ACE check: {CYAN}{:#x}{RESET}",
        f_pak_file_check_preamble
    );

    println!("{CYAN}[INFO]{RESET} Waiting for ACE init...");
    let pak_check_address = target_func as *const u64;
    loop {
        if unsafe { ptr::read_volatile(pak_check_address) } == f_pak_file_check_preamble {
            println!("{CYAN}[INFO]{RESET} {GREEN}ACE Initialization finished.{RESET}");
            break;
        }
        thread::sleep(Duration::from_millis(1));
    }

    println!("{MAGENTA}[HOOK]{RESET} Applying bypass hook to f_pak_file_check...");
    let mut interceptor = Interceptor::new();
    if let Err(e) = interceptor.replace(target_func as usize, f_pak_file_check_replacement, None) {
        eprintln!(
            "{RED}[ERROR]{RESET} Failed to hook function: {RED}{:?}{RESET}",
            e
        );
        return 1;
    }

    println!("{MAGENTA}[HOOK]{RESET} {GREEN}Bypass successfully applied!{RESET}");

    loop {
        thread::sleep(Duration::from_secs(u64::MAX));
    }
}

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
            )
        };
    }
    TRUE
}
