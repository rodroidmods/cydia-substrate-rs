use std::ffi::c_void;

#[cfg(all(target_arch = "aarch64", target_os = "android"))]
extern "C" {
    pub fn A64HookFunction(symbol: *const c_void, replace: *const c_void, result: *mut *mut c_void);
    pub fn A64HookFunctionV(symbol: *const c_void, replace: *const c_void, rwx: *const c_void, rwx_size: usize) -> *mut c_void;
}

#[cfg(all(target_arch = "aarch64", target_os = "android"))]
pub fn a64_hook_function(
    symbol: *const c_void,
    replace: *const c_void,
) -> Result<*mut c_void, &'static str> {
    if symbol.is_null() {
        return Err("Symbol pointer is null");
    }
    if replace.is_null() {
        return Err("Replace pointer is null");
    }

    let mut result: *mut c_void = std::ptr::null_mut();
    unsafe {
        A64HookFunction(symbol, replace, &mut result as *mut *mut c_void);
    }

    if result.is_null() {
        return Err("Hook failed");
    }

    Ok(result)
}

#[cfg(all(target_arch = "aarch64", target_os = "android"))]
pub fn a64_hook_function_v(
    symbol: *const c_void,
    replace: *const c_void,
    rwx: *const c_void,
    rwx_size: usize,
) -> Result<*mut c_void, &'static str> {
    if symbol.is_null() {
        return Err("Symbol pointer is null");
    }
    if replace.is_null() {
        return Err("Replace pointer is null");
    }

    let result = unsafe {
        A64HookFunctionV(symbol, replace, rwx, rwx_size)
    };

    if result.is_null() {
        return Err("Hook failed");
    }

    Ok(result)
}
