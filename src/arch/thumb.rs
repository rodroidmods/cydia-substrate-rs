use super::arm::*;
use crate::debug;
use crate::error::{Result, SubstrateError};
use crate::log::*;
use crate::memory::{self, MemoryGuard};

const T_NOP: u16 = 0x46c0;

fn t_bx(rm: u8) -> u16 {
    0x4700 | ((rm as u16) << 3)
}

fn t_blx(rm: u8) -> u16 {
    0x4780 | ((rm as u16) << 3)
}

fn t_add_rd_rm(rd: u8, rm: u8) -> u16 {
    0x4400
        | ((((rd) & 0x8) >> 3) << 7)
        | ((((rm) & 0x8) >> 3) << 6)
        | (((rm as u16) & 0x7) << 3)
        | ((rd as u16) & 0x7)
}

fn t_push_r(r: u32) -> u16 {
    0xb400 | ((((r) & (1 << ArmReg::LR as u32)) >> (ArmReg::LR as u32)) << 8) as u16 | (r as u16 & 0xff)
}

fn t_pop_r(r: u32) -> u16 {
    0xbc00 | ((((r) & (1 << ArmReg::PC as u32)) >> (ArmReg::PC as u32)) << 8) as u16 | (r as u16 & 0xff)
}

fn t_mov_rd_rm(rd: u8, rm: u8) -> u16 {
    0x4600
        | ((((rd) & 0x8) >> 3) << 7) as u16
        | ((((rm) & 0x8) >> 3) << 6) as u16
        | (((rm as u16) & 0x7) << 3)
        | ((rd as u16) & 0x7)
}

fn t_ldr_rd_rn_im4(rd: u8, rn: u8, im: u16) -> u16 {
    0x6800 | ((im & 0x1f) << 6) | ((rn as u16) << 3) | (rd as u16)
}

fn t_ldr_rd_pc_im4(rd: u8, im: u16) -> u16 {
    0x4800 | ((rd as u16) << 8) | (im & 0xff)
}

fn t_cmp_rn_im(rn: u8, im: u8) -> u16 {
    0x2000 | ((rn as u16) << 8) | (im as u16 & 0xff)
}

fn t_it_cd(cd: u8, ms: u8) -> u16 {
    0xbf00 | ((cd as u16) << 4) | (ms as u16)
}

fn t_cbz_rn_im(op: u8, rn: u8, im: u16) -> u16 {
    0xb100
        | ((op as u16) << 11)
        | ((((im) & 0x40) >> 6) << 9)
        | ((((im) & 0x3e) >> 1) << 3)
        | (rn as u16)
}

fn t_b_im(cond: u8, im: i32) -> u16 {
    if cond == ArmCond::Al as u8 {
        0xe000 | (((im >> 1) as u16) & 0x7ff)
    } else {
        0xd000 | ((cond as u16) << 8) | (((im >> 1) as u16) & 0xff)
    }
}

fn t1_ldr_rt_rn_im(rt: u8, rn: u8, im: i32) -> u16 {
    let u_bit = if im >= 0 { 1u16 } else { 0u16 };
    0xf850 | (u_bit << 7) | (rn as u16)
}

fn t2_ldr_rt_rn_im(rt: u8, _rn: u8, im: i32) -> u16 {
    ((rt as u16) << 12) | ((im.unsigned_abs() as u16) & 0xfff)
}

fn t1_mrs_rd_apsr(_rd: u8) -> u16 {
    0xf3ef
}

fn t2_mrs_rd_apsr(rd: u8) -> u16 {
    0x8000 | ((rd as u16) << 8)
}

fn t1_msr_apsr_nzcvqg_rn(rn: u8) -> u16 {
    0xf380 | (rn as u16)
}

fn t2_msr_apsr_nzcvqg_rn(_rn: u8) -> u16 {
    0x8c00
}

fn t_msr_apsr_nzcvqg_rn(rn: u8) -> u32 {
    ((t2_msr_apsr_nzcvqg_rn(rn) as u32) << 16) | (t1_msr_apsr_nzcvqg_rn(rn) as u32)
}

fn t_label(l: usize, r: usize) -> i32 {
    ((r as i32) - (l as i32)) * 2 - 4 + if l % 2 == 0 { 0 } else { 2 }
}

fn t_32bit_i(ic: u16) -> bool {
    (ic & 0xe000) == 0xe000 && (ic & 0x1800) != 0x0000
}

fn t_pcrel_cbz(ic: u16) -> bool {
    (ic & 0xf500) == 0xb100
}

fn t_pcrel_b(ic: u16) -> bool {
    (ic & 0xf000) == 0xd000 && (ic & 0x0e00) != 0x0e00
}

unsafe fn t2_pcrel_b(ic: *const u16) -> bool {
    unsafe {
        let ic0 = *ic;
        let ic1 = *ic.add(1);
        (ic0 & 0xf800) == 0xf000
            && (((ic1 & 0xd000) == 0x9000 || (ic1 & 0xd000) == 0x8000)
                && (ic0 & 0x0380) != 0x0380)
    }
}

unsafe fn t_pcrel_bl(ic: *const u16) -> bool {
    unsafe {
        let ic0 = *ic;
        let ic1 = *ic.add(1);
        (ic0 & 0xf800) == 0xf000
            && ((ic1 & 0xd000) == 0xd000 || (ic1 & 0xd001) == 0xc000)
    }
}

fn t_pcrel_ldr(ic: u16) -> bool {
    (ic & 0xf800) == 0x4800
}

fn t_pcrel_add(ic: u16) -> bool {
    (ic & 0xff78) == 0x4478
}

fn t_pcrel_ldrw(ic: u16) -> bool {
    (ic & 0xff7f) == 0xf85f
}

fn get_instruction_width_thumb(start: *const u8) -> usize {
    let thumb = start as *const u16;
    let ic = unsafe { *thumb };
    if t_32bit_i(ic) { 4 } else { 2 }
}

pub unsafe fn hook_function_thumb(
    symbol: *mut u8,
    replace: *mut u8,
    result: *mut *mut u8,
) -> Result<usize> {
    if symbol.is_null() {
        return Err(SubstrateError::NullSymbol);
    }

    info!("SubstrateHookFunctionThumb");

    let area = symbol as *mut u16;
    let align: usize = if (area as usize & 0x2) == 0 { 0 } else { 1 };
    let thumb = unsafe { area.add(align) };
    let arm = unsafe { thumb.add(2) as *mut u32 };
    let trail = unsafe { arm.add(2) as *mut u16 };

    unsafe {

    if (align == 0 || *area == T_NOP)
        && *thumb == t_bx(ArmReg::PC as u8)
        && *thumb.add(1) == T_NOP
        && *arm == a_ldr_rd_rn_im(ArmReg::PC as u8, ArmReg::PC as u8, 4 - 8)
    {
        if !result.is_null() {
            *result = (*arm.add(1)) as *mut u8;
        }

        let _guard = MemoryGuard::new(arm.add(1) as *mut u8, 4)?;
        *arm.add(1) = replace as u32;

        return Ok(4);
    }

    let required = ((trail as usize) - (area as usize)) as usize;

    let mut used: usize = 0;
    while used < required {
        used += get_instruction_width_thumb((area as *const u8).add(used));
    }
    used = (used + 1) / 2 * 2;

    let blank = (used - required) / 2;

    let mut backup = vec![0u16; used / 2];
    core::ptr::copy_nonoverlapping(area as *const u16, backup.as_mut_ptr(), used / 2);

    if debug::is_debug() {
        let name = format!("{:p}", area);
        debug::log_hex_ex(
            core::slice::from_raw_parts(area as *const u8, used + 2),
            2,
            Some(&name),
        );
    }

    if !result.is_null() {
        let mut length = used;
        let mut offset = 0usize;
        while offset < used / 2 {
            if t_pcrel_ldr(backup[offset]) {
                length += 3 * 2;
            } else if t_pcrel_b(backup[offset]) {
                length += 6 * 2;
            } else if offset + 1 < backup.len() && t2_pcrel_b(backup.as_ptr().add(offset)) {
                length += 5 * 2;
                offset += 1;
            } else if offset + 1 < backup.len() && t_pcrel_bl(backup.as_ptr().add(offset)) {
                length += 5 * 2;
                offset += 1;
            } else if t_pcrel_cbz(backup[offset]) {
                length += 16 * 2;
            } else if t_pcrel_ldrw(backup[offset]) {
                length += 4 * 2;
                offset += 1;
            } else if t_pcrel_add(backup[offset]) {
                length += 6 * 2;
            } else if t_32bit_i(backup[offset]) {
                offset += 1;
            }
            offset += 1;
        }

        let pad: usize = if length & 0x2 == 0 { 0 } else { 1 };
        length += (pad + 2) * 2 + 2 * 4;

        let buffer = memory::alloc_rwx(length)? as *mut u16;

        let mut start = pad;
        let mut end = length / 2;
        let mut trailer = buffer.add(end) as *mut u32;

        let mut offset = 0usize;
        while offset < used / 2 {
            if t_pcrel_ldr(backup[offset]) {
                let immediate = (backup[offset] & 0xff) as u16;
                let rd = ((backup[offset] >> 8) & 0x7) as u8;

                *buffer.add(start) = t_ldr_rd_pc_im4(rd, (t_label(start, end - 2) / 4) as u16);
                *buffer.add(start + 1) = t_ldr_rd_rn_im4(rd, rd, 0);

                trailer = trailer.sub(1);
                *trailer = ((area.add(offset) as u32).wrapping_add(4) & !0x2)
                    .wrapping_add((immediate as u32) * 4);

                start += 2;
                end -= 2;
            } else if t_pcrel_b(backup[offset]) {
                let cond = ((backup[offset] >> 8) & 0xf) as u8;
                let mut jump = ((backup[offset] & 0xff) as i32) << 1;
                jump |= 1;
                jump <<= 23;
                jump >>= 23;

                *buffer.add(start) = t_b_im(cond, ((end as i32 - 6) - (start as i32)) * 2 - 4);

                trailer = trailer.sub(1);
                *trailer = (area.add(offset) as u32).wrapping_add(4).wrapping_add(jump as u32);
                trailer = trailer.sub(1);
                *trailer = a_ldr_rd_rn_im(ArmReg::PC as u8, ArmReg::PC as u8, 4 - 8);
                trailer = trailer.sub(1);
                *trailer = ((T_NOP as u32) << 16) | (t_bx(ArmReg::PC as u8) as u32);

                start += 1;
                end -= 6;
            } else if offset + 1 < backup.len() && t2_pcrel_b(backup.as_ptr().add(offset)) {
                let ic0 = backup[offset];
                let ic1 = backup[offset + 1];

                let imm6 = (ic0 & 0x3f) as u32;
                let cond = ((ic0 >> 6) & 0xf) as u8;
                let s = ((ic0 >> 10) & 1) as u32;

                let imm11 = (ic1 & 0x7ff) as u32;
                let j2 = ((ic1 >> 11) & 1) as u32;
                let a = ((ic1 >> 12) & 1) as u32;
                let j1 = ((ic1 >> 13) & 1) as u32;

                let mut jump: i32 = 1;
                jump |= (imm11 << 1) as i32;
                jump |= (imm6 << 12) as i32;

                if a != 0 {
                    jump |= (s << 24) as i32;
                    jump |= ((!(s ^ j1) & 0x1) << 23) as i32;
                    jump |= ((!(s ^ j2) & 0x1) << 22) as i32;
                    jump |= (cond as i32) << 18;
                    jump <<= 7;
                    jump >>= 7;
                } else {
                    jump |= (s << 20) as i32;
                    jump |= (j2 << 19) as i32;
                    jump |= (j1 << 18) as i32;
                    jump <<= 11;
                    jump >>= 11;
                }

                let branch_cond = if a != 0 { ArmCond::Al as u8 } else { cond };
                *buffer.add(start) =
                    t_b_im(branch_cond, ((end as i32 - 6) - (start as i32)) * 2 - 4);

                trailer = trailer.sub(1);
                *trailer = (area.add(offset) as u32).wrapping_add(4).wrapping_add(jump as u32);
                trailer = trailer.sub(1);
                *trailer = a_ldr_rd_rn_im(ArmReg::PC as u8, ArmReg::PC as u8, 4 - 8);
                trailer = trailer.sub(1);
                *trailer = ((T_NOP as u32) << 16) | (t_bx(ArmReg::PC as u8) as u32);

                offset += 1;
                start += 1;
                end -= 6;
            } else if offset + 1 < backup.len() && t_pcrel_bl(backup.as_ptr().add(offset)) {
                let ic0 = backup[offset];
                let ic1 = backup[offset + 1];

                let bits_immediate = (ic0 & 0x3ff) as u32;
                let bits_s = ((ic0 >> 10) & 1) as u32;

                let exts_immediate = (ic1 & 0x7ff) as u32;
                let exts_j2 = ((ic1 >> 11) & 1) as u32;
                let exts_x = ((ic1 >> 12) & 1) as u32;
                let exts_j1 = ((ic1 >> 13) & 1) as u32;

                let mut jump: i32 = 0;
                jump |= (bits_s << 24) as i32;
                jump |= ((!(bits_s ^ exts_j1) & 0x1) << 23) as i32;
                jump |= ((!(bits_s ^ exts_j2) & 0x1) << 22) as i32;
                jump |= (bits_immediate << 12) as i32;
                jump |= (exts_immediate << 1) as i32;
                jump |= exts_x as i32;
                jump <<= 7;
                jump >>= 7;

                *buffer.add(start) = t_push_r(1 << ArmReg::R7 as u32);
                *buffer.add(start + 1) = t_ldr_rd_pc_im4(
                    ArmReg::R7 as u8,
                    (((end as i32 - 2) - (start as i32 + 1)) * 2 - 4 + 2) as u16 / 4,
                );
                *buffer.add(start + 2) = t_mov_rd_rm(ArmReg::LR as u8, ArmReg::R7 as u8);
                *buffer.add(start + 3) = t_pop_r(1 << ArmReg::R7 as u32);
                *buffer.add(start + 4) = t_blx(ArmReg::LR as u8);

                trailer = trailer.sub(1);
                *trailer = (area.add(offset) as u32).wrapping_add(4).wrapping_add(jump as u32);

                offset += 1;
                start += 5;
                end -= 2;
            } else if t_pcrel_cbz(backup[offset]) {
                let ic = backup[offset];
                let rn = (ic & 0x7) as u8;
                let immediate = ((ic >> 3) & 0x1f) as u16;
                let i_bit = ((ic >> 9) & 1) as u16;
                let op = ((ic >> 11) & 1) as u8;

                let mut jump: i32 = 1;
                jump |= (i_bit as i32) << 6;
                jump |= (immediate as i32) << 1;

                let rt = if rn == ArmReg::R7 as u8 { ArmReg::R6 as u8 } else { ArmReg::R7 as u8 };

                *buffer.add(start) = t_push_r(1 << rt as u32);
                *buffer.add(start + 1) = t1_mrs_rd_apsr(rt);
                *buffer.add(start + 2) = t2_mrs_rd_apsr(rt);
                *buffer.add(start + 3) =
                    t_cbz_rn_im(op, rn, ((end as i32 - 10) - (start as i32 + 3)) as u16 * 2 - 4);
                *buffer.add(start + 4) = t1_msr_apsr_nzcvqg_rn(rt);
                *buffer.add(start + 5) = t2_msr_apsr_nzcvqg_rn(rt);
                *buffer.add(start + 6) = t_pop_r(1 << rt as u32);

                trailer = trailer.sub(1);
                *trailer = (area.add(offset) as u32).wrapping_add(4).wrapping_add(jump as u32);
                trailer = trailer.sub(1);
                *trailer = a_ldr_rd_rn_im(ArmReg::PC as u8, ArmReg::PC as u8, 4 - 8);
                trailer = trailer.sub(1);
                *trailer = ((T_NOP as u32) << 16) | (t_bx(ArmReg::PC as u8) as u32);
                trailer = trailer.sub(1);
                *trailer = ((T_NOP as u32) << 16) | (t_pop_r(1 << rt as u32) as u32);
                trailer = trailer.sub(1);
                *trailer = t_msr_apsr_nzcvqg_rn(rt);

                start += 7;
                end -= 10;
            } else if t_pcrel_ldrw(backup[offset]) {
                let ic0 = backup[offset];
                let ic1 = backup[offset + 1];

                let u_bit = ((ic0 >> 7) & 1) as u8;
                let immediate = (ic1 & 0xfff) as i32;
                let rt = ((ic1 >> 12) & 0xf) as u8;

                let label = t_label(start, end - 2);

                *buffer.add(start) = t1_ldr_rt_rn_im(rt, ArmReg::PC as u8, label);
                *buffer.add(start + 1) = t2_ldr_rt_rn_im(rt, ArmReg::PC as u8, label);

                *buffer.add(start + 2) = t1_ldr_rt_rn_im(rt, rt, 0);
                *buffer.add(start + 3) = t2_ldr_rt_rn_im(rt, rt, 0);

                let target_addr = (area.add(offset) as u32).wrapping_add(4) & !0x2;
                let final_addr = if u_bit == 0 {
                    target_addr.wrapping_sub(immediate as u32)
                } else {
                    target_addr.wrapping_add(immediate as u32)
                };

                trailer = trailer.sub(1);
                *trailer = final_addr;

                offset += 1;
                start += 4;
                end -= 2;
            } else if t_pcrel_add(backup[offset]) {
                let ic = backup[offset];
                let rd = (ic & 0x7) as u8;
                let rm = ((ic >> 3) & 0x7) as u8;
                let h1 = ((ic >> 7) & 1) as u8;

                if h1 != 0 {
                    error!("pcrel({}):add (rd > r7)", offset);
                    memory::dealloc(buffer as *mut u8, length);
                    if !result.is_null() {
                        *result = core::ptr::null_mut();
                    }
                    return Ok(0);
                }

                let rt = if rd == ArmReg::R7 as u8 { ArmReg::R6 as u8 } else { ArmReg::R7 as u8 };

                *buffer.add(start) = t_push_r(1 << rt as u32);
                *buffer.add(start + 1) = t_mov_rd_rm(rt, (h1 << 3) | rd);
                *buffer.add(start + 2) =
                    t_ldr_rd_pc_im4(rd, (t_label(start + 2, end - 2) / 4) as u16);
                *buffer.add(start + 3) = t_add_rd_rm((h1 << 3) | rd, rt);
                *buffer.add(start + 4) = t_pop_r(1 << rt as u32);

                trailer = trailer.sub(1);
                *trailer = (area.add(offset) as u32).wrapping_add(4);

                start += 5;
                end -= 2;
            } else if t_32bit_i(backup[offset]) {
                *buffer.add(start) = backup[offset];
                start += 1;
                offset += 1;
                *buffer.add(start) = backup[offset];
                start += 1;
            } else {
                *buffer.add(start) = backup[offset];
                start += 1;
            }
            offset += 1;
        }

        *buffer.add(start) = t_bx(ArmReg::PC as u8);
        start += 1;
        *buffer.add(start) = T_NOP;
        start += 1;

        let transfer = buffer.add(start) as *mut u32;
        *transfer = a_ldr_rd_rn_im(ArmReg::PC as u8, ArmReg::PC as u8, 4 - 8);
        *transfer.add(1) = (area.add(used / 2) as u32) + 1;

        memory::protect_rx(buffer as *mut u8, length)?;

        *result = (buffer.add(pad) as *mut u8).add(1);

        if debug::is_debug() {
            let name = format!("{:p}", *result);
            debug::log_hex_ex(
                core::slice::from_raw_parts(buffer as *const u8, length),
                2,
                Some(&name),
            );
        }
    }

    {
        let _guard = MemoryGuard::new(area as *mut u8, used)?;

        if align != 0 {
            *area = T_NOP;
        }

        *thumb = t_bx(ArmReg::PC as u8);
        *thumb.add(1) = T_NOP;

        *arm = a_ldr_rd_rn_im(ArmReg::PC as u8, ArmReg::PC as u8, 4 - 8);
        *arm.add(1) = replace as u32;

        for i in 0..blank {
            *trail.add(i) = T_NOP;
        }
    }

    if debug::is_debug() {
        let name = format!("{:p}", area);
        debug::log_hex_ex(
            core::slice::from_raw_parts(area as *const u8, used + 2),
            2,
            Some(&name),
        );
    }

    } // unsafe

    Ok(used)
}
