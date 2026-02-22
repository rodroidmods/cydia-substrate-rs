// Demonstrates the AArch64-specific A64HookFunction and A64HookFunctionV APIs.
//
// These are the And64InlineHook APIs, available only on aarch64.
//
// A64HookFunction  — auto-allocates from the internal trampoline pool (256 slots).
// A64HookFunctionV — caller provides their own RWX buffer for the trampoline.
//
// Build:
//   cargo ndk --target aarch64-linux-android build --release --example a64_hook

// ---- Example 1: A64HookFunction (auto-pooled trampoline) ----

#[cfg(target_arch = "aarch64")]
static mut ORIG_ADD: *mut libc::c_void = std::ptr::null_mut();

#[cfg(target_arch = "aarch64")]
extern "C" fn target_add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(target_arch = "aarch64")]
extern "C" fn hooked_add(a: i32, b: i32) -> i32 {
    let original: extern "C" fn(i32, i32) -> i32 = unsafe {
        std::mem::transmute(ORIG_ADD)
    };
    let result = original(a, b);
    println!("[A64Hook] original({}, {}) = {}", a, b, result);
    result * 10
}

// ---- Example 2: A64HookFunctionV (user-provided RWX buffer) ----

#[cfg(target_arch = "aarch64")]
extern "C" fn target_mul(a: i32, b: i32) -> i32 {
    a * b
}

#[cfg(target_arch = "aarch64")]
static mut ORIG_MUL: *mut libc::c_void = std::ptr::null_mut();

#[cfg(target_arch = "aarch64")]
extern "C" fn hooked_mul(a: i32, b: i32) -> i32 {
    let original: extern "C" fn(i32, i32) -> i32 = unsafe {
        std::mem::transmute(ORIG_MUL)
    };
    let result = original(a, b);
    println!("[A64HookV] original({}, {}) = {}", a, b, result);
    result + 999
}

fn main() {
    #[cfg(target_arch = "aarch64")]
    {
        println!("=== A64HookFunction example (AArch64) ===\n");

        // --- A64HookFunction: uses internal trampoline pool ---
        println!("[1] A64HookFunction (auto-pooled trampoline)");
        println!("    before: target_add(3, 7) = {}", target_add(3, 7));

        unsafe {
            substrate::A64HookFunction(
                target_add as *mut libc::c_void,
                hooked_add as *mut libc::c_void,
                &raw mut ORIG_ADD as *mut *mut libc::c_void,
            );
        }

        println!("    after:  target_add(3, 7) = {}", target_add(3, 7));
        let trampoline = unsafe { ORIG_ADD };
        println!("    trampoline at: {:p}", trampoline);
        println!("    (expected: (3+7)*10 = 100)\n");

        // --- A64HookFunctionV: caller-provided RWX buffer ---
        println!("[2] A64HookFunctionV (user-provided RWX buffer)");
        println!("    before: target_mul(4, 5) = {}", target_mul(4, 5));

        let rwx_size: usize = 50 * std::mem::size_of::<u32>();
        let rwx = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                rwx_size,
                libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        assert!(!rwx.is_null() && rwx != libc::MAP_FAILED, "mmap failed");

        let trampoline = unsafe {
            substrate::A64HookFunctionV(
                target_mul as *mut libc::c_void,
                hooked_mul as *mut libc::c_void,
                rwx,
                rwx_size,
            )
        };
        unsafe { ORIG_MUL = trampoline; }

        if !trampoline.is_null() {
            println!("    hook installed, trampoline at: {:p}", trampoline);
            println!("    after:  target_mul(4, 5) = {}", target_mul(4, 5));
            println!("    (expected: (4*5)+999 = 1019)");
        } else {
            println!("    hook failed!");
        }

        unsafe { libc::munmap(rwx, rwx_size); }
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        println!("A64HookFunction is only available on AArch64.");
        println!("Build with: cargo ndk --target aarch64-linux-android build --release --example a64_hook");
    }
}
