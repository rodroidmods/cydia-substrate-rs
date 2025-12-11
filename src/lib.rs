//! # Cydia Substrate Rust Bindings
//!
//! Production-ready Rust bindings for **Cydia Substrate**, enabling function hooking
//! and code injection on iOS and Android platforms.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use cydia_substrate::{substrate_hook, utils};
//!
//! substrate_hook! {
//!     fn malloc(size: usize) -> *mut std::ffi::c_void {
//!         println!("Allocating {} bytes", size);
//!         unsafe { call_original_malloc(size) }
//!     }
//! }
//!
//! fn main() {
//!     let addr = utils::get_absolute_address("libc.so.6", 0x123456).unwrap();
//!     install_malloc_hook(addr as *mut _).expect("Hook failed");
//! }
//! ```
//!
//! ## Features
//!
//! - `default`: Core hooking functionality
//! - `debug`: Debug logging and utilities
//! - `advanced`: Architecture-specific APIs and inline hooks
//! - `disassembler`: x86/x64 instruction disassembler
//! - `substrate-internal`: Internal memory/process management
//!
//! ## Android ARM64 Inline Hooking
//!
//! On Android ARM64, the `and64` module provides additional inline hooking capabilities:
//!
//! ```rust,no_run
//! #[cfg(all(target_arch = "aarch64", target_os = "android"))]
//! use cydia_substrate::and64::a64_hook_function;
//!
//! #[cfg(all(target_arch = "aarch64", target_os = "android"))]
//! let original = a64_hook_function(symbol_ptr, hook_ptr).expect("Hook failed");
//! ```
//!
//! ## Safety
//!
//! This library provides `unsafe` FFI bindings. Users must ensure correct function
//! signatures, memory safety, and proper synchronization.

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::ffi::c_void;
use std::os::raw::{c_char, c_int};

pub mod sys;
pub mod ffi;
pub mod hook;
pub mod image;
pub mod symbol;
pub mod macros;
pub mod memory;
pub mod debug;
pub mod utils;

#[cfg(feature = "advanced")]
pub mod advanced;

#[cfg(feature = "advanced")]
pub mod arch;

#[cfg(all(feature = "disassembler", any(target_arch = "x86_64", target_arch = "x86")))]
pub mod disasm;

pub use hook::*;
pub use image::*;
pub use symbol::*;

#[cfg(test)]
mod tests;

#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "ios")]
pub mod ios;

#[cfg(all(target_arch = "aarch64", target_os = "android"))]
pub mod and64;

/// Opaque reference to a loaded library image.
pub type MSImageRef = *const c_void;

/// Opaque structure representing a process for internal use.
#[repr(C)]
pub struct SubstrateProcess {
    _private: [u8; 0],
}

/// Opaque structure representing memory for internal use.
#[repr(C)]
pub struct SubstrateMemory {
    _private: [u8; 0],
}

/// Reference to a SubstrateProcess.
pub type SubstrateProcessRef = *mut SubstrateProcess;

/// Reference to SubstrateMemory.
pub type SubstrateMemoryRef = *mut SubstrateMemory;

/// Reference to a custom allocator.
pub type SubstrateAllocatorRef = *mut c_void;

// Low-level FFI bindings to Cydia Substrate C library.
#[link(name = "substrate", kind = "static")]
extern "C" {
    /// Hook into another process by injecting a library.
    ///
    /// # Safety
    /// Requires proper permissions and valid PID/library path.
    pub fn MSHookProcess(pid: c_int, library: *const c_char) -> bool;

    /// Get a reference to a loaded library by name.
    ///
    /// # Safety
    /// `file` must be a valid null-terminated C string.
    pub fn MSGetImageByName(file: *const c_char) -> MSImageRef;

    /// Find a symbol within a library image.
    ///
    /// # Safety
    /// `image` and `name` must be valid. Pass null for `image` to search globally.
    pub fn MSFindSymbol(image: MSImageRef, name: *const c_char) -> *mut c_void;

    /// Hook a function, redirecting calls to a replacement.
    ///
    /// # Safety
    /// All pointers must be valid. `symbol` and `replace` must have matching signatures.
    pub fn MSHookFunction(symbol: *mut c_void, replace: *mut c_void, result: *mut *mut c_void);
}

#[cfg(feature = "debug")]
extern "C" {
    pub static mut MSDebug: bool;
}

/// Hook into another process by injecting a library.
///
/// # Arguments
/// * `pid` - Process ID to inject into
/// * `library` - Path to library to inject
///
/// # Returns
/// `true` if successful, `false` otherwise
///
/// # Example
/// ```no_run
/// use cydia_substrate::ms_hook_process;
/// let success = ms_hook_process(1234, "/path/to/lib.so");
/// ```
pub fn ms_hook_process(pid: i32, library: &str) -> bool {
    let c_library = std::ffi::CString::new(library).unwrap();
    unsafe { MSHookProcess(pid, c_library.as_ptr()) }
}

/// Get a reference to a loaded library by name.
///
/// # Arguments
/// * `file` - Name of the library (e.g., "libc.so.6")
///
/// # Returns
/// Reference to the loaded library, or null if not found
///
/// # Example
/// ```no_run
/// use cydia_substrate::ms_get_image_by_name;
/// let image = ms_get_image_by_name("libc.so.6");
/// ```
pub fn ms_get_image_by_name(file: &str) -> MSImageRef {
    let c_file = std::ffi::CString::new(file).unwrap();
    unsafe { MSGetImageByName(c_file.as_ptr()) }
}

/// Find a symbol within a library image.
///
/// # Arguments
/// * `image` - Library reference (use null for global search)
/// * `name` - Symbol name to find
///
/// # Returns
/// Pointer to symbol, or null if not found
///
/// # Example
/// ```no_run
/// use cydia_substrate::{ms_get_image_by_name, ms_find_symbol};
/// let image = ms_get_image_by_name("libc.so.6");
/// let symbol = ms_find_symbol(image, "malloc");
/// ```
pub fn ms_find_symbol(image: MSImageRef, name: &str) -> *mut c_void {
    let c_name = std::ffi::CString::new(name).unwrap();
    unsafe { MSFindSymbol(image, c_name.as_ptr()) }
}

/// Hook a function with safe error handling.
///
/// # Arguments
/// * `symbol` - Address of function to hook
/// * `replace` - Address of replacement function
///
/// # Returns
/// `Ok(original)` with pointer to original function, or `Err` if invalid
///
/// # Safety
/// Function signatures of `symbol` and `replace` must match.
///
/// # Example
/// ```no_run
/// use cydia_substrate::ms_hook_function;
/// use std::ffi::c_void;
///
/// unsafe extern "C" fn my_hook() {}
///
/// let original = ms_hook_function(
///     symbol_addr as *mut c_void,
///     my_hook as *mut c_void
/// ).expect("Hook failed");
/// ```
pub fn ms_hook_function(
    symbol: *mut c_void,
    replace: *mut c_void,
) -> Result<*mut c_void, &'static str> {
    if symbol.is_null() {
        return Err("Symbol pointer is null");
    }
    if replace.is_null() {
        return Err("Replace pointer is null");
    }

    let mut result: *mut c_void = std::ptr::null_mut();
    unsafe {
        MSHookFunction(symbol, replace, &mut result as *mut *mut c_void);
    }
    Ok(result)
}

#[cfg(feature = "debug")]
pub fn set_debug_mode(enabled: bool) {
    unsafe {
        MSDebug = enabled;
    }
}
