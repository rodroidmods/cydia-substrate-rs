const _C_NONE: u8 = 0x00;
const C_MODRM: u8 = 0x01;
const C_IMM8: u8 = 0x02;
const C_IMM16: u8 = 0x04;
const C_IMM_P66: u8 = 0x10;
const C_REL8: u8 = 0x20;
const C_REL32: u8 = 0x40;
const C_GROUP: u8 = 0x80;
const C_ERROR: u8 = 0xff;

const _PRE_ANY: u8 = 0x00;
const PRE_NONE: u8 = 0x01;
const PRE_F2: u8 = 0x02;
const PRE_F3: u8 = 0x04;
const PRE_66: u8 = 0x08;
const PRE_67: u8 = 0x10;
const PRE_LOCK: u8 = 0x20;
const PRE_SEG: u8 = 0x40;

const DELTA_OPCODES: usize = 0x4a;
const DELTA_FPU_REG: usize = 0xfd;
const DELTA_FPU_MODRM: usize = 0x104;
const DELTA_PREFIXES: usize = 0x13c;
const DELTA_OP_LOCK_OK: usize = 0x1ae;
const DELTA_OP2_LOCK_OK: usize = 0x1c6;
const DELTA_OP_ONLY_MEM: usize = 0x1d8;
const DELTA_OP2_ONLY_MEM: usize = 0x1e7;

pub const F_MODRM: u32 = 0x00000001;
pub const F_SIB: u32 = 0x00000002;
pub const F_IMM8: u32 = 0x00000004;
pub const F_IMM16: u32 = 0x00000008;
pub const F_IMM32: u32 = 0x00000010;
pub const F_IMM64: u32 = 0x00000020;
pub const F_DISP8: u32 = 0x00000040;
pub const F_DISP16: u32 = 0x00000080;
pub const F_DISP32: u32 = 0x00000100;
pub const F_RELATIVE: u32 = 0x00000200;
pub const F_ERROR: u32 = 0x00001000;
pub const F_ERROR_OPCODE: u32 = 0x00002000;
pub const F_ERROR_LENGTH: u32 = 0x00004000;
pub const F_ERROR_LOCK: u32 = 0x00008000;
pub const F_ERROR_OPERAND: u32 = 0x00010000;
pub const F_PREFIX_REPNZ: u32 = 0x01000000;
pub const F_PREFIX_REPX: u32 = 0x02000000;
pub const F_PREFIX_REP: u32 = 0x03000000;
pub const F_PREFIX_66: u32 = 0x04000000;
pub const F_PREFIX_67: u32 = 0x08000000;
pub const F_PREFIX_LOCK: u32 = 0x10000000;
pub const F_PREFIX_SEG: u32 = 0x20000000;
pub const F_PREFIX_REX: u32 = 0x40000000;

#[repr(C)]
#[derive(Clone, Default)]
pub struct Hde64 {
    pub len: u8,
    pub p_rep: u8,
    pub p_lock: u8,
    pub p_seg: u8,
    pub p_66: u8,
    pub p_67: u8,
    pub rex: u8,
    pub rex_w: u8,
    pub rex_r: u8,
    pub rex_x: u8,
    pub rex_b: u8,
    pub opcode: u8,
    pub opcode2: u8,
    pub modrm: u8,
    pub modrm_mod: u8,
    pub modrm_reg: u8,
    pub modrm_rm: u8,
    pub sib: u8,
    pub sib_scale: u8,
    pub sib_index: u8,
    pub sib_base: u8,
    pub imm: Imm64,
    pub disp: Disp64,
    pub flags: u32,
}

impl core::fmt::Debug for Hde64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Hde64")
            .field("len", &self.len)
            .field("opcode", &self.opcode)
            .field("opcode2", &self.opcode2)
            .field("modrm", &self.modrm)
            .field("flags", &self.flags)
            .finish()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Imm64 {
    pub imm64: u64,
}

impl Imm64 {
    pub fn imm8(&self) -> u8 {
        self.imm64 as u8
    }
    pub fn imm16(&self) -> u16 {
        self.imm64 as u16
    }
    pub fn imm32(&self) -> u32 {
        self.imm64 as u32
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union Disp64 {
    pub disp8: u8,
    pub disp16: u16,
    pub disp32: u32,
}

impl Default for Disp64 {
    fn default() -> Self {
        Self { disp32: 0 }
    }
}

#[rustfmt::skip]
static HDE64_TABLE: [u8; 529] = [
    0xa5,0xaa,0xa5,0xb8,0xa5,0xaa,0xa5,0xaa,0xa5,0xb8,0xa5,0xb8,0xa5,0xb8,0xa5,
    0xb8,0xc0,0xc0,0xc0,0xc0,0xc0,0xc0,0xc0,0xc0,0xac,0xc0,0xcc,0xc0,0xa1,0xa1,
    0xa1,0xa1,0xb1,0xa5,0xa5,0xa6,0xc0,0xc0,0xd7,0xda,0xe0,0xc0,0xe4,0xc0,0xea,
    0xea,0xe0,0xe0,0x98,0xc8,0xee,0xf1,0xa5,0xd3,0xa5,0xa5,0xa1,0xea,0x9e,0xc0,
    0xc0,0xc2,0xc0,0xe6,0x03,0x7f,0x11,0x7f,0x01,0x7f,0x01,0x3f,0x01,0x01,0xab,
    0x8b,0x90,0x64,0x5b,0x5b,0x5b,0x5b,0x5b,0x92,0x5b,0x5b,0x76,0x90,0x92,0x92,
    0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x6a,0x73,0x90,
    0x5b,0x52,0x52,0x52,0x52,0x5b,0x5b,0x5b,0x5b,0x77,0x7c,0x77,0x85,0x5b,0x5b,
    0x70,0x5b,0x7a,0xaf,0x76,0x76,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,
    0x5b,0x5b,0x86,0x01,0x03,0x01,0x04,0x03,0xd5,0x03,0xd5,0x03,0xcc,0x01,0xbc,
    0x03,0xf0,0x03,0x03,0x04,0x00,0x50,0x50,0x50,0x50,0xff,0x20,0x20,0x20,0x20,
    0x01,0x01,0x01,0x01,0xc4,0x02,0x10,0xff,0xff,0xff,0x01,0x00,0x03,0x11,0xff,
    0x03,0xc4,0xc6,0xc8,0x02,0x10,0x00,0xff,0xcc,0x01,0x01,0x01,0x00,0x00,0x00,
    0x00,0x01,0x01,0x03,0x01,0xff,0xff,0xc0,0xc2,0x10,0x11,0x02,0x03,0x01,0x01,
    0x01,0xff,0xff,0xff,0x00,0x00,0x00,0xff,0x00,0x00,0xff,0xff,0xff,0xff,0x10,
    0x10,0x10,0x10,0x02,0x10,0x00,0x00,0xc6,0xc8,0x02,0x02,0x02,0x02,0x06,0x00,
    0x04,0x00,0x02,0xff,0x00,0xc0,0xc2,0x01,0x01,0x03,0x03,0x03,0xca,0x40,0x00,
    0x0a,0x00,0x04,0x00,0x00,0x00,0x00,0x7f,0x00,0x33,0x01,0x00,0x00,0x00,0x00,
    0x00,0x00,0xff,0xbf,0xff,0xff,0x00,0x00,0x00,0x00,0x07,0x00,0x00,0xff,0x00,
    0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xff,0xff,
    0x00,0x00,0x00,0xbf,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x7f,0x00,0x00,
    0xff,0x40,0x40,0x40,0x40,0x41,0x49,0x40,0x40,0x40,0x40,0x4c,0x42,0x40,0x40,
    0x40,0x40,0x40,0x40,0x40,0x40,0x4f,0x44,0x53,0x40,0x40,0x40,0x44,0x57,0x43,
    0x5c,0x40,0x60,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,
    0x40,0x40,0x64,0x66,0x6e,0x6b,0x40,0x40,0x6a,0x46,0x40,0x40,0x44,0x46,0x40,
    0x40,0x5b,0x44,0x40,0x40,0x00,0x00,0x00,0x00,0x06,0x06,0x06,0x06,0x01,0x06,
    0x06,0x02,0x06,0x06,0x00,0x06,0x00,0x0a,0x0a,0x00,0x00,0x00,0x02,0x07,0x07,
    0x06,0x02,0x0d,0x06,0x06,0x06,0x0e,0x05,0x05,0x02,0x02,0x00,0x00,0x04,0x04,
    0x04,0x04,0x05,0x06,0x06,0x06,0x00,0x00,0x00,0x0e,0x00,0x00,0x08,0x00,0x10,
    0x00,0x18,0x00,0x20,0x00,0x28,0x00,0x30,0x00,0x80,0x01,0x82,0x01,0x86,0x00,
    0xf6,0xcf,0xfe,0x3f,0xab,0x00,0xb0,0x00,0xb1,0x00,0xb3,0x00,0xba,0xf8,0xbb,
    0x00,0xc0,0x00,0xc1,0x00,0xc7,0xbf,0x62,0xff,0x00,0x8d,0xff,0x00,0xc4,0xff,
    0x00,0xc5,0xff,0x00,0xff,0xff,0xeb,0x01,0xff,0x0e,0x12,0x08,0x00,0x13,0x09,
    0x00,0x16,0x08,0x00,0x17,0x09,0x00,0x2b,0x09,0x00,0xae,0xff,0x07,0xb2,0xff,
    0x00,0xb4,0xff,0x00,0xb5,0xff,0x00,0xc3,0x01,0x00,0xc7,0xff,0xbf,0xe7,0x08,
    0x00,0xf0,0x02,0x00,
];

pub unsafe fn hde64_disasm(code: *const u8, hs: &mut Hde64) -> u32 {
    *hs = Hde64::default();

    let mut p = code;
    let mut pref: u8 = 0;
    let mut op64: u8 = 0;
    let ht_base = HDE64_TABLE.as_ptr();
    let mut ht = ht_base;
    let mut disp_size: u8 = 0;

    unsafe {

    let mut c;
    for _ in 0..16u8 {
        c = *p;
        p = p.add(1);
        match c {
            0xf3 => {
                hs.p_rep = c;
                pref |= PRE_F3;
            }
            0xf2 => {
                hs.p_rep = c;
                pref |= PRE_F2;
            }
            0xf0 => {
                hs.p_lock = c;
                pref |= PRE_LOCK;
            }
            0x26 | 0x2e | 0x36 | 0x3e | 0x64 | 0x65 => {
                hs.p_seg = c;
                pref |= PRE_SEG;
            }
            0x66 => {
                hs.p_66 = c;
                pref |= PRE_66;
            }
            0x67 => {
                hs.p_67 = c;
                pref |= PRE_67;
            }
            _ => break,
        }
    }

    c = *p.sub(1);

    hs.flags = (pref as u32) << 23;

    if pref == 0 {
        pref |= PRE_NONE;
    }

    if (c & 0xf0) == 0x40 {
        hs.flags |= F_PREFIX_REX;
        hs.rex_w = (c & 0xf) >> 3;
        if hs.rex_w != 0 && (*p & 0xf8) == 0xb8 {
            op64 += 1;
        }
        hs.rex_r = (c & 7) >> 2;
        hs.rex_x = (c & 3) >> 1;
        hs.rex_b = c & 1;
        c = *p;
        p = p.add(1);
        if (c & 0xf0) == 0x40 {
            hs.flags |= F_ERROR | F_ERROR_OPCODE;
            hs.len = (p as usize - code as usize) as u8;
            if hs.len > 15 {
                hs.flags |= F_ERROR_LENGTH;
                hs.len = 15;
            }
            return hs.len as u32;
        }
    }

    hs.opcode = c;
    if c == 0x0f {
        c = *p;
        p = p.add(1);
        hs.opcode2 = c;
        ht = ht.add(DELTA_OPCODES);
    } else if c >= 0xa0 && c <= 0xa3 {
        op64 += 1;
        if pref & PRE_67 != 0 {
            pref |= PRE_66;
        } else {
            pref &= !PRE_66;
        }
    }

    let opcode = c;
    let idx1 = *ht.add((opcode / 4) as usize);
    let mut cflags = *ht.add(idx1 as usize + (opcode % 4) as usize);

    if cflags == C_ERROR {
        hs.flags |= F_ERROR | F_ERROR_OPCODE;
        cflags = 0;
        if (opcode & 0xfd) == 0x24 {
            cflags = cflags.wrapping_add(1);
        }
    }

    let mut x: u8 = 0;
    if cflags & C_GROUP != 0 {
        let t_ptr = ht.add((cflags & 0x7f) as usize) as *const u16;
        let t = *t_ptr;
        cflags = t as u8;
        x = (t >> 8) as u8;
    }

    if hs.opcode2 != 0 {
        let ht2 = ht_base.add(DELTA_PREFIXES);
        let idx2 = *ht2.add((opcode / 4) as usize);
        if *ht2.add(idx2 as usize + (opcode % 4) as usize) & pref != 0 {
            hs.flags |= F_ERROR | F_ERROR_OPCODE;
        }
    }

    let mut m_mod: u8;
    let m_rm: u8;
    let m_reg: u8;

    if cflags & C_MODRM != 0 {
        hs.flags |= F_MODRM;
        c = *p;
        p = p.add(1);
        hs.modrm = c;
        m_mod = c >> 6;
        hs.modrm_mod = m_mod;
        m_rm = c & 7;
        hs.modrm_rm = m_rm;
        m_reg = (c & 0x3f) >> 3;
        hs.modrm_reg = m_reg;

        if x != 0 && ((x << m_reg) & 0x80) != 0 {
            hs.flags |= F_ERROR | F_ERROR_OPCODE;
        }

        if hs.opcode2 == 0 && opcode >= 0xd9 && opcode <= 0xdf {
            let t = opcode - 0xd9;
            if m_mod == 3 {
                let fht = ht_base.add(DELTA_FPU_MODRM + (t as usize) * 8);
                let tv = *fht.add(m_reg as usize);
                if (tv << m_rm) & 0x80 != 0 {
                    hs.flags |= F_ERROR | F_ERROR_OPCODE;
                }
            } else {
                let fht = ht_base.add(DELTA_FPU_REG);
                let tv = *fht.add(t as usize);
                if (tv << m_reg) & 0x80 != 0 {
                    hs.flags |= F_ERROR | F_ERROR_OPCODE;
                }
            }
        }

        if pref & PRE_LOCK != 0 {
            if m_mod == 3 {
                hs.flags |= F_ERROR | F_ERROR_LOCK;
            } else {
                let mut op = opcode;
                let table_end;
                let mut lock_ht;
                if hs.opcode2 != 0 {
                    lock_ht = ht_base.add(DELTA_OP2_LOCK_OK);
                    table_end = ht_base.add(DELTA_OP_ONLY_MEM);
                } else {
                    lock_ht = ht_base.add(DELTA_OP_LOCK_OK);
                    table_end = ht_base.add(DELTA_OP2_LOCK_OK);
                    op &= 0xfe;
                }
                let mut found = false;
                while lock_ht < table_end {
                    if *lock_ht == op {
                        lock_ht = lock_ht.add(1);
                        if ((*lock_ht) << m_reg) & 0x80 == 0 {
                            found = true;
                        }
                        break;
                    }
                    lock_ht = lock_ht.add(2);
                }
                if !found {
                    hs.flags |= F_ERROR | F_ERROR_LOCK;
                }
            }
        }

        if hs.opcode2 != 0 {
            match opcode {
                0x20 | 0x22 => {
                    m_mod = 3;
                    if m_reg > 4 || m_reg == 1 {
                        hs.flags |= F_ERROR | F_ERROR_OPERAND;
                    }
                }
                0x21 | 0x23 => {
                    m_mod = 3;
                    if m_reg == 4 || m_reg == 5 {
                        hs.flags |= F_ERROR | F_ERROR_OPERAND;
                    }
                }
                _ => {}
            }
        } else {
            match opcode {
                0x8c => {
                    if m_reg > 5 {
                        hs.flags |= F_ERROR | F_ERROR_OPERAND;
                    }
                }
                0x8e => {
                    if m_reg == 1 || m_reg > 5 {
                        hs.flags |= F_ERROR | F_ERROR_OPERAND;
                    }
                }
                _ => {}
            }
        }

        if m_mod == 3 {
            let table_end;
            let mut mem_ht;
            if hs.opcode2 != 0 {
                mem_ht = ht_base.add(DELTA_OP2_ONLY_MEM);
                table_end = ht_base.add(HDE64_TABLE.len());
            } else {
                mem_ht = ht_base.add(DELTA_OP_ONLY_MEM);
                table_end = ht_base.add(DELTA_OP2_ONLY_MEM);
            }
            while mem_ht < table_end {
                if *mem_ht == opcode {
                    mem_ht = mem_ht.add(1);
                    if *mem_ht & pref != 0 {
                        mem_ht = mem_ht.add(1);
                        if ((*mem_ht) << m_reg) & 0x80 == 0 {
                            hs.flags |= F_ERROR | F_ERROR_OPERAND;
                        }
                    }
                    break;
                }
                mem_ht = mem_ht.add(3);
            }
        } else if hs.opcode2 != 0 {
            match opcode {
                0x50 | 0xd7 | 0xf7 => {
                    if pref & (PRE_NONE | PRE_66) != 0 {
                        hs.flags |= F_ERROR | F_ERROR_OPERAND;
                    }
                }
                0xd6 => {
                    if pref & (PRE_F2 | PRE_F3) != 0 {
                        hs.flags |= F_ERROR | F_ERROR_OPERAND;
                    }
                }
                0xc5 => {
                    hs.flags |= F_ERROR | F_ERROR_OPERAND;
                }
                _ => {}
            }
        }

        c = *p;
        p = p.add(1);

        if m_reg <= 1 {
            if opcode == 0xf6 {
                cflags |= C_IMM8;
            } else if opcode == 0xf7 {
                cflags |= C_IMM_P66;
            }
        }

        match m_mod {
            0 => {
                if pref & PRE_67 != 0 {
                    if m_rm == 6 {
                        disp_size = 2;
                    }
                } else if m_rm == 5 {
                    disp_size = 4;
                }
            }
            1 => {
                disp_size = 1;
            }
            2 => {
                disp_size = 2;
                if pref & PRE_67 == 0 {
                    disp_size <<= 1;
                }
            }
            _ => {}
        }

        if m_mod != 3 && m_rm == 4 {
            hs.flags |= F_SIB;
            p = p.add(1);
            hs.sib = c;
            hs.sib_scale = c >> 6;
            hs.sib_index = (c & 0x3f) >> 3;
            hs.sib_base = c & 7;
            if hs.sib_base == 5 && (m_mod & 1) == 0 {
                disp_size = 4;
            }
        }

        p = p.sub(1);
        match disp_size {
            1 => {
                hs.flags |= F_DISP8;
                hs.disp.disp8 = *p;
            }
            2 => {
                hs.flags |= F_DISP16;
                hs.disp.disp16 = *(p as *const u16);
            }
            4 => {
                hs.flags |= F_DISP32;
                hs.disp.disp32 = *(p as *const u32);
            }
            _ => {}
        }
        p = p.add(disp_size as usize);
    } else if pref & PRE_LOCK != 0 {
        hs.flags |= F_ERROR | F_ERROR_LOCK;
    }

    if cflags & C_IMM_P66 != 0 {
        if cflags & C_REL32 != 0 {
            if pref & PRE_66 != 0 {
                hs.flags |= F_IMM16 | F_RELATIVE;
                hs.imm.imm64 = (*(p as *const u16)) as u64;
                p = p.add(2);
                hs.len = (p as usize - code as usize) as u8;
                if hs.len > 15 {
                    hs.flags |= F_ERROR | F_ERROR_LENGTH;
                    hs.len = 15;
                }
                return hs.len as u32;
            }
            hs.flags |= F_IMM32 | F_RELATIVE;
            hs.imm.imm64 = (*(p as *const u32)) as u64;
            p = p.add(4);
        } else if op64 != 0 {
            hs.flags |= F_IMM64;
            hs.imm.imm64 = *(p as *const u64);
            p = p.add(8);
        } else if pref & PRE_66 == 0 {
            hs.flags |= F_IMM32;
            hs.imm.imm64 = (*(p as *const u32)) as u64;
            p = p.add(4);
        } else {
            hs.flags |= F_IMM16;
            hs.imm.imm64 = (*(p as *const u16)) as u64;
            p = p.add(2);
        }
    } else {
        if cflags & C_IMM16 != 0 {
            hs.flags |= F_IMM16;
            hs.imm.imm64 = (*(p as *const u16)) as u64;
            p = p.add(2);
        }
        if cflags & C_IMM8 != 0 {
            hs.flags |= F_IMM8;
            hs.imm.imm64 = (*p) as u64;
            p = p.add(1);
        }

        if cflags & C_REL32 != 0 {
            hs.flags |= F_IMM32 | F_RELATIVE;
            hs.imm.imm64 = (*(p as *const u32)) as u64;
            p = p.add(4);
        } else if cflags & C_REL8 != 0 {
            hs.flags |= F_IMM8 | F_RELATIVE;
            hs.imm.imm64 = (*p) as u64;
            p = p.add(1);
        }
    }

    hs.len = (p as usize - code as usize) as u8;
    if hs.len > 15 {
        hs.flags |= F_ERROR | F_ERROR_LENGTH;
        hs.len = 15;
    }

    } // unsafe

    hs.len as u32
}
