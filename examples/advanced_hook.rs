use cydia_substrate::{substrate_hook, find_and_hook, Image, debug_print};
use std::ffi::c_void;

substrate_hook! {
    fn malloc(size: usize) -> *mut c_void {
        debug_print!("malloc intercepted: size = {}", size);

        unsafe {
            let result = call_original_malloc(size);
            debug_print!("malloc returning: {:p}", result);
            result
        }
    }
}

fn main() {
    println!("Advanced Hook Example");

    let image = Image::by_name("libc.so.6")
        .expect("Failed to load libc");

    match install_malloc_hook(cydia_substrate::ms_find_symbol(image.handle(), "malloc")) {
        Ok(_) => println!("Successfully hooked malloc!"),
        Err(e) => eprintln!("Failed to hook malloc: {}", e),
    }

    let ptr = unsafe { libc::malloc(100) };
    println!("Allocated memory at: {:p}", ptr);

    if !ptr.is_null() {
        unsafe { libc::free(ptr); }
    }
}
