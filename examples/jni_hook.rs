// JNI-compatible shared library that hooks functions on load.
//
// Build:
//   cargo ndk --target aarch64-linux-android build --release --example jni_hook
//
// Load from Java/Kotlin:
//   System.loadLibrary("jni_hook")
//
// Demonstrates both MSHookFunction (all archs) and A64HookFunction (ARM64).

static mut ORIG_FUNC_A: *mut libc::c_void = std::ptr::null_mut();
static mut ORIG_FUNC_B: *mut libc::c_void = std::ptr::null_mut();

extern "C" fn replacement_a(arg: i32) -> i32 {
    let original: extern "C" fn(i32) -> i32 = unsafe {
        std::mem::transmute(ORIG_FUNC_A)
    };
    original(arg) + 1000
}

extern "C" fn replacement_b(arg: i32) -> i32 {
    let original: extern "C" fn(i32) -> i32 = unsafe {
        std::mem::transmute(ORIG_FUNC_B)
    };
    original(arg) * 2
}

// Called by Android when `System.loadLibrary("jni_hook")` is called.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn JNI_OnLoad(
    _vm: *mut libc::c_void,
    _reserved: *mut libc::c_void,
) -> libc::c_int {
    // --- Method 1: MSHookFunction (works on ALL architectures) ---
    //
    // The classic Cydia Substrate approach:
    // 1. Get a handle to the target library
    // 2. Resolve the symbol
    // 3. Hook with MSHookFunction

    let handle = unsafe { substrate::MSGetImageByName(c"libgame.so".as_ptr()) };
    if !handle.is_null() {
        let sym = unsafe { substrate::MSFindSymbol(handle, c"game_get_score".as_ptr()) };
        if !sym.is_null() {
            unsafe {
                substrate::MSHookFunction(
                    sym,
                    replacement_a as *mut libc::c_void,
                    &raw mut ORIG_FUNC_A as *mut *mut libc::c_void,
                );
            }
        }
    }

    // --- Method 2: A64HookFunction (AArch64-only, from And64InlineHook) ---
    //
    // Better trampoline pool management on ARM64.

    #[cfg(target_arch = "aarch64")]
    {
        let pid = std::process::id() as i32;
        if let Ok(addr) = substrate::find_symbol(pid, "game_get_health", "libgame.so") {
            unsafe {
                substrate::A64HookFunction(
                    addr as *mut libc::c_void,
                    replacement_b as *mut libc::c_void,
                    &raw mut ORIG_FUNC_B as *mut *mut libc::c_void,
                );
            }
        }
    }

    // On non-AArch64, fall back to MSHookFunction
    #[cfg(not(target_arch = "aarch64"))]
    {
        let pid = std::process::id() as i32;
        if let Ok(addr) = substrate::find_symbol(pid, "game_get_health", "libgame.so") {
            unsafe {
                substrate::MSHookFunction(
                    addr as *mut libc::c_void,
                    replacement_b as *mut libc::c_void,
                    &raw mut ORIG_FUNC_B as *mut *mut libc::c_void,
                );
            }
        }
    }

    0x00010006 // JNI_VERSION_1_6
}

fn main() {
    println!("This example is designed for Android JNI usage.");
    println!("Build: cargo ndk --target aarch64-linux-android build --release --example jni_hook");
    println!("Load:  System.loadLibrary(\"jni_hook\")");
    println!();
    println!("Demonstrates both MSHookFunction (all archs) and A64HookFunction (ARM64).");
}
