use substrate::hook_function;

// Basic inline hooking using the Rust `hook_function` API.
//
// Hooks a local function, intercepts calls, and forwards
// to the original via trampoline.

static mut ORIGINAL_FN: *mut u8 = std::ptr::null_mut();

extern "C" fn target_function(x: i32, y: i32) -> i32 {
    x + y
}

extern "C" fn replacement_function(x: i32, y: i32) -> i32 {
    let original: extern "C" fn(i32, i32) -> i32 = unsafe {
        std::mem::transmute(ORIGINAL_FN)
    };

    println!("[hook] intercepted: x={}, y={}", x, y);
    let result = original(x, y);
    println!("[hook] original returned: {}", result);
    result * 2
}

fn main() {
    println!("=== substrate::hook_function example ===\n");
    println!("before: target_function(3, 4) = {}", target_function(3, 4));

    unsafe {
        match hook_function(
            target_function as *mut u8,
            replacement_function as *mut u8,
            &raw mut ORIGINAL_FN as *mut *mut u8,
        ) {
            Ok(bytes) => println!("hook installed ({} bytes patched)\n", bytes),
            Err(e) => {
                eprintln!("hook failed: {}", e);
                return;
            }
        }
    }

    println!("after: target_function(3, 4) = {}", target_function(3, 4));
}
