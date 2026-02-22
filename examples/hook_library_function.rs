use substrate::{hook_function, find_symbol};
use std::process;

// Demonstrates hooking a shared library function (e.g., libc's `open`).
//
// This pattern is common in Android modding — intercepting system calls
// or game library functions to modify their behavior at runtime.
//
// ⚠️  This example must be run on Android (or Linux with libc.so).

static mut ORIGINAL_OPEN: *mut u8 = std::ptr::null_mut();

// Replacement for libc's `open()`.
//
// Logs every file open attempt, then forwards to the real `open`.
unsafe extern "C" fn hooked_open(
    path: *const libc::c_char,
    flags: libc::c_int,
    mode: libc::mode_t,
) -> libc::c_int {
    let path_str = if !path.is_null() {
        unsafe { std::ffi::CStr::from_ptr(path).to_string_lossy() }
    } else {
        "<null>".into()
    };

    println!("[hook] open(\"{}\", flags=0x{:x}, mode=0o{:o})", path_str, flags, mode);

    // Call the original open via trampoline
    let real_open: unsafe extern "C" fn(
        *const libc::c_char,
        libc::c_int,
        libc::mode_t,
    ) -> libc::c_int = unsafe { std::mem::transmute(ORIGINAL_OPEN) };

    let fd = unsafe { real_open(path, flags, mode) };
    println!("[hook] open returned fd={}", fd);
    fd
}

fn main() {
    let pid = process::id() as i32;
    println!("=== substrate library hook example ===");
    println!("PID: {}\n", pid);

    // Find libc's open()
    let open_addr = match find_symbol(pid, "open", "libc.so") {
        Ok(addr) => {
            println!("found open() at 0x{:x}", addr);
            addr
        }
        Err(e) => {
            eprintln!("failed to find open: {}", e);
            return;
        }
    };

    // Install the hook
    unsafe {
        match hook_function(
            open_addr as *mut u8,
            hooked_open as *mut u8,
            &raw mut ORIGINAL_OPEN as *mut *mut u8,
        ) {
            Ok(bytes) => {
                println!("hooked open() successfully ({} bytes patched)", bytes);
                let trampoline = ORIGINAL_OPEN;
                println!("trampoline at {:p}\n", trampoline);
            }
            Err(e) => {
                eprintln!("failed to hook open: {}", e);
                return;
            }
        }
    }

    // Now any call to open() goes through our hook
    let test_path = std::ffi::CString::new("/dev/null").unwrap();
    let fd = unsafe { libc::open(test_path.as_ptr(), libc::O_RDONLY) };
    println!("\nlibc::open(\"/dev/null\") returned fd={}", fd);
    if fd >= 0 {
        unsafe { libc::close(fd) };
    }
}
