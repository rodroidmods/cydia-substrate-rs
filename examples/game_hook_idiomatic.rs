use cydia_substrate::{substrate_hook, utils, ms_hook_function};
use std::ffi::c_void;

mod offsets {
    pub const ADD_COIN: usize = 0x123456;
    pub const REMOVE_COIN: usize = 0x789ABC;
    pub const GET_COIN_COUNT: usize = 0xDEF012;
}

substrate_hook! {
    fn add_coins(amount: i32) -> i32 {
        let modified = amount * 10;
        println!("AddCoins: {} -> {}", amount, modified);

        unsafe { call_original_add_coins(modified) }
    }
}

substrate_hook! {
    fn remove_coins(amount: i32) -> i32 {
        println!("RemoveCoins blocked: {}", amount);
        0
    }
}

fn hook_game_function(lib: &str, offset: usize, hook: *mut c_void) -> Result<*mut c_void, String> {
    let addr = utils::get_absolute_address(lib, offset)
        .ok_or_else(|| format!("Failed to get address for offset 0x{:x}", offset))?;

    ms_hook_function(addr as *mut c_void, hook)
        .map_err(|e| e.to_string())
}

fn main() {
    println!("Idiomatic Rust Game Hook Example\n");

    if !utils::wait_for_library("libil2cpp.so", 30000) {
        eprintln!("Failed to load libil2cpp.so");
        return;
    }

    println!("Library loaded!\n");

    match install_add_coins_hook(
        utils::get_absolute_address("libil2cpp.so", offsets::ADD_COIN)
            .unwrap() as *mut c_void
    ) {
        Ok(_) => println!("✓ AddCoins hooked"),
        Err(e) => eprintln!("✗ AddCoins failed: {}", e),
    }

    match install_remove_coins_hook(
        utils::get_absolute_address("libil2cpp.so", offsets::REMOVE_COIN)
            .unwrap() as *mut c_void
    ) {
        Ok(_) => println!("✓ RemoveCoins hooked"),
        Err(e) => eprintln!("✗ RemoveCoins failed: {}", e),
    }

    println!("\nAll hooks installed successfully!");
}
