use ilhook::x64::Registers;
use interceptor_rs::Interceptor;

use crate::error::{AppError, Result};

pub struct PakFileHook {
    interceptor: Interceptor,
}

impl PakFileHook {
    pub fn new() -> Self {
        Self {
            interceptor: Interceptor::new(),
        }
    }

    pub fn apply(
        &mut self,
        target_address: usize,
        replacement: unsafe extern "win64" fn(*mut Registers, usize, usize) -> usize,
    ) -> Result<()> {
        self.interceptor
            .replace(target_address, replacement, None)
            .map_err(|e| AppError::HookFailed(format!("{:?}", e)))?;

        Ok(())
    }
}
