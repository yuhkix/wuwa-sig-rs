use ilhook::x64::Registers;
use interceptor_rs::Interceptor;
use std::sync::{Arc, Mutex};

use crate::error::{AppError, Result};
use crate::logger::Logger;

/// Hook state for tracking and management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookState {
    Uninitialized,
    Applied,
    Failed,
    Removed,
}

/// Enhanced PAK file hook with better error handling and state management
pub struct PakFileHook {
    interceptor: Arc<Mutex<Interceptor>>,
    state: Arc<Mutex<HookState>>,
    target_address: Arc<Mutex<Option<usize>>>,
}

impl PakFileHook {
    /// Create a new PAK file hook
    pub fn new() -> Self {
        Self {
            interceptor: Arc::new(Mutex::new(Interceptor::new())),
            state: Arc::new(Mutex::new(HookState::Uninitialized)),
            target_address: Arc::new(Mutex::new(None)),
        }
    }

    /// Apply the hook to the target address
    pub fn apply(
        &self,
        target_address: usize,
        replacement: unsafe extern "win64" fn(*mut Registers, usize, usize) -> usize,
    ) -> Result<()> {
        // Check current state
        {
            let state = self.state.lock().unwrap();
            if *state == HookState::Applied {
                return Err(AppError::HookFailed {
                    message: "Hook is already applied".to_string(),
                });
            }
        }

        Logger::hook(&format!("Applying hook to address: {:#x}", target_address));

        // Apply the hook
        let result = {
            let mut interceptor = self.interceptor.lock().unwrap();
            Logger::hook(&format!(
                "Calling interceptor.replace with address: {:#x}",
                target_address
            ));
            let res = interceptor.replace(target_address, replacement, None);
            Logger::hook(&format!("Interceptor.replace result: {:?}", res));
            res
        };

        match result {
            Ok(_) => {
                // Update state and target address
                {
                    let mut state = self.state.lock().unwrap();
                    *state = HookState::Applied;
                }
                {
                    let mut addr = self.target_address.lock().unwrap();
                    *addr = Some(target_address);
                }

                Logger::success(&format!(
                    "Hook successfully applied to {:#x}",
                    target_address
                ));
                Ok(())
            }
            Err(e) => {
                // Update state to failed
                {
                    let mut state = self.state.lock().unwrap();
                    *state = HookState::Failed;
                }

                let error_msg = format!("Failed to apply hook: {:?}", e);
                Logger::error(&error_msg);
                Err(AppError::HookFailed { message: error_msg })
            }
        }
    }

    /// Remove the hook if it's currently applied
    pub fn remove(&self) -> Result<()> {
        let current_state = {
            let state = self.state.lock().unwrap();
            *state
        };

        if current_state != HookState::Applied {
            return Err(AppError::HookFailed {
                message: format!("Cannot remove hook in state: {:?}", current_state),
            });
        }

        // Note: The interceptor-rs library doesn't provide a remove method
        // This is a placeholder for future implementation
        Logger::warning("Hook removal not implemented in interceptor-rs");

        {
            let mut state = self.state.lock().unwrap();
            *state = HookState::Removed;
        }

        Ok(())
    }

    /// Get the current hook state
    pub fn state(&self) -> HookState {
        let state = self.state.lock().unwrap();
        *state
    }

    /// Get the target address if the hook is applied
    pub fn target_address(&self) -> Option<usize> {
        let addr = self.target_address.lock().unwrap();
        *addr
    }

    /// Check if the hook is currently active
    pub fn is_active(&self) -> bool {
        let state = self.state.lock().unwrap();
        *state == HookState::Applied
    }

    /// Get hook statistics and information
    pub fn info(&self) -> HookInfo {
        let state = self.state.lock().unwrap();
        let target_addr = self.target_address.lock().unwrap();

        HookInfo {
            state: *state,
            target_address: *target_addr,
            is_active: *state == HookState::Applied,
        }
    }
}

/// Information about the hook state
#[derive(Debug, Clone)]
pub struct HookInfo {
    pub state: HookState,
    pub target_address: Option<usize>,
    pub is_active: bool,
}

impl Default for PakFileHook {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-safe implementation
unsafe impl Send for PakFileHook {}
unsafe impl Sync for PakFileHook {}
