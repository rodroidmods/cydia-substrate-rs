use crate::error::{Result, SubstrateError};

const MAX_NAME_LEN: usize = 256;
const MEMORY_ONLY: &str = "[memory]";

struct MemoryMapping {
    name: [u8; MAX_NAME_LEN],
    name_len: usize,
    start: usize,
    end: usize,
}

impl MemoryMapping {
    fn new() -> Self {
        Self {
            name: [0u8; MAX_NAME_LEN],
            name_len: 0,
            start: 0,
            end: 0,
        }
    }

    fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }

    fn set_name(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let len = bytes.len().min(MAX_NAME_LEN - 1);
        self.name[..len].copy_from_slice(&bytes[..len]);
        self.name[len] = 0;
        self.name_len = len;
    }
}

fn load_memmap(pid: libc::pid_t) -> Result<Vec<MemoryMapping>> {
    let path = format!("/proc/{}/maps", pid);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| SubstrateError::MemoryMapError(format!("cannot open {}: {}", path, e)))?;

    let mut mappings: Vec<MemoryMapping> = Vec::new();

    for line in content.lines() {
        if line.is_empty() {
            continue;
        }

        let mut parts = line.splitn(6, char::is_whitespace);
        let range = parts.next().unwrap_or("");
        let mut range_parts = range.split('-');
        let start_str = range_parts.next().unwrap_or("0");
        let end_str = range_parts.next().unwrap_or("0");

        let start = usize::from_str_radix(start_str, 16).unwrap_or(0);
        let end = usize::from_str_radix(end_str, 16).unwrap_or(0);

        let _perms = parts.next();
        let _offset = parts.next();
        let _dev = parts.next();
        let _inode = parts.next();
        let name = parts.next().map(|s| s.trim()).unwrap_or("");

        if name.is_empty() {
            let mut m = MemoryMapping::new();
            m.start = start;
            m.end = end;
            m.set_name(MEMORY_ONLY);
            mappings.push(m);
            continue;
        }

        let mut found = false;
        for existing in mappings.iter_mut().rev() {
            if existing.name_str() == name {
                if start < existing.start {
                    existing.start = start;
                }
                if end > existing.end {
                    existing.end = end;
                }
                found = true;
                break;
            }
        }

        if !found {
            let mut m = MemoryMapping::new();
            m.start = start;
            m.end = end;
            m.set_name(name);
            mappings.push(m);
        }
    }

    Ok(mappings)
}

fn find_lib_in_maps(lib_name: &str, mappings: &[MemoryMapping]) -> Option<(String, usize)> {
    for m in mappings {
        let name = m.name_str();
        if name == MEMORY_ONLY {
            continue;
        }
        let basename = match name.rfind('/') {
            Some(pos) => &name[pos + 1..],
            None => continue,
        };
        if !basename.starts_with(lib_name) {
            continue;
        }
        let rest = &basename[lib_name.len()..];
        if rest.starts_with("so") || rest.starts_with(".so") || rest.is_empty() || rest.starts_with('-') {
            unsafe {
                libc::mprotect(
                    m.start as *mut libc::c_void,
                    m.end - m.start,
                    libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
                );
            }
            return Some((name.to_string(), m.start));
        }
    }
    None
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf32Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u32,
    e_phoff: u32,
    e_shoff: u32,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf32Shdr {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u32,
    sh_addr: u32,
    sh_offset: u32,
    sh_size: u32,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u32,
    sh_entsize: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf32Sym {
    st_name: u32,
    st_value: u32,
    st_size: u32,
    st_info: u8,
    st_other: u8,
    st_shndx: u16,
}

const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;
const SHT_DYNSYM: u32 = 11;
const STT_FUNC: u8 = 2;
const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

struct SymList {
    symbols: Vec<Elf32Sym>,
    strings: Vec<u8>,
}

struct SymTab {
    static_syms: Option<SymList>,
    dynamic_syms: Option<SymList>,
}

fn read_at<T: Copy>(fd: libc::c_int, offset: u64) -> Option<T> {
    unsafe {
        libc::lseek(fd, offset as libc::off_t, libc::SEEK_SET);
        let mut val = core::mem::MaybeUninit::<T>::uninit();
        let size = core::mem::size_of::<T>();
        let ret = libc::read(fd, val.as_mut_ptr() as *mut libc::c_void, size as _);
        if (ret as usize) != size {
            return None;
        }
        Some(val.assume_init())
    }
}

fn read_vec(fd: libc::c_int, offset: u64, size: usize) -> Option<Vec<u8>> {
    unsafe {
        libc::lseek(fd, offset as libc::off_t, libc::SEEK_SET);
        let mut buf = vec![0u8; size];
        let ret = libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, size as _);
        if (ret as usize) != size {
            return None;
        }
        Some(buf)
    }
}

fn load_symtab(path: &str) -> Result<SymTab> {
    let c_path = std::ffi::CString::new(path)
        .map_err(|_| SubstrateError::ElfParseFailed("invalid path".into()))?;

    let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDONLY) };
    if fd < 0 {
        return Err(SubstrateError::ElfParseFailed(format!("cannot open {}", path)));
    }

    let result = load_symtab_from_fd(fd);
    unsafe { libc::close(fd); }
    result
}

fn load_symtab_from_fd(fd: libc::c_int) -> Result<SymTab> {
    let ehdr: Elf32Ehdr = read_at(fd, 0)
        .ok_or_else(|| SubstrateError::ElfParseFailed("cannot read ELF header".into()))?;

    if ehdr.e_ident[..4] != ELF_MAGIC {
        return Err(SubstrateError::ElfParseFailed("not an ELF file".into()));
    }

    if ehdr.e_shentsize as usize != core::mem::size_of::<Elf32Shdr>() {
        return Err(SubstrateError::ElfParseFailed("section header size mismatch".into()));
    }

    let sh_size = ehdr.e_shentsize as usize * ehdr.e_shnum as usize;
    let sh_data = read_vec(fd, ehdr.e_shoff as u64, sh_size)
        .ok_or_else(|| SubstrateError::ElfParseFailed("cannot read section headers".into()))?;

    let sections: &[Elf32Shdr] = unsafe {
        core::slice::from_raw_parts(
            sh_data.as_ptr() as *const Elf32Shdr,
            ehdr.e_shnum as usize,
        )
    };

    let shstrtab_sec = &sections[ehdr.e_shstrndx as usize];
    let shstrtab = read_vec(fd, shstrtab_sec.sh_offset as u64, shstrtab_sec.sh_size as usize)
        .ok_or_else(|| SubstrateError::ElfParseFailed("cannot read shstrtab".into()))?;

    let mut symh: Option<&Elf32Shdr> = None;
    let mut strh: Option<&Elf32Shdr> = None;
    let mut dynsymh: Option<&Elf32Shdr> = None;
    let mut dynstrh: Option<&Elf32Shdr> = None;

    for sec in sections {
        let name_offset = sec.sh_name as usize;
        let sec_name = cstr_from_bytes(&shstrtab, name_offset);

        match sec.sh_type {
            SHT_SYMTAB => {
                if symh.is_some() {
                    return Err(SubstrateError::ElfParseFailed("too many symbol tables".into()));
                }
                symh = Some(sec);
            }
            SHT_DYNSYM => {
                if dynsymh.is_some() {
                    return Err(SubstrateError::ElfParseFailed("too many dynamic symbol tables".into()));
                }
                dynsymh = Some(sec);
            }
            SHT_STRTAB => {
                if sec_name == ".strtab" {
                    if strh.is_some() {
                        return Err(SubstrateError::ElfParseFailed("too many string tables".into()));
                    }
                    strh = Some(sec);
                } else if sec_name == ".dynstr" {
                    if dynstrh.is_some() {
                        return Err(SubstrateError::ElfParseFailed("too many dynamic string tables".into()));
                    }
                    dynstrh = Some(sec);
                }
            }
            _ => {}
        }
    }

    if dynsymh.is_none() && symh.is_none() {
        return Err(SubstrateError::ElfParseFailed("no symbol table".into()));
    }

    let dynamic_syms = if let (Some(dsym), Some(dstr)) = (dynsymh, dynstrh) {
        load_sym_list(fd, dsym, dstr)?
    } else {
        None
    };

    let static_syms = if let (Some(sym), Some(str_sec)) = (symh, strh) {
        load_sym_list(fd, sym, str_sec)?
    } else {
        None
    };

    Ok(SymTab {
        static_syms,
        dynamic_syms,
    })
}

fn load_sym_list(fd: libc::c_int, symh: &Elf32Shdr, strh: &Elf32Shdr) -> Result<Option<SymList>> {
    if symh.sh_size as usize % core::mem::size_of::<Elf32Sym>() != 0 {
        return Ok(None);
    }

    let sym_data = read_vec(fd, symh.sh_offset as u64, symh.sh_size as usize)
        .ok_or_else(|| SubstrateError::ElfParseFailed("cannot read symbol table".into()))?;
    let str_data = read_vec(fd, strh.sh_offset as u64, strh.sh_size as usize)
        .ok_or_else(|| SubstrateError::ElfParseFailed("cannot read string table".into()))?;

    let num = symh.sh_size as usize / core::mem::size_of::<Elf32Sym>();
    let symbols: Vec<Elf32Sym> = unsafe {
        let ptr = sym_data.as_ptr() as *const Elf32Sym;
        core::slice::from_raw_parts(ptr, num).to_vec()
    };

    Ok(Some(SymList {
        symbols,
        strings: str_data,
    }))
}

fn cstr_from_bytes(data: &[u8], offset: usize) -> &str {
    if offset >= data.len() {
        return "";
    }
    let slice = &data[offset..];
    let end = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
    core::str::from_utf8(&slice[..end]).unwrap_or("")
}

fn lookup_func(symtab: &SymTab, name: &str) -> Option<u32> {
    if let Some(ref dyn_syms) = symtab.dynamic_syms {
        if let Some(val) = lookup_in_list(dyn_syms, STT_FUNC, name) {
            return Some(val);
        }
    }
    if let Some(ref static_syms) = symtab.static_syms {
        if let Some(val) = lookup_in_list(static_syms, STT_FUNC, name) {
            return Some(val);
        }
    }
    None
}

fn lookup_in_list(sl: &SymList, sym_type: u8, name: &str) -> Option<u32> {
    for sym in &sl.symbols {
        let sym_name = cstr_from_bytes(&sl.strings, sym.st_name as usize);
        if sym_name == name && (sym.st_info & 0xf) == sym_type {
            return Some(sym.st_value);
        }
    }
    None
}

pub fn find_name(pid: libc::pid_t, name: &str, lib_name: &str) -> Result<usize> {
    let mappings = load_memmap(pid)?;

    let (lib_path, lib_base) = find_lib_in_maps(lib_name, &mappings)
        .ok_or_else(|| SubstrateError::LibraryNotFound(lib_name.to_string()))?;

    let symtab = load_symtab(&lib_path)?;

    let addr = lookup_func(&symtab, name)
        .ok_or_else(|| SubstrateError::SymbolNotFound(name.to_string()))?;

    Ok(addr as usize + lib_base)
}

pub fn find_libbase(pid: libc::pid_t, lib_name: &str) -> Result<usize> {
    let mappings = load_memmap(pid)?;

    let (_, lib_base) = find_lib_in_maps(lib_name, &mappings)
        .ok_or_else(|| SubstrateError::LibraryNotFound(lib_name.to_string()))?;

    Ok(lib_base)
}
