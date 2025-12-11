# Cydia Substrate Rust Bindings

[![Crates.io](https://img.shields.io/crates/v/cydia-substrate)](https://crates.io/crates/cydia-substrate)
[![Documentation](https://docs.rs/cydia-substrate/badge.svg)](https://docs.rs/cydia-substrate)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Production-ready Rust bindings for **Cydia Substrate**, the powerful code insertion platform for iOS and Android. Hook functions, intercept calls, and modify behavior at runtime with a safe, ergonomic Rust API.

## Features

✨ **Complete Feature Set**
- 🎣 Function hooking (MSHookFunction)
- 🔍 Symbol finding and library loading
- 📱 iOS Objective-C message hooking
- 🤖 Android ELF symbol resolution
- 🛠️ Automatic FFI generation with bindgen
- 🔐 Memory-safe abstractions over unsafe C/C++

🚀 **Advanced Capabilities** (with features)
- 📋 x86/x64 instruction disassembler (HDE64)
- 🎯 Inline hooking with trampolines
- 🔗 Hook chains for multiple hooks
- 🏗️ ARM/x86 instruction generation
- 🐛 Debug utilities and hex dumping

🎯 **Cross-Platform Support**
- **iOS**: All versions, ARM/ARM64
- **Android**: All versions, ARM/ARM64/x86/x86_64
- **Architectures**: ARM, ARM64, x86, x86_64

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
cydia-substrate = "0.1"
```

Or with all features:

```toml
[dependencies]
cydia-substrate = { version = "0.1", features = ["advanced", "disassembler"] }
```

### Basic Example

```rust
use cydia_substrate::{substrate_hook, utils};

substrate_hook! {
    fn malloc(size: usize) -> *mut std::ffi::c_void {
        println!("Allocating {} bytes", size);
        unsafe { call_original_malloc(size) }
    }
}

fn main() {
    let addr = utils::get_absolute_address("libc.so.6", 0x123456).unwrap();
    install_malloc_hook(addr as *mut _).expect("Hook failed");
}
```

### Game Modding Example

```rust
use cydia_substrate::{substrate_hook, utils};

substrate_hook! {
    fn add_coins(amount: i32) -> i32 {
        let modified = amount * 100;
        println!("Coins: {} -> {}", amount, modified);
        unsafe { call_original_add_coins(modified) }
    }
}

fn main() {
    utils::wait_for_library("libgame.so", 30000);

    let addr = utils::get_absolute_address("libgame.so", 0xABCDEF).unwrap();
    install_add_coins_hook(addr as *mut _).expect("Hook failed");

    println!("Hook installed!");
}
```

## Feature Flags

```toml
[features]
default = []              # Core hooking functionality
debug = []                # Debug logging and utilities
substrate-internal = []   # Internal memory/process APIs
advanced = []             # Architecture-specific + inline hooks
disassembler = []         # HDE64 x86/x64 disassembler
```

## Platform-Specific Features

### Android
```rust
use cydia_substrate::android::{find_symbol_in_library, find_library_base};

let addr = find_symbol_in_library(0, "malloc", "libc.so")?;
let base = find_library_base(0, "libgame.so")?;
```

### iOS
```rust
use cydia_substrate::ios::ObjCHook;

let hook = ObjCHook::hook_message(class, selector, implementation)?;
```

## Building

### For Host Platform

```bash
cargo build --release
```

### Cross-Compilation

#### Android

```bash
cargo build --release --target aarch64-linux-android
cargo build --release --target armv7-linux-androideabi
```

#### iOS

```bash
cargo build --release --target aarch64-apple-ios
cargo build --release --target x86_64-apple-ios
```

## API Reference

### Core Functions

- `ms_hook_function`: Hook a function and get pointer to original
- `ms_get_image_by_name`: Get image reference by library name
- `ms_find_symbol`: Find symbol in an image
- `ms_hook_process`: Hook into another process

### Types

- `Image`: Safe wrapper for MSImageRef
- `Symbol<T>`: Type-safe symbol wrapper
- `Hook<F>`: Function hook wrapper

### Platform-Specific

- **iOS**: `ObjCHook` for Objective-C message hooking
- **Android**: Symbol finding utilities for ELF binaries

## License

MIT License

Original Cydia Substrate: GNU Lesser General Public License v3

## Safety

This library provides unsafe FFI bindings. Users must ensure:

- Hooked functions have correct signatures
- Memory safety when calling original functions
- Proper synchronization in multithreaded contexts

## Contributing

Contributions are welcome. Please ensure:

- Code compiles for all target platforms
- No runtime comments in production code
- Proper error handling
- Safe abstractions over unsafe code
