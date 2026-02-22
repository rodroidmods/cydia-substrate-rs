use crate::error::{Result, SubstrateError};
use crate::log::*;

unsafe extern "C" {
    fn __clear_cache(beg: *mut libc::c_void, end: *mut libc::c_void);
}

pub struct MemoryGuard {
    address: *mut libc::c_void,
    width: usize,
}

impl MemoryGuard {
    pub unsafe fn new(data: *mut u8, size: usize) -> Result<Self> {
        if size == 0 {
            return Err(SubstrateError::MprotectFailed(0));
        }

        let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize };
        let base = (data as usize) / page * page;
        let width = ((data as usize + size - 1) / page + 1) * page - base;
        let address = base as *mut libc::c_void;

        let ret = unsafe {
            libc::mprotect(
                address,
                width,
                libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
            )
        };

        if ret == -1 {
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            error!("mprotect() = {}", errno);
            return Err(SubstrateError::MprotectFailed(errno));
        }

        Ok(Self { address, width })
    }
}

impl Drop for MemoryGuard {
    fn drop(&mut self) {
        unsafe {
            let ret = libc::mprotect(
                self.address,
                self.width,
                libc::PROT_READ | libc::PROT_EXEC,
            );
            if ret == -1 {
                let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
                error!("mprotect restore failed: {}", errno);
            }
            __clear_cache(
                self.address,
                (self.address as *mut u8).add(self.width) as *mut libc::c_void,
            );
        }
    }
}

pub unsafe fn alloc_rwx(size: usize) -> Result<*mut u8> {
    let ptr = unsafe {
        libc::mmap(
            core::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_ANON | libc::MAP_PRIVATE,
            -1,
            0,
        )
    };

    if ptr == libc::MAP_FAILED {
        let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        error!("mmap() = {}", errno);
        return Err(SubstrateError::MmapFailed(errno));
    }

    Ok(ptr as *mut u8)
}

pub unsafe fn protect_rx(ptr: *mut u8, size: usize) -> Result<()> {
    let ret = unsafe { libc::mprotect(ptr as *mut libc::c_void, size, libc::PROT_READ | libc::PROT_EXEC) };
    if ret == -1 {
        let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        error!("mprotect() = {}", errno);
        return Err(SubstrateError::MprotectFailed(errno));
    }
    Ok(())
}

pub unsafe fn dealloc(ptr: *mut u8, size: usize) {
    unsafe {
        libc::munmap(ptr as *mut libc::c_void, size);
    }
}
