#[cfg(all(target_arch = "aarch64", target_os = "android"))]
use cydia_substrate::{and64::a64_hook_function, utils};
#[cfg(all(target_arch = "aarch64", target_os = "android"))]
use std::ffi::c_void;

#[cfg(all(target_arch = "aarch64", target_os = "android"))]
static mut OLD_FUNCTION: Option<unsafe extern "C" fn(i32) -> i32> = None;

#[cfg(all(target_arch = "aarch64", target_os = "android"))]
unsafe extern "C" fn my_hook(value: i32) -> i32 {
    println!("Hook called with value: {}", value);

    if let Some(original) = OLD_FUNCTION {
        original(value * 2)
    } else {
        value * 2
    }
}

#[cfg(all(target_arch = "aarch64", target_os = "android"))]
fn main() {
    println!("Android ARM64 Inline Hook Example");

    if !utils::is_library_loaded("libc.so") {
        eprintln!("libc.so not loaded!");
        return;
    }

    match utils::get_absolute_address("libc.so", 0x12345) {
        Some(addr) => {
            println!("Found target function at: 0x{:x}", addr);

            match a64_hook_function(addr as *const c_void, my_hook as *const c_void) {
                Ok(original) => {
                    unsafe {
                        OLD_FUNCTION = Some(std::mem::transmute(original));
                    }
                    println!("Hook installed successfully!");
                }
                Err(e) => {
                    eprintln!("Failed to install hook: {}", e);
                }
            }
        }
        None => {
            eprintln!("Failed to find target function");
        }
    }
}

#[cfg(not(all(target_arch = "aarch64", target_os = "android")))]
fn main() {
    println!("This example only works on Android ARM64");
    println!("Current platform: {} - {}",
             std::env::consts::OS,
             std::env::consts::ARCH);
}
