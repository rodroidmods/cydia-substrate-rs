# substrate

**Pure Rust rewrite of [Cydia Substrate](http://www.cydiasubstrate.com/) + [And64InlineHook](https://github.com/jbro129/Unity-Substrate-Hook-Android)** — cross-architecture inline function hooking for Android.

[![License: LGPL-3.0](https://img.shields.io/badge/license-LGPL--3.0-blue.svg)](LICENSE)

## Features

- **Multi-architecture**: ARM32, Thumb, AArch64, x86, and x86-64
- **Drop-in C compatibility**: Exports `MSHookFunction`, `A64HookFunction`, and other familiar symbols
- **Safe Rust API**: Idiomatic wrappers alongside raw `extern "C"` FFI
- **Zero runtime deps**: Only `libc` for syscalls
- **Production-ready**: Full trampoline generation with PC-relative instruction relocation

## Supported Architectures

| Architecture | Hooking Engine | Instructions Relocated |
|---|---|---|
| **ARM32** | Substrate | PC-relative LDR |
| **Thumb** | Substrate | LDR, B, BL, CBZ, LDRW, ADD |
| **AArch64** | And64InlineHook | B, BL, B.cond, CBZ, CBNZ, TBZ, TBNZ, LDR literal, ADR, ADRP |
| **x86** | Substrate + HDE64 | CALL, JMP, Jcc |
| **x86-64** | Substrate + HDE64 | RIP-relative MOV, CALL, JMP, Jcc |

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
substrate = { git = "https://github.com/rodroidmods/cydia-substrate-rs" }
```

## Examples

### 1. `MSHookFunction` — The Classic Cydia Substrate API

The drop-in C-compatible API that works on **all architectures**:

```rust
extern "C" {
    fn MSHookFunction(symbol: *mut libc::c_void, replace: *mut libc::c_void, result: *mut *mut libc::c_void);
    fn MSGetImageByName(file: *const libc::c_char) -> *mut libc::c_void;
    fn MSFindSymbol(image: *mut libc::c_void, name: *const libc::c_char) -> *mut libc::c_void;
}

static mut ORIGINAL: *mut libc::c_void = std::ptr::null_mut();

extern "C" fn target(x: i32) -> i32 { x + 1 }
extern "C" fn hook(x: i32) -> i32 {
    let orig: extern "C" fn(i32) -> i32 = unsafe { std::mem::transmute(ORIGINAL) };
    orig(x) * 2
}

fn main() {
    // Hook a local function
    unsafe {
        MSHookFunction(
            target as *mut libc::c_void,
            hook as *mut libc::c_void,
            &mut ORIGINAL as *mut *mut libc::c_void,
        );
    }
    
    // Hook a library function by name
    let handle = unsafe { MSGetImageByName(c"libtarget.so".as_ptr()) };
    let sym = unsafe { MSFindSymbol(handle, c"target_func".as_ptr()) };
    unsafe { MSHookFunction(sym, hook as _, &mut ORIGINAL as _); }
}
```

### 2. `A64HookFunction` — AArch64-Specific API (And64InlineHook)

Optimized for ARM64 with an internal trampoline pool (256 slots):

```rust
#[cfg(target_arch = "aarch64")]
extern "C" {
    fn A64HookFunction(symbol: *mut libc::c_void, replace: *mut libc::c_void, result: *mut *mut libc::c_void);
    fn A64HookFunctionV(symbol: *mut libc::c_void, replace: *mut libc::c_void, rwx: *mut libc::c_void, rwx_size: usize) -> *mut libc::c_void;
}

static mut ORIG: *mut libc::c_void = std::ptr::null_mut();

extern "C" fn target(a: i32, b: i32) -> i32 { a + b }
extern "C" fn hook(a: i32, b: i32) -> i32 {
    let orig: extern "C" fn(i32, i32) -> i32 = unsafe { std::mem::transmute(ORIG) };
    orig(a, b) * 10
}

fn main() {
    // Auto-pooled trampoline
    unsafe { A64HookFunction(target as _, hook as _, &mut ORIG as _); }

    // Or provide your own RWX buffer
    let rwx = unsafe { libc::mmap(std::ptr::null_mut(), 200, 7, 0x22, -1, 0) };
    let trampoline = unsafe { A64HookFunctionV(target as _, hook as _, rwx, 200) };
}
```

### 3. Rust API — Idiomatic Wrapper

```rust
use substrate::{hook_function, find_symbol, find_lib_base};

static mut ORIG: *mut u8 = std::ptr::null_mut();

extern "C" fn hook(x: i32) -> i32 {
    let orig: extern "C" fn(i32) -> i32 = unsafe { std::mem::transmute(ORIG) };
    orig(x) + 100
}

fn main() {
    let pid = std::process::id() as i32;
    
    // Find a symbol in a loaded library
    if let Ok(addr) = find_symbol(pid, "target_func", "libtarget.so") {
        unsafe {
            let _ = hook_function(addr as *mut u8, hook as *mut u8, &mut ORIG as _);
        }
    }
    
    // Get library base address
    if let Ok(base) = find_lib_base(pid, "libc.so") {
        println!("libc @ 0x{:x}", base);
    }
}
```

### 4. Android JNI — Game Modding Pattern

```rust
extern "C" {
    fn MSHookFunction(symbol: *mut libc::c_void, replace: *mut libc::c_void, result: *mut *mut libc::c_void);
    fn MSGetImageByName(file: *const libc::c_char) -> *mut libc::c_void;
    fn MSFindSymbol(image: *mut libc::c_void, name: *const libc::c_char) -> *mut libc::c_void;
}

static mut ORIG: *mut libc::c_void = std::ptr::null_mut();

extern "C" fn hooked_get_score(arg: i32) -> i32 {
    let orig: extern "C" fn(i32) -> i32 = unsafe { std::mem::transmute(ORIG) };
    orig(arg) + 99999
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn JNI_OnLoad(_vm: *mut libc::c_void, _: *mut libc::c_void) -> i32 {
    let handle = unsafe { MSGetImageByName(c"libil2cpp.so".as_ptr()) };
    let sym = unsafe { MSFindSymbol(handle, c"GameManager_GetScore".as_ptr()) };
    unsafe { MSHookFunction(sym, hooked_get_score as _, &mut ORIG as _); }
    0x00010006 // JNI_VERSION_1_6
}
```

> **Full runnable examples** in the [`examples/`](examples/) directory:
> - `basic_hook.rs` — Rust API basics
> - `ms_hook.rs` — MSHookFunction + MSGetImageByName + MSFindSymbol
> - `a64_hook.rs` — A64HookFunction + A64HookFunctionV (AArch64)
> - `jni_hook.rs` — Android JNI with both MS + A64 APIs
> - `symbol_lookup.rs` — find_symbol / find_lib_base
> - `hook_library_function.rs` — Hooking libc's open()

## Building

```bash
# Install targets
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# Install cargo-ndk
cargo install cargo-ndk

# Build for AArch64
cargo ndk --target aarch64-linux-android build --release

# Build all targets
cargo ndk --target aarch64-linux-android \
          --target armv7-linux-androideabi \
          --target x86_64-linux-android \
          --target i686-linux-android \
          build --release
```

Output: `target/<triple>/release/libsubstrate.so`

## C FFI Exports

| Symbol | Architectures | Signature |
|---|---|---|
| `MSHookFunction` | All | `void(void*, void*, void**)` |
| `MSGetImageByName` | All | `void*(const char*)` |
| `MSFindSymbol` | All | `void*(void*, const char*)` |
| `MSGetInstructionWidth` | All | `size_t(void*)` |
| `A64HookFunction` | AArch64 | `void(void*, void*, void**)` |
| `A64HookFunctionV` | AArch64 | `void*(void*, void*, void*, size_t)` |

## How It Works

```
Before:                          After:
┌──────────────┐                 ┌──────────────┐
│ original()   │                 │ JMP replace  │──→ replace()
│ prologue...  │                 │ ...          │
│ body...      │                 │ body...      │
└──────────────┘                 └──────────────┘
                                        ↑
                                 ┌──────────────┐
                                 │ trampoline   │
                                 │ relocated    │
                                 │ prologue...  │
                                 │ JMP original │──→ original+N
                                 └──────────────┘
```

## Credits

- [Rodroid Mods](https://t.me/rodroidmods) by Rodroid Mods

## License

[GNU Lesser General Public License v3.0](LICENSE)
