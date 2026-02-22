#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmReg {
    R0 = 0, R1, R2, R3,
    R4, R5, R6, R7,
    R8, R9, R10, R11,
    R12, R13, R14, R15,
}

impl ArmReg {
    pub const SP: Self = Self::R13;
    pub const LR: Self = Self::R14;
    pub const PC: Self = Self::R15;
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmCond {
    Eq = 0, Ne, Cs, Cc,
    Mi, Pl, Vs, Vc,
    Hi, Ls, Ge, Lt,
    Gt, Le, Al,
}

impl ArmCond {
    pub const HS: Self = Self::Cs;
    pub const LO: Self = Self::Cc;
}

pub fn a_mrs_rm_cpsr(rd: u8) -> u32 {
    0xe10f0000 | ((rd as u32) << 12)
}

pub fn a_msr_cpsr_f_rm(rm: u8) -> u32 {
    0xe128f000 | (rm as u32)
}

pub fn a_ldr_rd_rn_im(rd: u8, rn: u8, im: i32) -> u32 {
    let u_bit = if im >= 0 { 1u32 << 23 } else { 0 };
    0xe5100000 | u_bit | ((rn as u32) << 16) | ((rd as u32) << 12) | (im.unsigned_abs() & 0xfff)
}

pub fn a_str_rd_rn_im(rd: u8, rn: u8, im: i32) -> u32 {
    let u_bit = if im >= 0 { 1u32 << 23 } else { 0 };
    0xe5000000 | u_bit | ((rn as u32) << 16) | ((rd as u32) << 12) | (im.unsigned_abs() & 0xfff)
}

pub fn a_sub_rd_rn_im(rd: u8, rn: u8, im: u32) -> u32 {
    0xe2400000 | ((rn as u32) << 16) | ((rd as u32) << 12) | (im & 0xff)
}

pub fn a_blx_rm(rm: u8) -> u32 {
    0xe12fff30 | (rm as u32)
}

pub fn a_mov_rd_rm(rd: u8, rm: u8) -> u32 {
    0xe1a00000 | ((rd as u32) << 12) | (rm as u32)
}

pub fn a_ldmia_sp_rs(rs: u32) -> u32 {
    0xe8b00000 | ((ArmReg::SP as u32) << 16) | rs
}

pub fn a_stmdb_sp_rs(rs: u32) -> u32 {
    0xe9200000 | ((ArmReg::SP as u32) << 16) | rs
}

pub const A_STMIA_SP_R0: u32 = 0xe8ad0001;
pub const A_BX_R0: u32 = 0xe12fff10;

pub fn a_add(rd: u8, rn: u8, im: u32) -> u32 {
    0xe2800000 | ((rn as u32) << 16) | ((rd as u32) << 12) | (im & 0xff)
}

pub fn a_pcrel_r(ic: u32) -> bool {
    (ic & 0x0c000000) == 0x04000000
        && (ic & 0xf0000000) != 0xf0000000
        && (ic & 0x000f0000) == 0x000f0000
}
