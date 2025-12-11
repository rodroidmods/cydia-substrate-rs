use cydia_substrate::{ms_find_symbol, ms_hook_function, Image};
use std::ffi::c_void;

extern "C" fn hook_malloc(size: usize) -> *mut c_void {
    println!("malloc called with size: {}", size);
    unsafe {
        if let Some(original) = ORIGINAL_MALLOC {
            original(size)
        } else {
            std::ptr::null_mut()
        }
    }
}

static mut ORIGINAL_MALLOC: Option<extern "C" fn(usize) -> *mut c_void> = None;

fn main() {
    let image = Image::by_name("libc.so.6").expect("Failed to get libc image");
    let malloc_symbol = ms_find_symbol(image.handle(), "malloc");

    if !malloc_symbol.is_null() {
        match ms_hook_function(malloc_symbol, hook_malloc as *mut c_void) {
            Ok(original) => {
                unsafe {
                    ORIGINAL_MALLOC = Some(std::mem::transmute(original));
                }
                println!("Successfully hooked malloc!");
            }
            Err(e) => {
                eprintln!("Failed to hook malloc: {}", e);
            }
        }
    }
}
