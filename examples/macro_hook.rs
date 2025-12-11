use cydia_substrate::{define_hook, ms_hook_symbol, Image};
use std::ffi::c_void;

define_hook! {
    fn strlen(s: *const libc::c_char) -> libc::size_t
}

unsafe extern "C" fn hooked_strlen(s: *const libc::c_char) -> libc::size_t {
    let result = strlen_original(s);
    println!("strlen called on string of length: {}", result);
    result
}

fn main() {
    println!("Macro-based Hook Example");

    let image = Image::by_name("libc.so.6").expect("Failed to load libc");

    match ms_hook_symbol!(image.handle(), "strlen", install_strlen) {
        Ok(_) => {
            unsafe {
                install_strlen(
                    cydia_substrate::ms_find_symbol(image.handle(), "strlen"),
                    hooked_strlen
                ).expect("Failed to install hook");
            }
            println!("strlen hooked successfully!");
        }
        Err(e) => eprintln!("Failed: {}", e),
    }

    let test_str = std::ffi::CString::new("Hello, World!").unwrap();
    let len = unsafe { libc::strlen(test_str.as_ptr()) };
    println!("Result: {}", len);
}
