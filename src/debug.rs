use std::ffi::c_void;

#[cfg(feature = "debug")]
extern "C" {
    pub static mut MSDebug: bool;
}

pub struct Debug;

impl Debug {
    #[cfg(feature = "debug")]
    pub fn enable() {
        unsafe {
            MSDebug = true;
        }
    }

    #[cfg(feature = "debug")]
    pub fn disable() {
        unsafe {
            MSDebug = false;
        }
    }

    #[cfg(feature = "debug")]
    pub fn is_enabled() -> bool {
        unsafe { MSDebug }
    }

    #[cfg(not(feature = "debug"))]
    pub fn enable() {}

    #[cfg(not(feature = "debug"))]
    pub fn disable() {}

    #[cfg(not(feature = "debug"))]
    pub fn is_enabled() -> bool {
        false
    }

    pub fn print_hex(data: &[u8]) {
        for (i, chunk) in data.chunks(16).enumerate() {
            print!("{:04x}: ", i * 16);
            for byte in chunk {
                print!("{:02x} ", byte);
            }
            for _ in 0..(16 - chunk.len()) {
                print!("   ");
            }
            print!(" |");
            for &byte in chunk {
                let ch = if byte >= 0x20 && byte < 0x80 {
                    byte as char
                } else {
                    '.'
                };
                print!("{}", ch);
            }
            println!("|");
        }
    }

    pub fn print_memory(addr: *const c_void, size: usize, label: Option<&str>) {
        if let Some(l) = label {
            println!("Memory dump [{}] at {:p}:", l, addr);
        } else {
            println!("Memory dump at {:p}:", addr);
        }

        if addr.is_null() {
            println!("  (null pointer)");
            return;
        }

        unsafe {
            let slice = std::slice::from_raw_parts(addr as *const u8, size);
            Self::print_hex(slice);
        }
    }

    pub fn print_pointer_info(ptr: *const c_void, name: &str) {
        println!("{}: {:p}", name, ptr);
    }

    pub fn print_hook_info(symbol: *const c_void, replacement: *const c_void, original: *const c_void) {
        println!("Hook installed:");
        println!("  Symbol:      {:p}", symbol);
        println!("  Replacement: {:p}", replacement);
        println!("  Original:    {:p}", original);
    }
}

#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug")]
        {
            if $crate::debug::Debug::is_enabled() {
                println!($($arg)*);
            }
        }
    };
}

#[macro_export]
macro_rules! debug_hex {
    ($data:expr) => {
        #[cfg(feature = "debug")]
        {
            if $crate::debug::Debug::is_enabled() {
                $crate::debug::Debug::print_hex($data);
            }
        }
    };
}

#[macro_export]
macro_rules! debug_memory {
    ($addr:expr, $size:expr) => {
        #[cfg(feature = "debug")]
        {
            if $crate::debug::Debug::is_enabled() {
                $crate::debug::Debug::print_memory($addr, $size, None);
            }
        }
    };
    ($addr:expr, $size:expr, $label:expr) => {
        #[cfg(feature = "debug")]
        {
            if $crate::debug::Debug::is_enabled() {
                $crate::debug::Debug::print_memory($addr, $size, Some($label));
            }
        }
    };
}
