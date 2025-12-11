use std::ffi::c_void;
use std::os::raw::c_char;

extern "C" {
    pub fn find_name(
        pid: libc::pid_t,
        name: *const c_char,
        libn: *const c_char,
        addr: *mut libc::c_ulong,
    ) -> i32;

    pub fn find_libbase(
        pid: libc::pid_t,
        libn: *const c_char,
        addr: *mut libc::c_ulong,
    ) -> i32;
}

pub fn find_symbol_in_library(
    pid: i32,
    symbol_name: &str,
    library_name: &str,
) -> Result<usize, &'static str> {
    let c_symbol = std::ffi::CString::new(symbol_name).map_err(|_| "Invalid symbol name")?;
    let c_library = std::ffi::CString::new(library_name).map_err(|_| "Invalid library name")?;

    let mut addr: libc::c_ulong = 0;
    let result = unsafe {
        find_name(
            pid,
            c_symbol.as_ptr(),
            c_library.as_ptr(),
            &mut addr as *mut libc::c_ulong,
        )
    };

    if result == 0 {
        Ok(addr as usize)
    } else {
        Err("Symbol not found")
    }
}

pub fn find_library_base(pid: i32, library_name: &str) -> Result<usize, &'static str> {
    let c_library = std::ffi::CString::new(library_name).map_err(|_| "Invalid library name")?;

    let mut addr: libc::c_ulong = 0;
    let result = unsafe { find_libbase(pid, c_library.as_ptr(), &mut addr as *mut libc::c_ulong) };

    if result == 0 {
        Ok(addr as usize)
    } else {
        Err("Library not found")
    }
}
