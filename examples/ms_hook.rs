// Demonstrates using the C-compatible MSHookFunction / MSGetImageByName /
// MSFindSymbol / MSGetInstructionWidth APIs from Rust.
//
// These are the same symbols Cydia Substrate exposes for C/C++ consumers.
// From Rust, we call them directly through the crate's public API.

static mut ORIG_TARGET: *mut libc::c_void = std::ptr::null_mut();

extern "C" fn target_func(x: i32) -> i32 {
    x + 100
}

extern "C" fn replacement(x: i32) -> i32 {
    let original: extern "C" fn(i32) -> i32 = unsafe {
        std::mem::transmute(ORIG_TARGET)
    };
    let result = original(x);
    println!("[MSHook] original({}) = {}, returning {}", x, result, result * 3);
    result * 3
}

fn main() {
    println!("=== MSHookFunction example ===\n");

    // --- MSHookFunction: hook a local function ---
    println!("before hook: target_func(5) = {}", target_func(5));

    unsafe {
        substrate::MSHookFunction(
            target_func as *mut libc::c_void,
            replacement as *mut libc::c_void,
            &raw mut ORIG_TARGET as *mut *mut libc::c_void,
        );
    }

    println!("after hook:  target_func(5) = {}", target_func(5));
    println!("  (expected: (5+100)*3 = 315)\n");

    // --- MSGetInstructionWidth ---
    let width = unsafe { substrate::MSGetInstructionWidth(target_func as *mut libc::c_void) };
    println!("instruction width at target_func: {} bytes\n", width);

    // --- MSGetImageByName + MSFindSymbol ---
    let libc_name = std::ffi::CString::new("libc.so").unwrap();
    let handle = unsafe { substrate::MSGetImageByName(libc_name.as_ptr()) };
    if !handle.is_null() {
        println!("libc.so handle: {:p}", handle);

        let malloc_name = std::ffi::CString::new("malloc").unwrap();
        let malloc_addr = unsafe { substrate::MSFindSymbol(handle, malloc_name.as_ptr()) };
        println!("MSFindSymbol(libc, \"malloc\") = {:p}", malloc_addr);

        let free_name = std::ffi::CString::new("free").unwrap();
        let free_addr = unsafe { substrate::MSFindSymbol(handle, free_name.as_ptr()) };
        println!("MSFindSymbol(libc, \"free\")   = {:p}", free_addr);
    } else {
        println!("libc.so not found via MSGetImageByName (expected on non-Android)");
    }
}
