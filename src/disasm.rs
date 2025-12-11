use std::ffi::c_void;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use crate::sys::hde64s;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
extern "C" {
    pub fn hde64_disasm(code: *const c_void, hs: *mut hde64s) -> u32;
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub struct Disassembler;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
impl Disassembler {
    pub fn disassemble(code: *const u8) -> Option<DisassembledInstruction> {
        if code.is_null() {
            return None;
        }

        unsafe {
            let mut hs: hde64s = std::mem::zeroed();
            let len = hde64_disasm(code as *const c_void, &mut hs);

            if len == 0 || len > 15 {
                None
            } else {
                Some(DisassembledInstruction {
                    length: len as usize,
                    data: hs,
                })
            }
        }
    }

    pub fn instruction_length(code: *const u8) -> usize {
        Self::disassemble(code)
            .map(|i| i.length)
            .unwrap_or(0)
    }

    pub fn copy_instructions(from: *const u8, min_size: usize) -> Vec<u8> {
        let mut copied = Vec::new();
        let mut offset = 0;

        while offset < min_size {
            if let Some(instr) = Self::disassemble(unsafe { from.add(offset) }) {
                let slice = unsafe {
                    std::slice::from_raw_parts(from.add(offset), instr.length)
                };
                copied.extend_from_slice(slice);
                offset += instr.length;
            } else {
                break;
            }
        }

        copied
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub struct DisassembledInstruction {
    pub length: usize,
    pub data: hde64s,
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
impl DisassembledInstruction {
    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn opcode(&self) -> u8 {
        self.data.opcode
    }

    pub fn opcode2(&self) -> u8 {
        self.data.opcode2
    }

    pub fn modrm(&self) -> u8 {
        self.data.modrm
    }

    pub fn modrm_mod(&self) -> u8 {
        self.data.modrm_mod
    }

    pub fn modrm_reg(&self) -> u8 {
        self.data.modrm_reg
    }

    pub fn modrm_rm(&self) -> u8 {
        self.data.modrm_rm
    }

    pub fn has_modrm(&self) -> bool {
        (self.data.flags & 0x00000001) != 0
    }

    pub fn has_sib(&self) -> bool {
        (self.data.flags & 0x00000002) != 0
    }

    pub fn has_imm8(&self) -> bool {
        (self.data.flags & 0x00000004) != 0
    }

    pub fn has_imm16(&self) -> bool {
        (self.data.flags & 0x00000008) != 0
    }

    pub fn has_imm32(&self) -> bool {
        (self.data.flags & 0x00000010) != 0
    }

    pub fn has_imm64(&self) -> bool {
        (self.data.flags & 0x00000020) != 0
    }

    pub fn has_disp8(&self) -> bool {
        (self.data.flags & 0x00000040) != 0
    }

    pub fn has_disp16(&self) -> bool {
        (self.data.flags & 0x00000080) != 0
    }

    pub fn has_disp32(&self) -> bool {
        (self.data.flags & 0x00000100) != 0
    }

    pub fn is_relative(&self) -> bool {
        (self.data.flags & 0x00000200) != 0
    }

    pub fn has_error(&self) -> bool {
        (self.data.flags & 0x00001000) != 0
    }

    pub fn is_rip_relative(&self) -> bool {
        self.has_modrm() && (self.data.modrm & 0xc7) == 0x05
    }
}
