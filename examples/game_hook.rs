use cydia_substrate::{MSHookFunction, utils};
use std::ffi::c_void;

struct Offsets;

impl Offsets {
    const ADD_COIN: usize = 0x123456;
}

static mut OLD_ADD_COINS: Option<unsafe extern "C" fn(i32) -> i32> = None;

unsafe extern "C" fn add_coins_hook(amount: i32) -> i32 {
    println!("AddCoins hooked! Amount: {}", amount);

    let modified_amount = amount * 10;
    println!("Modified amount to: {}", modified_amount);

    if let Some(original) = OLD_ADD_COINS {
        original(modified_amount)
    } else {
        0
    }
}

fn main() {
    println!("Game Hook Example - Similar to C++ usage");

    println!("Waiting for libil2cpp.so to load...");
    if !utils::wait_for_library("libil2cpp.so", 30000) {
        eprintln!("Timeout waiting for libil2cpp.so");
        return;
    }

    println!("Library loaded! Setting up hooks...");

    match utils::get_absolute_address("libil2cpp.so", Offsets::ADD_COIN) {
        Some(absolute_addr) => {
            println!("Target address: 0x{:x}", absolute_addr);

            let target_ptr = absolute_addr as *mut c_void;
            let hook_ptr = add_coins_hook as *mut c_void;
            let mut original_ptr: *mut c_void = std::ptr::null_mut();

            unsafe {
                MSHookFunction(target_ptr, hook_ptr, &mut original_ptr as *mut *mut c_void);

                OLD_ADD_COINS = Some(std::mem::transmute(original_ptr));

                println!("Hook installed successfully!");
                println!("  Target:   {:p}", target_ptr);
                println!("  Hook:     {:p}", hook_ptr);
                println!("  Original: {:p}", original_ptr);
            }
        }
        None => {
            eprintln!("Failed to get absolute address for AddCoin");
        }
    }
}
