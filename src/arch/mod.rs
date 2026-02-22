#[cfg(target_arch = "arm")]
pub mod arm;
#[cfg(target_arch = "arm")]
pub mod thumb;
#[cfg(target_arch = "arm")]
pub mod arm32;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod x86;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod x86_hook;

use crate::debug;
use crate::error::{Result, SubstrateError};
use crate::log::*;

pub unsafe fn hook_function(
    symbol: *mut u8,
    replace: *mut u8,
    result: *mut *mut u8,
) -> Result<usize> {
    if symbol.is_null() {
        return Err(SubstrateError::NullSymbol);
    }

    if debug::is_debug() {
        info!(
            "hook_function(symbol={:p}, replace={:p}, result={:p})",
            symbol, replace, result
        );
    }

    #[cfg(target_arch = "arm")]
    {
        let addr = symbol as usize;
        if addr & 1 == 0 {
            return unsafe { arm32::hook_function_arm(symbol, replace, result) };
        } else {
            let thumb_addr = (addr & !1) as *mut u8;
            return unsafe { thumb::hook_function_thumb(thumb_addr, replace, result) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        return unsafe { aarch64::hook_function_aarch64(symbol, replace, result) };
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        return unsafe { x86_hook::hook_function_x86(symbol, replace, result) };
    }

    #[allow(unreachable_code)]
    Err(SubstrateError::NullSymbol)
}

