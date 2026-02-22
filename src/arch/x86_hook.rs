use super::x86::*;
use crate::buffer::CodeWriter;
use crate::debug;
use crate::error::{Result, SubstrateError};
use crate::hde64::{self, Hde64};
use crate::log::*;
use crate::memory::{self, MemoryGuard};

fn get_instruction_width_intel(start: *const u8) -> usize {
    let mut decode = Hde64::default();
    unsafe { hde64::hde64_disasm(start, &mut decode) as usize }
}

pub unsafe fn hook_function_x86(
    symbol: *mut u8,
    replace: *mut u8,
    result: *mut *mut u8,
) -> Result<usize> {
    if symbol.is_null() {
        return Err(SubstrateError::NullSymbol);
    }

    if debug::is_debug() {
        info!("MSHookFunction({:p}, {:p}, {:p})", symbol, replace, result);
    }

    let source = symbol as usize;
    let target = replace as usize;
    let area = symbol;

    let required = size_of_jump_from(target, source);

    if debug::is_debug() {
        let name = format!("{:p}", area);
        debug::log_hex(
            unsafe { core::slice::from_raw_parts(area, 32) },
            Some(&name),
        );
    }

    let mut used = 0usize;
    while used < required {
        let width = get_instruction_width_intel(unsafe { area.add(used) });
        if width == 0 {
            return Err(SubstrateError::DecodeError(used));
        }
        used += width;
    }

    let blank = used - required;

    unsafe {

    let mut backup = vec![0u8; used];
    core::ptr::copy_nonoverlapping(area, backup.as_mut_ptr(), used);

    if debug::is_debug() {
        let name = format!("{:p}", area);
        debug::log_hex(
            core::slice::from_raw_parts(area, used + 2),
            Some(&name),
        );
    }

    if !result.is_null() {
        if backup[0] == 0xe9 {
            let rel = *(backup.as_ptr().add(1) as *const u32) as i32;
            *result = (source.wrapping_add(5).wrapping_add(rel as usize)) as *mut u8;
            return Ok(used);
        }

        if !IA32 && backup[0] == 0xff && backup[1] == 0x25 {
            let rel = *(backup.as_ptr().add(2) as *const u32) as i32;
            let ptr_addr = source.wrapping_add(6).wrapping_add(rel as usize);
            *result = *(ptr_addr as *const *mut u8);
            return Ok(used);
        }

        let mut length = used + size_of_jump_blind(source + used);

        let mut offset = 0usize;
        while offset < used {
            let mut decode = Hde64::default();
            hde64::hde64_disasm(backup.as_ptr().add(offset), &mut decode);
            let width = decode.len as usize;

            #[cfg(target_pointer_width = "64")]
            {
                if (decode.modrm & 0xc7) == 0x05 {
                    if decode.opcode == 0x8b {
                        let destiny_addr = area.add(offset).add(width) as usize;
                        let disp = decode.disp.disp32 as i32;
                        let destiny = destiny_addr.wrapping_add(disp as usize);
                        let reg = (decode.rex_r << 3) | decode.modrm_reg;
                        length -= decode.len as usize;
                        length += size_of_push_pointer(destiny as *const u8);
                        length += size_of_pop(reg);
                        length += size_of_move64();
                    }
                } else if backup[offset] == 0xe8 {
                    let relative = *(backup.as_ptr().add(offset + 1) as *const i32);
                    let destiny = area.add(offset).add(decode.len as usize) as usize;
                    let destiny = destiny.wrapping_add(relative as usize);

                    if relative == 0 {
                        length -= decode.len as usize;
                        length += size_of_push_pointer(destiny as *const u8);
                    } else {
                        length += size_of_skip();
                        length += size_of_jump_blind(destiny);
                    }
                } else if backup[offset] == 0xeb {
                    length -= decode.len as usize;
                    let rel8 = *(backup.as_ptr().add(offset + 1) as *const i8);
                    let dest = area.add(offset).add(decode.len as usize) as usize;
                    let dest = dest.wrapping_add(rel8 as usize);
                    length += size_of_jump_blind(dest);
                } else if backup[offset] == 0xe9 {
                    length -= decode.len as usize;
                    let rel32 = *(backup.as_ptr().add(offset + 1) as *const i32);
                    let dest = area.add(offset).add(decode.len as usize) as usize;
                    let dest = dest.wrapping_add(rel32 as usize);
                    length += size_of_jump_blind(dest);
                } else if backup[offset] == 0xe3 || (backup[offset] & 0xf0) == 0x70 {
                    length += decode.len as usize;
                    let rel8 = *(backup.as_ptr().add(offset + 1) as *const i8);
                    let dest = area.add(offset).add(decode.len as usize) as usize;
                    let dest = dest.wrapping_add(rel8 as usize);
                    length += size_of_jump_blind(dest);
                }
            }

            #[cfg(target_pointer_width = "32")]
            {
                if backup[offset] == 0xe8 {
                    let relative = *(backup.as_ptr().add(offset + 1) as *const i32);
                    let destiny = area.add(offset).add(decode.len as usize) as usize;
                    let destiny = destiny.wrapping_add(relative as usize);

                    if relative == 0 {
                        length -= decode.len as usize;
                        length += size_of_push_pointer(destiny as *const u8);
                    } else {
                        length += size_of_skip();
                        length += size_of_jump_blind(destiny);
                    }
                } else if backup[offset] == 0xeb {
                    length -= decode.len as usize;
                    let rel8 = *(backup.as_ptr().add(offset + 1) as *const i8);
                    let dest = area.add(offset).add(decode.len as usize) as usize;
                    let dest = dest.wrapping_add(rel8 as usize);
                    length += size_of_jump_blind(dest);
                } else if backup[offset] == 0xe9 {
                    length -= decode.len as usize;
                    let rel32 = *(backup.as_ptr().add(offset + 1) as *const i32);
                    let dest = area.add(offset).add(decode.len as usize) as usize;
                    let dest = dest.wrapping_add(rel32 as usize);
                    length += size_of_jump_blind(dest);
                } else if backup[offset] == 0xe3 || (backup[offset] & 0xf0) == 0x70 {
                    length += decode.len as usize;
                    let rel8 = *(backup.as_ptr().add(offset + 1) as *const i8);
                    let dest = area.add(offset).add(decode.len as usize) as usize;
                    let dest = dest.wrapping_add(rel8 as usize);
                    length += size_of_jump_blind(dest);
                }
            }

            offset += width;
        }

        let buffer = memory::alloc_rwx(length)?;

        {
            let mut w = CodeWriter::new(buffer);

            let mut offset = 0usize;
            while offset < used {
                let mut decode = Hde64::default();
                hde64::hde64_disasm(backup.as_ptr().add(offset), &mut decode);
                let width = decode.len as usize;

                #[cfg(target_pointer_width = "64")]
                {
                    if (decode.modrm & 0xc7) == 0x05 {
                        if decode.opcode == 0x8b {
                            let destiny_addr = area.add(offset).add(width) as usize;
                            let disp = decode.disp.disp32 as i32;
                            let destiny = destiny_addr.wrapping_add(disp as usize);
                            let reg = (decode.rex_r << 3) | decode.modrm_reg;
                            push_pointer(&mut w, destiny);
                            write_pop(&mut w, reg);
                            write_move64(&mut w, reg, reg);
                        } else {
                            w.write_bytes(&backup[offset..offset + width]);
                        }
                        offset += width;
                        continue;
                    }
                }

                if backup[offset] == 0xe8 {
                    let relative = *(backup.as_ptr().add(offset + 1) as *const i32);
                    if relative == 0 {
                        let dest = area.add(offset).add(decode.len as usize) as usize;
                        push_pointer(&mut w, dest);
                    } else {
                        w.write_u8(0xe8);
                        w.write_i32(size_of_skip() as i32);
                        let destiny = area.add(offset).add(decode.len as usize) as usize;
                        let destiny = destiny.wrapping_add(relative as usize);
                        let skip_size = size_of_jump_from(destiny, w.address() + size_of_skip());
                        write_skip(&mut w, skip_size as isize);
                        write_jump(&mut w, destiny);
                    }
                } else if backup[offset] == 0xeb {
                    let rel8 = *(backup.as_ptr().add(offset + 1) as *const i8);
                    let dest = area.add(offset).add(decode.len as usize) as usize;
                    let dest = dest.wrapping_add(rel8 as usize);
                    write_jump(&mut w, dest);
                } else if backup[offset] == 0xe9 {
                    let rel32 = *(backup.as_ptr().add(offset + 1) as *const i32);
                    let dest = area.add(offset).add(decode.len as usize) as usize;
                    let dest = dest.wrapping_add(rel32 as usize);
                    write_jump(&mut w, dest);
                } else if backup[offset] == 0xe3 || (backup[offset] & 0xf0) == 0x70 {
                    w.write_u8(backup[offset]);
                    w.write_u8(2);
                    w.write_u8(0xeb);
                    let rel8 = *(backup.as_ptr().add(offset + 1) as *const i8);
                    let dest = area.add(offset).add(decode.len as usize) as usize;
                    let dest = dest.wrapping_add(rel8 as usize);
                    let jmp_size = size_of_jump_from(dest, w.address() + 1) as u8;
                    w.write_u8(jmp_size);
                    write_jump(&mut w, dest);
                } else {
                    w.write_bytes(&backup[offset..offset + width]);
                }

                offset += width;
            }

            write_jump(&mut w, area.add(used) as usize);
        }

        memory::protect_rx(buffer, length)?;

        *result = buffer;

        if debug::is_debug() {
            let name = format!("{:p}", *result);
            debug::log_hex(
                core::slice::from_raw_parts(buffer, length),
                Some(&name),
            );
        }
    }

    {
        let _guard = MemoryGuard::new(area, used)?;
        let mut w = CodeWriter::new(area);
        write_jump(&mut w, target);
        for _ in 0..blank {
            w.write_u8(0x90);
        }
    }

    if debug::is_debug() {
        let name = format!("{:p}", area);
        debug::log_hex(
            core::slice::from_raw_parts(area, used + 2),
            Some(&name),
        );
    }

    } // unsafe

    Ok(used)
}
