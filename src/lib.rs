//! # substrate
//!
//! Pure Rust rewrite of **Cydia Substrate** + **And64InlineHook** — cross-architecture
//! inline function hooking for Android.
//!
//! ## Supported Architectures
//!
//! | Architecture | Engine |
//! |---|---|
//! | ARM32 | Substrate (PC-relative LDR relocation) |
//! | Thumb | Substrate (LDR, B, BL, CBZ, LDRW, ADD) |
//! | AArch64 | And64InlineHook (B, BL, B.cond, CBZ, TBZ, LDR, ADR, ADRP) |
//! | x86 | Substrate + HDE64 |
//! | x86-64 | Substrate + HDE64 (RIP-relative) |
//!
//! ## Quick Start
//!
//! ```no_run
//! use substrate::hook_function;
//!
//! static mut ORIGINAL: *mut u8 = std::ptr::null_mut();
//!
//! extern "C" fn target(x: i32) -> i32 { x + 1 }
//! extern "C" fn hook(x: i32) -> i32 {
//!     let orig: extern "C" fn(i32) -> i32 = unsafe { std::mem::transmute(ORIGINAL) };
//!     orig(x) * 2
//! }
//!
//! unsafe {
//!     let _ = hook_function(
//!         target as *mut u8,
//!         hook as *mut u8,
//!         &mut ORIGINAL as *mut *mut u8,
//!     );
//! }
//! ```

pub mod arch;
pub mod buffer;
pub mod debug;
pub mod error;
pub mod hde64;
pub mod log;
pub mod memory;
pub mod symbol;

use error::Result;

/// Inline-hooks a function, redirecting calls from `symbol` to `replace`.
///
/// If `result` is non-null, a trampoline is allocated and written there.
/// Calling through the trampoline invokes the **original** function.
///
/// # Safety
///
/// - `symbol` must point to the start of an executable function.
/// - `replace` must have the same calling convention and signature.
/// - `result` (if non-null) must point to a valid `*mut u8` location.
/// - The caller must ensure no other thread is executing the patched prologue
///   during hook installation.
pub unsafe fn hook_function(
    symbol: *mut u8,
    replace: *mut u8,
    result: *mut *mut u8,
) -> Result<usize> {
    unsafe { arch::hook_function(symbol, replace, result) }
}

/// Finds a symbol address by name in a loaded shared library.
///
/// Parses `/proc/{pid}/maps` and the library's ELF symbol table to resolve
/// the address.
///
/// # Arguments
///
/// - `pid` — Process ID (use `std::process::id() as i32` for self)
/// - `name` — Symbol name (e.g., `"open"`)
/// - `lib_name` — Library name (e.g., `"libc.so"`)
pub fn find_symbol(pid: i32, name: &str, lib_name: &str) -> Result<usize> {
    symbol::find_name(pid, name, lib_name)
}

/// Returns the base address of a loaded shared library.
///
/// Parses `/proc/{pid}/maps` to find the mapping.
pub fn find_lib_base(pid: i32, lib_name: &str) -> Result<usize> {
    symbol::find_libbase(pid, lib_name)
}

/// Returns a handle to a loaded shared library.
///
/// Wrapper around `dlopen` with `RTLD_NOLOAD | RTLD_LAZY` — will not load
/// a new library, only return a handle if already loaded.
///
/// # Safety
///
/// `file` must be a valid null-terminated C string.
pub unsafe fn get_image_by_name(file: *const libc::c_char) -> *mut libc::c_void {
    unsafe { libc::dlopen(file, libc::RTLD_NOLOAD | libc::RTLD_LAZY) }
}

/// Resolves a symbol from a library handle.
///
/// Wrapper around `dlsym`.
///
/// # Safety
///
/// - `image` must be a valid library handle from [`get_image_by_name`].
/// - `name` must be a valid null-terminated C string.
pub unsafe fn ms_find_symbol(
    image: *mut libc::c_void,
    name: *const libc::c_char,
) -> *mut libc::c_void {
    unsafe { libc::dlsym(image, name) }
}

/// Returns the byte width of the instruction at `start`.
///
/// - **ARM32/AArch64**: Returns 4 (ARM) or 2/4 (Thumb, based on encoding).
/// - **x86/x86-64**: Uses the HDE64 disassembler to decode instruction length.
pub fn get_instruction_width(start: *const u8) -> usize {
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        let addr = start as usize;
        if addr & 1 == 0 {
            4
        } else {
            let thumb = (addr & !1) as *const u16;
            let ic = unsafe { *thumb };
            if (ic & 0xe000) == 0xe000 && (ic & 0x1800) != 0x0000 {
                4
            } else {
                2
            }
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let mut decode = hde64::Hde64::default();
        unsafe { hde64::hde64_disasm(start, &mut decode) as usize }
    }

    #[cfg(not(any(
        target_arch = "arm",
        target_arch = "aarch64",
        target_arch = "x86",
        target_arch = "x86_64"
    )))]
    {
        0
    }
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn MSHookFunction(
    symbol: *mut libc::c_void,
    replace: *mut libc::c_void,
    result: *mut *mut libc::c_void,
) {
    let _ = unsafe {
        hook_function(
            symbol as *mut u8,
            replace as *mut u8,
            result as *mut *mut u8,
        )
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn MSGetImageByName(file: *const libc::c_char) -> *mut libc::c_void {
    unsafe { get_image_by_name(file) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn MSFindSymbol(
    image: *mut libc::c_void,
    name: *const libc::c_char,
) -> *mut libc::c_void {
    unsafe { ms_find_symbol(image, name) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn MSGetInstructionWidth(start: *mut libc::c_void) -> usize {
    get_instruction_width(start as *const u8)
}

#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn A64HookFunction(
    symbol: *mut libc::c_void,
    replace: *mut libc::c_void,
    result: *mut *mut libc::c_void,
) {
    let _ = unsafe {
        arch::aarch64::hook_function_aarch64(
            symbol as *mut u8,
            replace as *mut u8,
            result as *mut *mut u8,
        )
    };
}

#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn A64HookFunctionV(
    symbol: *mut libc::c_void,
    replace: *mut libc::c_void,
    rwx: *mut libc::c_void,
    rwx_size: usize,
) -> *mut libc::c_void {
    unsafe {
        arch::aarch64::a64_hook_function_v(
            symbol as *mut u8,
            replace as *mut u8,
            rwx as *mut u8,
            rwx_size,
        ) as *mut libc::c_void
    }
}
