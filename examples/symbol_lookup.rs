use substrate::{find_symbol, find_lib_base};
use std::process;

// Demonstrates how to find symbols and library base addresses
// by parsing /proc/self/maps and ELF symbol tables.
//
// This is useful for hooking functions in shared libraries
// loaded into the current process.

fn main() {
    let pid = process::id() as i32;
    println!("=== substrate symbol lookup example ===");
    println!("PID: {}\n", pid);

    // Find the base address of libc
    match find_lib_base(pid, "libc.so") {
        Ok(base) => println!("libc.so base address: 0x{:x}", base),
        Err(e) => println!("failed to find libc.so: {}", e),
    }

    // Find specific symbols in libc
    let symbols = ["malloc", "free", "open", "close", "mmap"];
    for name in &symbols {
        match find_symbol(pid, name, "libc.so") {
            Ok(addr) => println!("  {} @ 0x{:x}", name, addr),
            Err(e) => println!("  {} — not found: {}", name, e),
        }
    }

    println!("\n--- other libraries ---");

    // Find base of linker
    match find_lib_base(pid, "linker") {
        Ok(base) => println!("linker base: 0x{:x}", base),
        Err(e) => println!("linker: {}", e),
    }

    match find_lib_base(pid, "libm.so") {
        Ok(base) => {
            println!("libm.so base: 0x{:x}", base);
            for name in &["sin", "cos", "sqrt", "pow"] {
                match find_symbol(pid, name, "libm.so") {
                    Ok(addr) => println!("  {} @ 0x{:x}", name, addr),
                    Err(e) => println!("  {} — {}", name, e),
                }
            }
        }
        Err(e) => println!("libm.so: {}", e),
    }
}
