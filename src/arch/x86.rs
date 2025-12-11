use std::ffi::c_void;

#[cfg(target_pointer_width = "64")]
pub const IS_32BIT: bool = false;
#[cfg(target_pointer_width = "32")]
pub const IS_32BIT: bool = true;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X86Register {
    RAX = 0, RCX, RDX, RBX, RSP, RBP, RSI, RDI,
    R8, R9, R10, R11, R12, R13, R14, R15,
}

pub struct X86Instruction;

impl X86Instruction {
    pub fn is_32bit_offset(target: usize, source: usize) -> bool {
        let offset = target.wrapping_sub(source) as isize;
        offset as i32 as isize == offset
    }

    pub fn size_of_skip() -> usize {
        5
    }

    pub fn size_of_push_pointer(target: usize) -> usize {
        if (target >> 32) == 0 {
            5
        } else {
            13
        }
    }

    pub fn size_of_jump_blind(target: usize) -> usize {
        if IS_32BIT {
            Self::size_of_skip()
        } else {
            Self::size_of_push_pointer(target) + 1
        }
    }

    pub fn size_of_jump(target: usize, source: usize) -> usize {
        if IS_32BIT || Self::is_32bit_offset(target, source + 5) {
            Self::size_of_skip()
        } else {
            Self::size_of_push_pointer(target) + 1
        }
    }

    pub fn write_skip(buffer: &mut [u8], offset: usize, size: i32) {
        buffer[offset] = 0xe9;
        buffer[offset + 1..offset + 5].copy_from_slice(&size.to_le_bytes());
    }

    pub fn write_push_pointer(buffer: &mut [u8], offset: usize, target: usize) -> usize {
        let mut pos = offset;
        buffer[pos] = 0x68;
        pos += 1;
        buffer[pos..pos + 4].copy_from_slice(&(target as u32).to_le_bytes());
        pos += 4;

        let high = (target >> 32) as u32;
        if high != 0 {
            buffer[pos] = 0xc7;
            buffer[pos + 1] = 0x44;
            buffer[pos + 2] = 0x24;
            buffer[pos + 3] = 0x04;
            pos += 4;
            buffer[pos..pos + 4].copy_from_slice(&high.to_le_bytes());
            pos += 4;
        }
        pos - offset
    }

    pub fn write_call_register(buffer: &mut [u8], offset: usize, target: X86Register) -> usize {
        let mut pos = offset;
        let reg = target as u8;
        if (reg >> 3) != 0 {
            buffer[pos] = 0x40 | ((reg & 0x08) >> 3);
            pos += 1;
        }
        buffer[pos] = 0xff;
        buffer[pos + 1] = 0xd0 | (reg & 0x07);
        pos + 2 - offset
    }

    pub fn write_call_address(buffer: &mut [u8], offset: usize, target: usize, source: usize) -> usize {
        if IS_32BIT || Self::is_32bit_offset(target, source + 5) {
            buffer[offset] = 0xe8;
            let rel = (target.wrapping_sub(source + 5)) as i32;
            buffer[offset + 1..offset + 5].copy_from_slice(&rel.to_le_bytes());
            5
        } else {
            let mut pos = offset;
            pos += Self::write_push_pointer(buffer, pos, target);
            buffer[pos..pos + 3].copy_from_slice(&[0x83, 0xc4, 0x08]);
            pos += 3;
            buffer[pos..pos + 5].copy_from_slice(&[0x67, 0xff, 0x54, 0x24, 0xf8]);
            pos + 5 - offset
        }
    }

    pub fn write_jump_address(buffer: &mut [u8], offset: usize, target: usize, source: usize) -> usize {
        if IS_32BIT || Self::is_32bit_offset(target, source + 5) {
            Self::write_skip(buffer, offset, (target.wrapping_sub(source + 5)) as i32);
            5
        } else {
            let mut pos = offset;
            pos += Self::write_push_pointer(buffer, pos, target);
            buffer[pos] = 0xc3;
            pos + 1 - offset
        }
    }

    pub fn write_jump_register(buffer: &mut [u8], offset: usize, target: X86Register) -> usize {
        let mut pos = offset;
        let reg = target as u8;
        if (reg >> 3) != 0 {
            buffer[pos] = 0x40 | ((reg & 0x08) >> 3);
            pos += 1;
        }
        buffer[pos] = 0xff;
        buffer[pos + 1] = 0xe0 | (reg & 0x07);
        pos + 2 - offset
    }

    pub fn write_pop(buffer: &mut [u8], offset: usize, target: u8) -> usize {
        let mut pos = offset;
        if (target >> 3) != 0 {
            buffer[pos] = 0x40 | ((target & 0x08) >> 3);
            pos += 1;
        }
        buffer[pos] = 0x58 | (target & 0x07);
        pos + 1 - offset
    }

    pub fn size_of_pop(target: u8) -> usize {
        if (target >> 3) != 0 { 2 } else { 1 }
    }

    pub fn write_push(buffer: &mut [u8], offset: usize, target: X86Register) -> usize {
        let mut pos = offset;
        let reg = target as u8;
        if (reg >> 3) != 0 {
            buffer[pos] = 0x40 | ((reg & 0x08) >> 3);
            pos += 1;
        }
        buffer[pos] = 0x50 | (reg & 0x07);
        pos + 1 - offset
    }

    pub fn write_add(buffer: &mut [u8], offset: usize, target: X86Register, value: u8) -> usize {
        buffer[offset] = 0x83;
        buffer[offset + 1] = 0xc4 | ((target as u8) & 0x07);
        buffer[offset + 2] = value;
        3
    }

    pub fn write_set64(buffer: &mut [u8], offset: usize, target: X86Register, value: u64) -> usize {
        let reg = target as u8;
        buffer[offset] = 0x48 | (((reg & 0x08) >> 3) << 2);
        buffer[offset + 1] = 0xb8 | (reg & 0x7);
        buffer[offset + 2..offset + 10].copy_from_slice(&value.to_le_bytes());
        10
    }

    pub fn write_move64(buffer: &mut [u8], offset: usize, source: u8, target: u8) -> usize {
        buffer[offset] = 0x48 | (((target & 0x08) >> 3) << 2) | ((source & 0x08) >> 3);
        buffer[offset + 1] = 0x8b;
        buffer[offset + 2] = ((target & 0x07) << 3) | (source & 0x07);
        3
    }

    pub fn size_of_move64() -> usize {
        3
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub fn get_instruction_width(start: *const c_void) -> usize {
    #[cfg(feature = "disassembler")]
    {
        use crate::disasm::hde64_disasm;
        unsafe {
            let mut hs = std::mem::zeroed();
            hde64_disasm(start, &mut hs) as usize
        }
    }
    #[cfg(not(feature = "disassembler"))]
    {
        1
    }
}
