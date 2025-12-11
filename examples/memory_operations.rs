use cydia_substrate::memory::MemoryProtection;
use cydia_substrate::debug::Debug;
use std::ffi::c_void;

fn main() {
    println!("Memory Operations Example");

    let mut buffer = vec![0u8; 4096];
    let ptr = buffer.as_mut_ptr() as *mut c_void;

    println!("\nBuffer allocated at: {:p}", ptr);
    println!("Buffer size: {} bytes", buffer.len());

    match MemoryProtection::make_writable(ptr, buffer.len()) {
        Ok(_) => println!("Memory protection changed to RWX"),
        Err(e) => eprintln!("Failed to change protection: {}", e),
    }

    buffer[0] = 0x90;
    buffer[1] = 0x90;
    buffer[2] = 0xC3;

    Debug::print_memory(ptr, 16, Some("Modified Buffer"));

    match MemoryProtection::make_executable(ptr, buffer.len()) {
        Ok(_) => println!("\nMemory protection changed to RX (executable)"),
        Err(e) => eprintln!("Failed to make executable: {}", e),
    }

    println!("\nMemory operations completed successfully");
}
