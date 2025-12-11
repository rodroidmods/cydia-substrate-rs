use std::ffi::c_void;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmRegister {
    R0 = 0, R1, R2, R3, R4, R5, R6, R7,
    R8, R9, R10, R11, R12, R13, R14, R15,
}

pub const ARM_SP: ArmRegister = ArmRegister::R13;
pub const ARM_LR: ArmRegister = ArmRegister::R14;
pub const ARM_PC: ArmRegister = ArmRegister::R15;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmCondition {
    EQ = 0, NE, CS, CC, MI, PL, VS, VC,
    HI, LS, GE, LT, GT, LE, AL,
}

pub const ARM_HS: ArmCondition = ArmCondition::CS;
pub const ARM_LO: ArmCondition = ArmCondition::CC;

pub struct ArmInstruction;

impl ArmInstruction {
    pub fn mrs_rm_cpsr(rd: ArmRegister) -> u32 {
        0xe10f0000 | ((rd as u32) << 12)
    }

    pub fn msr_cpsr_f_rm(rm: ArmRegister) -> u32 {
        0xe128f000 | (rm as u32)
    }

    pub fn ldr_rd_rn_imm(rd: ArmRegister, rn: ArmRegister, imm: i32) -> u32 {
        let u_bit = if imm >= 0 { 1u32 << 23 } else { 0 };
        0xe5100000 | u_bit | ((rn as u32) << 16) | ((rd as u32) << 12) | imm.abs() as u32
    }

    pub fn str_rd_rn_imm(rd: ArmRegister, rn: ArmRegister, imm: i32) -> u32 {
        let u_bit = if imm >= 0 { 1u32 << 23 } else { 0 };
        0xe5000000 | u_bit | ((rn as u32) << 16) | ((rd as u32) << 12) | imm.abs() as u32
    }

    pub fn sub_rd_rn_imm(rd: ArmRegister, rn: ArmRegister, imm: u8) -> u32 {
        0xe2400000 | ((rn as u32) << 16) | ((rd as u32) << 12) | (imm as u32 & 0xff)
    }

    pub fn blx_rm(rm: ArmRegister) -> u32 {
        0xe12fff30 | (rm as u32)
    }

    pub fn mov_rd_rm(rd: ArmRegister, rm: ArmRegister) -> u32 {
        0xe1a00000 | ((rd as u32) << 12) | (rm as u32)
    }

    pub fn ldmia_sp_regs(regs: u16) -> u32 {
        0xe8b00000 | ((ARM_SP as u32) << 16) | (regs as u32)
    }

    pub fn stmdb_sp_regs(regs: u16) -> u32 {
        0xe9200000 | ((ARM_SP as u32) << 16) | (regs as u32)
    }

    pub const STMIA_SP_R0: u32 = 0xe8ad0001;
    pub const BX_R0: u32 = 0xe12fff10;
}

pub struct ThumbInstruction;

impl ThumbInstruction {
    pub const POP_R0: u16 = 0xbc01;
    pub const NOP: u16 = 0x46c0;

    pub fn b(imm: i8) -> u16 {
        0xde00 | (imm as u16 & 0xff)
    }

    pub fn blx(rm: u8) -> u16 {
        0x4780 | ((rm as u16) << 3)
    }

    pub fn bx(rm: u8) -> u16 {
        0x4700 | ((rm as u16) << 3)
    }

    pub fn add_rd_rm(rd: u8, rm: u8) -> u16 {
        0x4400 | (((rd & 0x8) as u16) >> 3 << 7) |
                 (((rm & 0x8) as u16) >> 3 << 6) |
                 (((rm & 0x7) as u16) << 3) |
                 ((rd & 0x7) as u16)
    }

    pub fn push_regs(regs: u16) -> u16 {
        let lr_bit = (regs & (1 << 14)) >> 14 << 8;
        0xb400 | lr_bit | (regs & 0xff)
    }

    pub fn pop_regs(regs: u16) -> u16 {
        let pc_bit = (regs & (1 << 15)) >> 15 << 8;
        0xbc00 | pc_bit | (regs & 0xff)
    }

    pub fn mov_rd_rm(rd: u8, rm: u8) -> u16 {
        0x4600 | (((rd & 0x8) as u16) >> 3 << 7) |
                 (((rm & 0x8) as u16) >> 3 << 6) |
                 (((rm & 0x7) as u16) << 3) |
                 ((rd & 0x7) as u16)
    }

    pub fn ldr_rd_rn_imm(rd: u8, rn: u8, imm: u8) -> u16 {
        0x6800 | (((imm & 0x1f) as u16) << 6) | ((rn as u16) << 3) | (rd as u16)
    }

    pub fn ldr_rd_pc_imm(rd: u8, imm: u8) -> u16 {
        0x4800 | ((rd as u16) << 8) | (imm as u16 & 0xff)
    }

    pub fn cmp_rn_imm(rn: u8, imm: u8) -> u16 {
        0x2000 | ((rn as u16) << 8) | (imm as u16 & 0xff)
    }

    pub fn is_32bit(instr: u16) -> bool {
        (instr & 0xe000) == 0xe000 && (instr & 0x1800) != 0x0000
    }
}

pub fn get_instruction_width_arm(_start: *const c_void) -> usize {
    4
}

pub fn get_instruction_width_thumb(start: *const c_void) -> usize {
    let thumb = start as *const u16;
    if ThumbInstruction::is_32bit(unsafe { *thumb }) {
        4
    } else {
        2
    }
}

pub fn get_instruction_width(start: *const c_void) -> usize {
    let addr = start as usize;
    if (addr & 0x1) == 0 {
        get_instruction_width_arm(start)
    } else {
        get_instruction_width_thumb((addr & !0x1) as *const c_void)
    }
}
