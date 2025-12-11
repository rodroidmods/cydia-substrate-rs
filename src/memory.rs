use std::ffi::c_void;

#[cfg(feature = "substrate-internal")]
use std::os::raw::c_int;

#[cfg(feature = "substrate-internal")]
use crate::{SubstrateAllocatorRef, SubstrateMemoryRef, SubstrateProcessRef};

#[cfg(feature = "substrate-internal")]
extern "C" {
    fn SubstrateProcessCreate(allocator: SubstrateAllocatorRef, pid: c_int) -> SubstrateProcessRef;
    fn SubstrateProcessRelease(process: SubstrateProcessRef);
    fn SubstrateMemoryCreate(
        allocator: SubstrateAllocatorRef,
        process: SubstrateProcessRef,
        data: *mut c_void,
        size: usize,
    ) -> SubstrateMemoryRef;
    fn SubstrateMemoryRelease(memory: SubstrateMemoryRef);
}

#[cfg(feature = "substrate-internal")]
pub struct Process {
    handle: SubstrateProcessRef,
}

#[cfg(feature = "substrate-internal")]
impl Process {
    pub fn create(pid: i32) -> Option<Self> {
        let handle = unsafe { SubstrateProcessCreate(std::ptr::null_mut(), pid) };
        if handle.is_null() {
            None
        } else {
            Some(Process { handle })
        }
    }

    pub fn handle(&self) -> SubstrateProcessRef {
        self.handle
    }
}

#[cfg(feature = "substrate-internal")]
impl Drop for Process {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                SubstrateProcessRelease(self.handle);
            }
        }
    }
}

#[cfg(feature = "substrate-internal")]
unsafe impl Send for Process {}
#[cfg(feature = "substrate-internal")]
unsafe impl Sync for Process {}

#[cfg(feature = "substrate-internal")]
pub struct Memory {
    handle: SubstrateMemoryRef,
}

#[cfg(feature = "substrate-internal")]
impl Memory {
    pub fn create(process: Option<&Process>, data: *mut c_void, size: usize) -> Option<Self> {
        let proc_handle = process.map(|p| p.handle()).unwrap_or(std::ptr::null_mut());
        let handle = unsafe {
            SubstrateMemoryCreate(std::ptr::null_mut(), proc_handle, data, size)
        };

        if handle.is_null() {
            None
        } else {
            Some(Memory { handle })
        }
    }

    pub fn handle(&self) -> SubstrateMemoryRef {
        self.handle
    }
}

#[cfg(feature = "substrate-internal")]
impl Drop for Memory {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                SubstrateMemoryRelease(self.handle);
            }
        }
    }
}

#[cfg(feature = "substrate-internal")]
unsafe impl Send for Memory {}
#[cfg(feature = "substrate-internal")]
unsafe impl Sync for Memory {}

pub struct MemoryProtection;

impl MemoryProtection {
    pub fn make_writable(addr: *mut c_void, size: usize) -> Result<(), &'static str> {
        #[cfg(unix)]
        {
            use libc::{mprotect, PROT_EXEC, PROT_READ, PROT_WRITE};
            let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as usize;
            let base = (addr as usize / page_size) * page_size;
            let aligned_size = ((addr as usize + size - 1) / page_size + 1) * page_size - base;

            let result = unsafe {
                mprotect(
                    base as *mut c_void,
                    aligned_size,
                    PROT_READ | PROT_WRITE | PROT_EXEC,
                )
            };

            if result == 0 {
                Ok(())
            } else {
                Err("mprotect failed")
            }
        }

        #[cfg(not(unix))]
        {
            Err("Unsupported platform")
        }
    }

    pub fn make_executable(addr: *mut c_void, size: usize) -> Result<(), &'static str> {
        #[cfg(unix)]
        {
            use libc::{mprotect, PROT_EXEC, PROT_READ};
            let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as usize;
            let base = (addr as usize / page_size) * page_size;
            let aligned_size = ((addr as usize + size - 1) / page_size + 1) * page_size - base;

            let result = unsafe {
                mprotect(base as *mut c_void, aligned_size, PROT_READ | PROT_EXEC)
            };

            if result == 0 {
                Self::clear_cache(addr, size);
                Ok(())
            } else {
                Err("mprotect failed")
            }
        }

        #[cfg(not(unix))]
        {
            Err("Unsupported platform")
        }
    }

    pub fn clear_cache(addr: *mut c_void, size: usize) {
        #[cfg(unix)]
        {
            extern "C" {
                fn __clear_cache(begin: *mut c_void, end: *mut c_void);
            }
            unsafe {
                __clear_cache(addr, (addr as usize + size) as *mut c_void);
            }
        }
    }
}
