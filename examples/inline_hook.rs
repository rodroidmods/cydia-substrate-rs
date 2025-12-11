use cydia_substrate::advanced::{InlineHook, Trampoline};
use cydia_substrate::Image;
use std::ffi::c_void;

fn main() {
    println!("Inline Hook Example");

    let image = Image::by_name("libc.so.6").expect("Failed to load libc");
    let target = cydia_substrate::ms_find_symbol(image.handle(), "puts");

    if target.is_null() {
        eprintln!("Failed to find puts symbol");
        return;
    }

    println!("Target function at: {:p}", target);

    let mut hook = InlineHook::new(target, 16).expect("Failed to create inline hook");
    println!("Original bytes: {:?}", hook.original_bytes());

    match hook.create_trampoline() {
        Ok(trampoline) => {
            println!("Trampoline created at: {:p}", trampoline);
        }
        Err(e) => {
            eprintln!("Failed to create trampoline: {}", e);
        }
    }
}
