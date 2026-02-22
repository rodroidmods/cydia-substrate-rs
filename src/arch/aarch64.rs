use core::sync::atomic::{AtomicI32, Ordering};
use crate::error::{Result, SubstrateError};
use crate::log::*;

const A64_MAX_BACKUPS: usize = 256;
const A64_MAX_INSTRUCTIONS: usize = 5;
const A64_MAX_REFERENCES: usize = A64_MAX_INSTRUCTIONS * 2;
const A64_NOP: u32 = 0xd503201f;
const PAGE_SIZE: usize = 4096;

fn align_up(x: usize, n: usize) -> usize {
    (x + (n - 1)) & !(n - 1)
}

fn align_down(x: usize, n: usize) -> usize {
    x & (-(n as isize) as usize)
}

unsafe fn make_rwx(p: *mut u8, n: usize) -> i32 {
    let ptr_val = p as usize;
    let aligned = align_down(ptr_val, PAGE_SIZE);
    let size = if align_up(ptr_val + n, PAGE_SIZE) != align_up(ptr_val, PAGE_SIZE) {
        align_up(n, PAGE_SIZE) + PAGE_SIZE
    } else {
        align_up(n, PAGE_SIZE)
    };
    unsafe {
        libc::mprotect(
            aligned as *mut libc::c_void,
            size,
            libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
        )
    }
}

unsafe fn flush_cache(c: *mut u8, n: usize) {
    unsafe {
        core::arch::asm!(
            "
            // dc cvau: clean data cache by VA to PoU
            // ic ivau: invalidate instruction cache by VA to PoU
            // dsb ish: data synchronization barrier (inner shareable)
            // isb: instruction synchronization barrier
            1:
                dc cvau, {addr}
                ic ivau, {addr}
                add {addr}, {addr}, #4
                subs {len}, {len}, #4
                b.gt 1b
                dsb ish
                isb
            ",
            addr = inout(reg) c as usize => _,
            len = inout(reg) n => _,
            options(nostack)
        );
    }
}

#[derive(Clone, Copy, Default)]
struct FixInfo {
    bp: *mut u32,
    ls: u32,
    ad: u32,
}

#[derive(Clone, Copy)]
struct InsnsInfo {
    ins: i64,
    fmap: [FixInfo; A64_MAX_REFERENCES],
}

impl Default for InsnsInfo {
    fn default() -> Self {
        Self {
            ins: 0,
            fmap: [FixInfo {
                bp: core::ptr::null_mut(),
                ls: 0,
                ad: 0,
            }; A64_MAX_REFERENCES],
        }
    }
}

struct Context {
    basep: i64,
    endp: i64,
    dat: [InsnsInfo; A64_MAX_INSTRUCTIONS],
}

impl Context {
    fn new(inp: *mut u32, count: i32) -> Self {
        Self {
            basep: inp as i64,
            endp: unsafe { inp.offset(count as isize) } as i64,
            dat: [InsnsInfo::default(); A64_MAX_INSTRUCTIONS],
        }
    }

    fn is_in_fixing_range(&self, absolute_addr: i64) -> bool {
        absolute_addr >= self.basep && absolute_addr < self.endp
    }

    fn get_ref_ins_index(&self, absolute_addr: i64) -> isize {
        ((absolute_addr - self.basep) / 4) as isize
    }

    fn get_and_set_current_index(&mut self, inp: *mut u32, outp: *mut u32) -> isize {
        let current_idx = self.get_ref_ins_index(inp as i64);
        self.dat[current_idx as usize].ins = outp as i64;
        current_idx
    }

    fn reset_current_ins(&mut self, idx: isize, outp: *mut u32) {
        self.dat[idx as usize].ins = outp as i64;
    }

    fn insert_fix_map(&mut self, idx: isize, bp: *mut u32, ls: u32, ad: u32) {
        for f in self.dat[idx as usize].fmap.iter_mut() {
            if f.bp.is_null() {
                f.bp = bp;
                f.ls = ls;
                f.ad = ad;
                return;
            }
        }
    }

    fn process_fix_map(&mut self, idx: isize) {
        for f in self.dat[idx as usize].fmap.iter_mut() {
            if f.bp.is_null() {
                break;
            }
            unsafe {
                let bp_val = *f.bp;
                let offset = (self.dat[idx as usize].ins - f.bp as i64) >> 2;
                *f.bp = bp_val | (((offset as i32) << f.ls) as u32 & f.ad);
            }
            f.bp = core::ptr::null_mut();
        }
    }
}

unsafe fn fix_branch_imm(
    inpp: &mut *mut u32,
    outpp: &mut *mut u32,
    ctx: &mut Context,
) -> bool {
    const MBITS: u32 = 6;
    const MASK: u32 = 0xfc000000;
    const RMASK: u32 = 0x03ffffff;
    const OP_B: u32 = 0x14000000;
    const OP_BL: u32 = 0x94000000;

    let ins = unsafe { **inpp };
    let opc = ins & MASK;
    match opc {
        OP_B | OP_BL => {
            let current_idx = ctx.get_and_set_current_index(*inpp, *outpp);
            let absolute_addr =
                *inpp as i64 + ((((ins << MBITS) as i32) >> (MBITS - 2)) as i64);
            let mut new_pc_offset =
                (absolute_addr - *outpp as i64) >> 2;
            let special_fix_type = ctx.is_in_fixing_range(absolute_addr);

            if !special_fix_type && (new_pc_offset.unsigned_abs() as u64) >= (RMASK >> 1) as u64 {
                let b_aligned = ((*outpp as usize).wrapping_add(8) & 7) == 0;
                if opc == OP_B {
                    if !b_aligned {
                        unsafe { **outpp = A64_NOP; }
                        *outpp = unsafe { (*outpp).add(1) };
                        ctx.reset_current_ins(current_idx, *outpp);
                    }
                    unsafe {
                        **outpp = 0x58000051; // LDR X17, #0x8
                        *(*outpp).add(1) = 0xd61f0220; // BR X17
                        core::ptr::copy_nonoverlapping(
                            &absolute_addr as *const i64 as *const u8,
                            (*outpp).add(2) as *mut u8,
                            8,
                        );
                    }
                    *outpp = unsafe { (*outpp).add(4) };
                } else {
                    if b_aligned {
                        unsafe { **outpp = A64_NOP; }
                        *outpp = unsafe { (*outpp).add(1) };
                        ctx.reset_current_ins(current_idx, *outpp);
                    }
                    unsafe {
                        **outpp = 0x58000071; // LDR X17, #12
                        *(*outpp).add(1) = 0x1000009e; // ADR X30, #16
                        *(*outpp).add(2) = 0xd61f0220; // BR X17
                        core::ptr::copy_nonoverlapping(
                            &absolute_addr as *const i64 as *const u8,
                            (*outpp).add(3) as *mut u8,
                            8,
                        );
                    }
                    *outpp = unsafe { (*outpp).add(5) };
                }
            } else {
                if special_fix_type {
                    let ref_idx = ctx.get_ref_ins_index(absolute_addr);
                    if ref_idx <= current_idx {
                        new_pc_offset =
                            (ctx.dat[ref_idx as usize].ins - *outpp as i64) >> 2;
                    } else {
                        ctx.insert_fix_map(ref_idx, *outpp, 0, RMASK);
                        new_pc_offset = 0;
                    }
                }
                unsafe {
                    **outpp = opc | ((new_pc_offset as u32) & !MASK);
                }
                *outpp = unsafe { (*outpp).add(1) };
            }

            *inpp = unsafe { (*inpp).add(1) };
            ctx.process_fix_map(current_idx);
            true
        }
        _ => false,
    }
}

unsafe fn fix_cond_comp_test_branch(
    inpp: &mut *mut u32,
    outpp: &mut *mut u32,
    ctx: &mut Context,
) -> bool {
    const LSB: u32 = 5;
    const LMASK01: u32 = 0xff00001f;
    const MASK0: u32 = 0xff000010;
    const OP_BC: u32 = 0x54000000;
    const MASK1: u32 = 0x7f000000;
    const OP_CBZ: u32 = 0x34000000;
    const OP_CBNZ: u32 = 0x35000000;
    const LMASK2: u32 = 0xfff8001f;
    const MASK2: u32 = 0x7f000000;
    const OP_TBZ: u32 = 0x36000000;
    const OP_TBNZ: u32 = 0x37000000;

    let ins = unsafe { **inpp };
    let mut lmask = LMASK01;
    if (ins & MASK0) != OP_BC {
        let opc = ins & MASK1;
        if opc != OP_CBZ && opc != OP_CBNZ {
            let opc2 = ins & MASK2;
            if opc2 != OP_TBZ && opc2 != OP_TBNZ {
                return false;
            }
            lmask = LMASK2;
        }
    }

    let msb = (!lmask).leading_zeros();
    let current_idx = ctx.get_and_set_current_index(*inpp, *outpp);
    let absolute_addr = *inpp as i64
        + ((((ins & !lmask) << msb) as i32) >> (LSB - 2 + msb)) as i64;
    let mut new_pc_offset = (absolute_addr - *outpp as i64) >> 2;
    let special_fix_type = ctx.is_in_fixing_range(absolute_addr);

    if !special_fix_type
        && (new_pc_offset.unsigned_abs() as u64) >= ((!lmask) >> (LSB + 1)) as u64
    {
        if ((*outpp as usize).wrapping_add(16) & 7) != 0 {
            unsafe { **outpp = A64_NOP; }
            *outpp = unsafe { (*outpp).add(1) };
            ctx.reset_current_ins(current_idx, *outpp);
        }
        unsafe {
            **outpp = (((8u32 >> 2) << LSB) & !lmask) | (ins & lmask); // B.C #0x8
            *(*outpp).add(1) = 0x14000005; // B #0x14
            *(*outpp).add(2) = 0x58000051; // LDR X17, #0x8
            *(*outpp).add(3) = 0xd61f0220; // BR X17
            core::ptr::copy_nonoverlapping(
                &absolute_addr as *const i64 as *const u8,
                (*outpp).add(4) as *mut u8,
                8,
            );
        }
        *outpp = unsafe { (*outpp).add(6) };
    } else {
        if special_fix_type {
            let ref_idx = ctx.get_ref_ins_index(absolute_addr);
            if ref_idx <= current_idx {
                new_pc_offset =
                    (ctx.dat[ref_idx as usize].ins - *outpp as i64) >> 2;
            } else {
                ctx.insert_fix_map(ref_idx, *outpp, LSB, !lmask);
                new_pc_offset = 0;
            }
        }
        unsafe {
            **outpp = ((new_pc_offset as u32) << LSB & !lmask) | (ins & lmask);
        }
        *outpp = unsafe { (*outpp).add(1) };
    }

    *inpp = unsafe { (*inpp).add(1) };
    ctx.process_fix_map(current_idx);
    true
}

unsafe fn fix_loadlit(
    inpp: &mut *mut u32,
    outpp: &mut *mut u32,
    ctx: &mut Context,
) -> bool {
    let ins = unsafe { **inpp };

    // memory prefetch ("prfm"), just skip it
    if (ins & 0xff000000) == 0xd8000000 {
        let idx = ctx.get_and_set_current_index(*inpp, *outpp);
        ctx.process_fix_map(idx);
        *inpp = unsafe { (*inpp).add(1) };
        return true;
    }

    const MSB: u32 = 8;
    const LSB: u32 = 5;
    const MASK_30: u32 = 0x40000000;
    const MASK_31: u32 = 0x80000000;
    const LMASK: u32 = 0xff00001f;
    const MASK_LDR: u32 = 0xbf000000;
    const OP_LDR: u32 = 0x18000000;
    const MASK_LDRV: u32 = 0x3f000000;
    const OP_LDRV: u32 = 0x1c000000;
    const MASK_LDRSW: u32 = 0xff000000;
    const OP_LDRSW: u32 = 0x98000000;

    let mut mask = MASK_LDR;
    let mut faligned: usize = if (ins & MASK_30) != 0 { 7 } else { 3 };
    if (ins & MASK_LDR) != OP_LDR {
        mask = MASK_LDRV;
        if faligned != 7 {
            faligned = if (ins & MASK_31) != 0 { 15 } else { 3 };
        }
        if (ins & MASK_LDRV) != OP_LDRV {
            if (ins & MASK_LDRSW) != OP_LDRSW {
                return false;
            }
            mask = MASK_LDRSW;
            faligned = 7;
        }
    }

    let current_idx = ctx.get_and_set_current_index(*inpp, *outpp);
    let absolute_addr = *inpp as i64
        + (((((ins << MSB) as i32) >> (MSB + LSB - 2)) & !3) as i64);
    let mut new_pc_offset = (absolute_addr - *outpp as i64) >> 2;
    let special_fix_type = ctx.is_in_fixing_range(absolute_addr);

    if special_fix_type
        || ((new_pc_offset.unsigned_abs() as u64) + ((faligned + 1 - 4) / 4) as u64)
            >= ((!LMASK) >> (LSB + 1)) as u64
    {
        while ((*outpp as usize).wrapping_add(8) & faligned) != 0 {
            unsafe { **outpp = A64_NOP; }
            *outpp = unsafe { (*outpp).add(1) };
        }
        ctx.reset_current_ins(current_idx, *outpp);

        let ns = ((faligned + 1) / 4) as u32;
        unsafe {
            **outpp = (((8u32 >> 2) << LSB) & !mask) | (ins & LMASK); // LDR #0x8
            *(*outpp).add(1) = 0x14000001 + ns; // B #0xc
            core::ptr::copy_nonoverlapping(
                absolute_addr as *const u8,
                (*outpp).add(2) as *mut u8,
                faligned + 1,
            );
        }
        *outpp = unsafe { (*outpp).add(2 + ns as usize) };
    } else {
        let fa_shifted = faligned >> 2;
        while (new_pc_offset as usize & fa_shifted) != 0 {
            unsafe { **outpp = A64_NOP; }
            *outpp = unsafe { (*outpp).add(1) };
            new_pc_offset = (absolute_addr - *outpp as i64) >> 2;
        }
        ctx.reset_current_ins(current_idx, *outpp);

        unsafe {
            **outpp = ((new_pc_offset as u32) << LSB & !mask) | (ins & LMASK);
        }
        *outpp = unsafe { (*outpp).add(1) };
    }

    *inpp = unsafe { (*inpp).add(1) };
    ctx.process_fix_map(current_idx);
    true
}

unsafe fn fix_pcreladdr(
    inpp: &mut *mut u32,
    outpp: &mut *mut u32,
    ctx: &mut Context,
) -> bool {
    const MSB: u32 = 8;
    const LSB: u32 = 5;
    const MASK: u32 = 0x9f000000;
    const RMASK: u32 = 0x0000001f;
    const LMASK: u32 = 0xff00001f;
    const FMASK: u32 = 0x00ffffff;
    const MAX_VAL: u32 = 0x001fffff;
    const OP_ADR: u32 = 0x10000000;
    const OP_ADRP: u32 = 0x90000000;

    let ins = unsafe { **inpp };
    let current_idx;
    match ins & MASK {
        OP_ADR => {
            current_idx = ctx.get_and_set_current_index(*inpp, *outpp);
            let lsb_bytes = ((ins << 1) >> 30) as i64;
            let absolute_addr = *inpp as i64
                + (((((ins << MSB) as i32) >> (MSB + LSB - 2)) & !3) as i64 | lsb_bytes);
            let mut new_pc_offset = absolute_addr - *outpp as i64;
            let special_fix_type = ctx.is_in_fixing_range(absolute_addr);

            if !special_fix_type
                && (new_pc_offset.unsigned_abs() as u64) >= (MAX_VAL >> 1) as u64
            {
                if ((*outpp as usize).wrapping_add(8) & 7) != 0 {
                    unsafe { **outpp = A64_NOP; }
                    *outpp = unsafe { (*outpp).add(1) };
                    ctx.reset_current_ins(current_idx, *outpp);
                }
                unsafe {
                    **outpp =
                        0x58000000 | (((8u32 >> 2) << LSB) & !MASK) | (ins & RMASK); // LDR #0x8
                    *(*outpp).add(1) = 0x14000003; // B #0xc
                    core::ptr::copy_nonoverlapping(
                        &absolute_addr as *const i64 as *const u8,
                        (*outpp).add(2) as *mut u8,
                        8,
                    );
                }
                *outpp = unsafe { (*outpp).add(4) };
            } else {
                if special_fix_type {
                    let ref_idx = ctx.get_ref_ins_index(absolute_addr & !3);
                    if ref_idx <= current_idx {
                        new_pc_offset =
                            ctx.dat[ref_idx as usize].ins - *outpp as i64;
                    } else {
                        ctx.insert_fix_map(ref_idx, *outpp, LSB, FMASK);
                        new_pc_offset = 0;
                    }
                }
                unsafe {
                    **outpp = (((new_pc_offset as u32) << (LSB - 2)) & FMASK) | (ins & LMASK);
                }
                *outpp = unsafe { (*outpp).add(1) };
            }
        }
        OP_ADRP => {
            current_idx = ctx.get_and_set_current_index(*inpp, *outpp);
            let lsb_bytes = ((ins << 1) >> 30) as i32;
            let absolute_addr = (*inpp as i64 & !0xfff)
                + (((((ins << MSB) as i32 >> (MSB + LSB - 2)) & !3) | lsb_bytes) as i64) * (1 << 12);

            if ctx.is_in_fixing_range(absolute_addr) {
                let ref_idx = ctx.get_ref_ins_index(absolute_addr);
                if ref_idx > current_idx {
                    warn!("adrp: ref_idx must be <= current_idx");
                }
                unsafe { *(*outpp) = ins; }
                *outpp = unsafe { (*outpp).add(1) };
            } else {
                if ((*outpp as usize).wrapping_add(8) & 7) != 0 {
                    unsafe { **outpp = A64_NOP; }
                    *outpp = unsafe { (*outpp).add(1) };
                    ctx.reset_current_ins(current_idx, *outpp);
                }
                unsafe {
                    **outpp =
                        0x58000000 | (((8u32 >> 2) << LSB) & !MASK) | (ins & RMASK); // LDR #0x8
                    *(*outpp).add(1) = 0x14000003; // B #0xc
                    core::ptr::copy_nonoverlapping(
                        &absolute_addr as *const i64 as *const u8,
                        (*outpp).add(2) as *mut u8,
                        8,
                    );
                }
                *outpp = unsafe { (*outpp).add(4) };
            }
        }
        _ => return false,
    }

    ctx.process_fix_map(current_idx);
    *inpp = unsafe { (*inpp).add(1) };
    true
}

unsafe fn fix_instructions(inp: *mut u32, count: i32, outp: *mut u32) {
    let mut ctx = Context::new(inp, count);
    let mut inp = inp;
    let mut outp = outp;
    let outp_base = outp;
    let mut remaining = count;

    while remaining > 0 {
        remaining -= 1;
        if unsafe { fix_branch_imm(&mut inp, &mut outp, &mut ctx) } {
            continue;
        }
        if unsafe { fix_cond_comp_test_branch(&mut inp, &mut outp, &mut ctx) } {
            continue;
        }
        if unsafe { fix_loadlit(&mut inp, &mut outp, &mut ctx) } {
            continue;
        }
        if unsafe { fix_pcreladdr(&mut inp, &mut outp, &mut ctx) } {
            continue;
        }

        let idx = ctx.get_and_set_current_index(inp, outp);
        ctx.process_fix_map(idx);
        unsafe {
            *outp = *inp;
            outp = outp.add(1);
            inp = inp.add(1);
        }
    }

    const MASK: u64 = 0x03ffffff;
    let callback = inp as i64;
    let pc_offset = (callback - outp as i64) >> 2;
    if (pc_offset.unsigned_abs() as u64) >= (MASK >> 1) {
        if ((outp as usize).wrapping_add(8) & 7) != 0 {
            unsafe { *outp = A64_NOP; }
            outp = unsafe { outp.add(1) };
        }
        unsafe {
            *outp = 0x58000051; // LDR X17, #0x8
            *outp.add(1) = 0xd61f0220; // BR X17
            *(outp.add(2) as *mut i64) = callback;
        }
        outp = unsafe { outp.add(4) };
    } else {
        unsafe {
            *outp = 0x14000000 | ((pc_offset as u32) & MASK as u32);
        }
        outp = unsafe { outp.add(1) };
    }

    let total = unsafe { outp.offset_from(outp_base) as usize * 4 };
    unsafe { flush_cache(outp_base as *mut u8, total) };
}

#[repr(C, align(4096))]
struct InsnsPool {
    data: [[u32; A64_MAX_INSTRUCTIONS * 10]; A64_MAX_BACKUPS],
}

static mut INSNS_POOL: InsnsPool = InsnsPool {
    data: [[0u32; A64_MAX_INSTRUCTIONS * 10]; A64_MAX_BACKUPS],
};
static POOL_INDEX: AtomicI32 = AtomicI32::new(-1);
static POOL_INIT: AtomicI32 = AtomicI32::new(0);

unsafe fn ensure_pool_init() {
    if POOL_INIT.load(Ordering::Acquire) == 0 {
        unsafe {
            let pool_ptr = core::ptr::addr_of_mut!(INSNS_POOL) as *mut u8;
            let pool_size = core::mem::size_of::<InsnsPool>();
            make_rwx(pool_ptr, pool_size);
        }
        POOL_INIT.store(1, Ordering::Release);
    }
}

fn fast_allocate_trampoline() -> *mut u32 {
    let i = POOL_INDEX.fetch_add(1, Ordering::SeqCst) + 1;
    if i >= 0 && (i as usize) < A64_MAX_BACKUPS {
        unsafe { &raw mut INSNS_POOL.data[i as usize] as *mut u32 }
    } else {
        error!("failed to allocate trampoline!");
        core::ptr::null_mut()
    }
}

pub unsafe fn a64_hook_function_v(
    symbol: *mut u8,
    replace: *mut u8,
    rwx: *mut u8,
    rwx_size: usize,
) -> *mut u8 {
    const MASK: u64 = 0x03ffffff;

    let mut trampoline = rwx as *mut u32;
    let original = symbol as *mut u32;

    let pc_offset = ((replace as i64) - (symbol as i64)) >> 2;
    if (pc_offset.unsigned_abs() as u64) >= (MASK >> 1) {
        let count: i32 = if ((original as usize).wrapping_add(8) & 7) != 0 { 5 } else { 4 };
        if !trampoline.is_null() {
            if rwx_size < (count as usize) * 10 {
                error!(
                    "rwx size too small for {} backup instructions",
                    count * 10
                );
                return core::ptr::null_mut();
            }
            unsafe { fix_instructions(original, count, trampoline) };
        }

        if unsafe { make_rwx(symbol, 5 * 4) } == 0 {
            let mut orig = original;
            if count == 5 {
                unsafe { *orig = A64_NOP; }
                orig = unsafe { orig.add(1) };
            }
            unsafe {
                *orig = 0x58000051; // LDR X17, #0x8
                *orig.add(1) = 0xd61f0220; // BR X17
                *(orig.add(2) as *mut i64) = replace as i64;
                flush_cache(symbol, 5 * 4);
            }
            info!(
                "inline hook {:p}->{:p} successfully! {} bytes overwritten",
                symbol, replace, 5 * 4
            );
        } else {
            error!(
                "mprotect failed, p={:p}, size={}",
                original,
                5 * core::mem::size_of::<u32>()
            );
            trampoline = core::ptr::null_mut();
        }
    } else {
        if !trampoline.is_null() {
            if rwx_size < 10 {
                error!("rwx size too small for 10 bytes backup instructions");
                return core::ptr::null_mut();
            }
            unsafe { fix_instructions(original, 1, trampoline) };
        }

        if unsafe { make_rwx(symbol, 4) } == 0 {
            let new_ins = 0x14000000u32 | ((pc_offset as u32) & MASK as u32);
            unsafe {
                let old = core::sync::atomic::AtomicU32::from_ptr(original);
                let _ = old.compare_exchange(
                    *original,
                    new_ins,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                );
                flush_cache(symbol, 4);
            }
            info!(
                "inline hook {:p}->{:p} successfully! 4 bytes overwritten",
                symbol, replace
            );
        } else {
            error!(
                "mprotect failed, p={:p}, size={}",
                original,
                core::mem::size_of::<u32>()
            );
            trampoline = core::ptr::null_mut();
        }
    }

    trampoline as *mut u8
}

pub unsafe fn hook_function_aarch64(
    symbol: *mut u8,
    replace: *mut u8,
    result: *mut *mut u8,
) -> Result<usize> {
    if symbol.is_null() {
        return Err(SubstrateError::NullSymbol);
    }

    unsafe { ensure_pool_init() };

    let mut trampoline: *mut u8 = core::ptr::null_mut();
    if !result.is_null() {
        trampoline = fast_allocate_trampoline() as *mut u8;
        unsafe { *result = trampoline; }
        if trampoline.is_null() {
            return Err(SubstrateError::MmapFailed(0));
        }
    }

    unsafe { make_rwx(symbol, 5 * core::mem::size_of::<usize>()) };

    let t = unsafe {
        a64_hook_function_v(
            symbol,
            replace,
            trampoline,
            A64_MAX_INSTRUCTIONS * 10,
        )
    };
    if t.is_null() && !result.is_null() {
        unsafe { *result = core::ptr::null_mut() };
        return Err(SubstrateError::MprotectFailed(0));
    }

    Ok(5 * 4)
}
