use crate::buffer::CodeWriter;

#[cfg(target_pointer_width = "64")]
pub const IA32: bool = false;
#[cfg(target_pointer_width = "32")]
pub const IA32: bool = true;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X86Reg {
    Rax = 0, Rcx, Rdx, Rbx,
    Rsp, Rbp, Rsi, Rdi,
    R8, R9, R10, R11,
    R12, R13, R14, R15,
}

pub fn is_32bit_offset(target: usize, source: usize) -> bool {
    let offset = target.wrapping_sub(source) as isize;
    offset as i32 as isize == offset
}

pub fn size_of_skip() -> usize {
    5
}

pub fn size_of_push_pointer_val(target: usize) -> usize {
    if (target as u64) >> 32 == 0 { 5 } else { 13 }
}

pub fn size_of_push_pointer(target: *const u8) -> usize {
    size_of_push_pointer_val(target as usize)
}

pub fn size_of_jump_blind(target: usize) -> usize {
    if IA32 {
        size_of_skip()
    } else {
        size_of_push_pointer_val(target) + 1
    }
}

pub fn size_of_jump_from(target: usize, source: usize) -> usize {
    if IA32 || is_32bit_offset(target, source + 5) {
        size_of_skip()
    } else {
        size_of_push_pointer_val(target) + 1
    }
}

pub fn size_of_pop(target: u8) -> usize {
    if target >> 3 != 0 { 2 } else { 1 }
}

pub fn size_of_move64() -> usize {
    3
}

pub unsafe fn write_skip(w: &mut CodeWriter, size: isize) {
    unsafe {
        w.write_u8(0xe9);
        w.write_i32(size as i32);
    }
}

pub unsafe fn push_pointer(w: &mut CodeWriter, target: usize) {
    unsafe {
        w.write_u8(0x68);
        w.write_u32(target as u32);

        let high = ((target as u64) >> 32) as u32;
        if high != 0 {
            w.write_u8(0xc7);
            w.write_u8(0x44);
            w.write_u8(0x24);
            w.write_u8(0x04);
            w.write_u32(high);
        }
    }
}

pub unsafe fn write_call_reg(w: &mut CodeWriter, target: X86Reg) {
    unsafe {
        let t = target as u8;
        if t >> 3 != 0 {
            w.write_u8(0x40 | (t & 0x08) >> 3);
        }
        w.write_u8(0xff);
        w.write_u8(0xd0 | (t & 0x07));
    }
}

pub unsafe fn write_call(w: &mut CodeWriter, target: usize) {
    let source = w.address();
    unsafe {
        if IA32 || is_32bit_offset(target, source + 5) {
            w.write_u8(0xe8);
            w.write_i32((target.wrapping_sub(source + 5)) as i32);
        } else {
            push_pointer(w, target);
            w.write_u8(0x83);
            w.write_u8(0xc4);
            w.write_u8(0x08);
            w.write_u8(0x67);
            w.write_u8(0xff);
            w.write_u8(0x54);
            w.write_u8(0x24);
            w.write_u8(0xf8);
        }
    }
}

pub unsafe fn write_jump(w: &mut CodeWriter, target: usize) {
    let source = w.address();
    unsafe {
        if IA32 || is_32bit_offset(target, source + 5) {
            write_skip(w, target.wrapping_sub(source + 5) as isize);
        } else {
            push_pointer(w, target);
            w.write_u8(0xc3);
        }
    }
}

pub unsafe fn write_jump_reg(w: &mut CodeWriter, target: X86Reg) {
    unsafe {
        let t = target as u8;
        if t >> 3 != 0 {
            w.write_u8(0x40 | (t & 0x08) >> 3);
        }
        w.write_u8(0xff);
        w.write_u8(0xe0 | (t & 0x07));
    }
}

pub unsafe fn write_pop(w: &mut CodeWriter, target: u8) {
    unsafe {
        if target >> 3 != 0 {
            w.write_u8(0x40 | (target & 0x08) >> 3);
        }
        w.write_u8(0x58 | (target & 0x07));
    }
}

pub unsafe fn write_push(w: &mut CodeWriter, target: X86Reg) {
    unsafe {
        let t = target as u8;
        if t >> 3 != 0 {
            w.write_u8(0x40 | (t & 0x08) >> 3);
        }
        w.write_u8(0x50 | (t & 0x07));
    }
}

pub unsafe fn write_add(w: &mut CodeWriter, target: X86Reg, source: u8) {
    unsafe {
        w.write_u8(0x83);
        w.write_u8(0xc4 | (target as u8) & 0x07);
        w.write_u8(source);
    }
}

pub unsafe fn write_set64(w: &mut CodeWriter, target: X86Reg, source: usize) {
    unsafe {
        let t = target as u8;
        w.write_u8(0x48 | ((t & 0x08) >> 3) << 2);
        w.write_u8(0xb8 | (t & 0x7));
        w.write_u64(source as u64);
    }
}

pub unsafe fn write_move64(w: &mut CodeWriter, source: u8, target: u8) {
    unsafe {
        w.write_u8(0x48 | ((target & 0x08) >> 3) << 2 | ((source & 0x08) >> 3));
        w.write_u8(0x8b);
        w.write_u8(((target & 0x07) << 3) | (source & 0x07));
    }
}
