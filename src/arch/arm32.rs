use super::arm::*;
use crate::debug;
use crate::error::{Result, SubstrateError};
use crate::log::*;
use crate::memory::{self, MemoryGuard};

pub unsafe fn hook_function_arm(
    symbol: *mut u8,
    replace: *mut u8,
    result: *mut *mut u8,
) -> Result<usize> {
    if symbol.is_null() {
        return Err(SubstrateError::NullSymbol);
    }

    info!("SubstrateHookFunctionARM");

    let area = symbol as *mut u32;
    let arm = area;
    let used: usize = 8;

    unsafe {

    let backup = [*arm, *arm.add(1)];

    if debug::is_debug() {
        let name = format!("{:p}", area);
        debug::log_hex_ex(
            core::slice::from_raw_parts(area as *const u8, used + 4),
            4,
            Some(&name),
        );
    }

    if !result.is_null() {
        if backup[0] == a_ldr_rd_rn_im(ArmReg::PC as u8, ArmReg::PC as u8, 4 - 8) {
            *result = backup[1] as *mut u8;
            return Ok(4);
        }

        let mut length = used;
        for offset in 0..(used / 4) {
            if a_pcrel_r(backup[offset]) {
                let mode_bit = (backup[offset] >> 25) & 1;
                let rd = (backup[offset] >> 12) & 0xf;
                let rm = backup[offset] & 0xf;
                if mode_bit == 0 || rd != rm {
                    length += 2 * 4;
                } else {
                    length += 4 * 4;
                }
            }
        }

        length += 2 * 4;

        let buffer = memory::alloc_rwx(length)? as *mut u32;

        let mut start = 0usize;
        let mut end = length / 4;
        let mut trailer = buffer.add(end);

        for offset in 0..(used / 4) {
            if a_pcrel_r(backup[offset]) {
                let rm = (backup[offset] & 0xf) as u8;
                let rd = ((backup[offset] >> 12) & 0xf) as u8;
                let mode_bit = (backup[offset] >> 25) & 1;

                let mut copy = backup[offset];
                let guard;

                if mode_bit == 0 || rd != rm {
                    copy = (copy & !0x000f0000) | ((rd as u32) << 16);
                    guard = false;
                } else {
                    let alt = if rm != ArmReg::R0 as u8 { ArmReg::R0 as u8 } else { ArmReg::R1 as u8 };
                    copy = (copy & !0x000f0000) | ((alt as u32) << 16);
                    guard = true;
                }

                if guard {
                    let copy_rn = ((copy >> 16) & 0xf) as u8;
                    *buffer.add(start) = a_stmdb_sp_rs(1 << copy_rn as u32);
                    start += 1;
                }

                let copy_rn = ((copy >> 16) & 0xf) as u8;
                *buffer.add(start) = a_ldr_rd_rn_im(
                    copy_rn,
                    ArmReg::PC as u8,
                    ((end as i32 - 1) - (start as i32)) * 4 - 8,
                );
                *buffer.add(start + 1) = copy;
                start += 2;

                if guard {
                    let copy_rn = ((copy >> 16) & 0xf) as u8;
                    *buffer.add(start) = a_ldmia_sp_rs(1 << copy_rn as u32);
                    start += 1;
                }

                trailer = trailer.sub(1);
                *trailer = (area.add(offset) as u32).wrapping_add(8);
                end -= 1;
            } else {
                *buffer.add(start) = backup[offset];
                start += 1;
            }
        }

        *buffer.add(start) = a_ldr_rd_rn_im(ArmReg::PC as u8, ArmReg::PC as u8, 4 - 8);
        *buffer.add(start + 1) = area.add(used / 4) as u32;

        memory::protect_rx(buffer as *mut u8, length)?;

        *result = buffer as *mut u8;

        if debug::is_debug() {
            let name = format!("{:p}", *result);
            debug::log_hex_ex(
                core::slice::from_raw_parts(buffer as *const u8, length),
                4,
                Some(&name),
            );
        }
    }

    {
        let _guard = MemoryGuard::new(symbol, used)?;

        *arm = a_ldr_rd_rn_im(ArmReg::PC as u8, ArmReg::PC as u8, 4 - 8);
        *arm.add(1) = replace as u32;
    }

    if debug::is_debug() {
        let name = format!("{:p}", area);
        debug::log_hex_ex(
            core::slice::from_raw_parts(area as *const u8, used + 4),
            4,
            Some(&name),
        );
    }

    } // unsafe

    Ok(used)
}
