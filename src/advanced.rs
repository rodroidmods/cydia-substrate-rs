use std::ffi::c_void;
use crate::memory::MemoryProtection;

pub struct Trampoline {
    code: Vec<u8>,
    ptr: *mut c_void,
}

impl Trampoline {
    pub fn new(capacity: usize) -> Result<Self, &'static str> {
        #[cfg(unix)]
        {
            use libc::{mmap, MAP_ANON, MAP_PRIVATE, PROT_READ, PROT_WRITE};

            let ptr = unsafe {
                mmap(
                    std::ptr::null_mut(),
                    capacity,
                    PROT_READ | PROT_WRITE,
                    MAP_ANON | MAP_PRIVATE,
                    -1,
                    0,
                )
            };

            if ptr == libc::MAP_FAILED {
                return Err("Failed to allocate trampoline memory");
            }

            Ok(Trampoline {
                code: Vec::with_capacity(capacity),
                ptr,
            })
        }

        #[cfg(not(unix))]
        {
            Err("Unsupported platform")
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), &'static str> {
        if self.code.len() + bytes.len() > self.code.capacity() {
            return Err("Trampoline capacity exceeded");
        }

        self.code.extend_from_slice(bytes);
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                (self.ptr as *mut u8).add(self.code.len() - bytes.len()),
                bytes.len(),
            );
        }

        Ok(())
    }

    pub fn finalize(self) -> Result<*mut c_void, &'static str> {
        MemoryProtection::make_executable(self.ptr, self.code.len())?;
        Ok(self.ptr)
    }

    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }
}

impl Drop for Trampoline {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            if !self.ptr.is_null() {
                unsafe {
                    libc::munmap(self.ptr, self.code.capacity());
                }
            }
        }
    }
}

pub struct InlineHook {
    target: *mut c_void,
    original_bytes: Vec<u8>,
    trampoline: Option<*mut c_void>,
}

impl InlineHook {
    pub fn new(target: *mut c_void, hook_size: usize) -> Result<Self, &'static str> {
        if target.is_null() {
            return Err("Target is null");
        }

        let original_bytes = unsafe {
            std::slice::from_raw_parts(target as *const u8, hook_size).to_vec()
        };

        Ok(InlineHook {
            target,
            original_bytes,
            trampoline: None,
        })
    }

    pub fn install(&mut self, replacement: *mut c_void) -> Result<(), &'static str> {
        if replacement.is_null() {
            return Err("Replacement is null");
        }

        MemoryProtection::make_writable(self.target, self.original_bytes.len())?;

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        {
            use crate::arch::x86::X86Instruction;

            let target_addr = self.target as usize;
            let replace_addr = replacement as usize;
            let size = X86Instruction::size_of_jump(replace_addr, target_addr);

            let mut jump_code = vec![0u8; size];
            X86Instruction::write_jump_address(&mut jump_code, 0, replace_addr, target_addr);

            unsafe {
                std::ptr::copy_nonoverlapping(
                    jump_code.as_ptr(),
                    self.target as *mut u8,
                    jump_code.len(),
                );
            }

            MemoryProtection::make_executable(self.target, jump_code.len())?;
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
        {
            return Err("Inline hooks not yet implemented for this architecture");
        }

        Ok(())
    }

    pub fn uninstall(&self) -> Result<(), &'static str> {
        MemoryProtection::make_writable(self.target, self.original_bytes.len())?;

        unsafe {
            std::ptr::copy_nonoverlapping(
                self.original_bytes.as_ptr(),
                self.target as *mut u8,
                self.original_bytes.len(),
            );
        }

        MemoryProtection::make_executable(self.target, self.original_bytes.len())?;

        Ok(())
    }

    pub fn create_trampoline(&mut self) -> Result<*mut c_void, &'static str> {
        let mut tramp = Trampoline::new(256)?;

        tramp.write_bytes(&self.original_bytes)?;

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        {
            use crate::arch::x86::X86Instruction;

            let return_addr = (self.target as usize) + self.original_bytes.len();
            let tramp_addr = tramp.as_ptr() as usize + tramp.len();
            let size = X86Instruction::size_of_jump(return_addr, tramp_addr);

            let mut jump_code = vec![0u8; size];
            X86Instruction::write_jump_address(&mut jump_code, 0, return_addr, tramp_addr);
            tramp.write_bytes(&jump_code)?;
        }

        let ptr = tramp.finalize()?;
        self.trampoline = Some(ptr);
        Ok(ptr)
    }

    pub fn original_bytes(&self) -> &[u8] {
        &self.original_bytes
    }

    pub fn trampoline(&self) -> Option<*mut c_void> {
        self.trampoline
    }
}

pub struct HookChain {
    hooks: Vec<InlineHook>,
}

impl HookChain {
    pub fn new() -> Self {
        HookChain { hooks: Vec::new() }
    }

    pub fn add_hook(&mut self, hook: InlineHook) {
        self.hooks.push(hook);
    }

    pub fn install_all(&mut self, replacements: &[*mut c_void]) -> Result<(), &'static str> {
        if self.hooks.len() != replacements.len() {
            return Err("Hook count mismatch");
        }

        for (hook, &replacement) in self.hooks.iter_mut().zip(replacements.iter()) {
            hook.install(replacement)?;
        }

        Ok(())
    }

    pub fn uninstall_all(&self) -> Result<(), &'static str> {
        for hook in &self.hooks {
            hook.uninstall()?;
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.hooks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }
}

impl Default for HookChain {
    fn default() -> Self {
        Self::new()
    }
}
