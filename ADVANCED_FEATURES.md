## Advanced Features Guide

This document explains all advanced features and how to use them compared to C/C++.

### Automatic FFI Generation with Bindgen

The bindings are automatically generated from C headers using bindgen. The generated `sys` module contains raw FFI bindings.

```rust
use cydia_substrate::sys::*;
```

### Feature Flags

Enable advanced features in `Cargo.toml`:

```toml
[dependencies]
cydia-substrate = { path = ".", features = ["advanced", "disassembler", "debug"] }
```

- `default`: Core hooking functionality
- `debug`: Debug logging and hex dumps
- `substrate-internal`: Internal memory/process management APIs
- `advanced`: Architecture-specific APIs and advanced hooking
- `disassembler`: HDE64 x86/x64 disassembler

### Equivalent C++ to Rust Code

#### Basic Hook (Your Example)

**C++:**
```cpp
void *old_AddCoins = nullptr;

int AddCoins_hook(int amount) {
    return ((int(*)(int))old_AddCoins)(amount * 10);
}

MSHookFunction(
    (void *)getAbsoluteAddress("libil2cpp.so", Offsets.AddCoin),
    (void *)AddCoins_hook,
    (void **)&old_AddCoins
);
```

**Rust (Direct Translation):**
```rust
static mut OLD_ADD_COINS: Option<unsafe extern "C" fn(i32) -> i32> = None;

unsafe extern "C" fn add_coins_hook(amount: i32) -> i32 {
    OLD_ADD_COINS.unwrap()(amount * 10)
}

let addr = utils::get_absolute_address("libil2cpp.so", offsets::ADD_COIN).unwrap();
let mut original: *mut c_void = std::ptr::null_mut();

unsafe {
    MSHookFunction(
        addr as *mut c_void,
        add_coins_hook as *mut c_void,
        &mut original as *mut *mut c_void,
    );
    OLD_ADD_COINS = Some(std::mem::transmute(original));
}
```

**Rust (Idiomatic with Macros):**
```rust
substrate_hook! {
    fn add_coins(amount: i32) -> i32 {
        unsafe { call_original_add_coins(amount * 10) }
    }
}

let addr = utils::get_absolute_address("libil2cpp.so", offsets::ADD_COIN).unwrap();
install_add_coins_hook(addr as *mut c_void).unwrap();
```

### Utility Functions (Utils.h equivalent)

#### Find Library Base

**C++:**
```cpp
DWORD findLibrary(const char *library) {
    // /proc/self/maps parsing
}
```

**Rust:**
```rust
let base = utils::find_library("libil2cpp.so");
```

#### Get Absolute Address

**C++:**
```cpp
DWORD getAbsoluteAddress(const char *libraryName, DWORD relativeAddr) {
    libBase = findLibrary(libraryName);
    return (reinterpret_cast<DWORD>(libBase + relativeAddr));
}
```

**Rust:**
```rust
let addr = utils::get_absolute_address("libil2cpp.so", 0x123456);
```

#### Check if Library is Loaded

**C++:**
```cpp
bool isLibraryLoaded(const char *libraryName) {
    // Check /proc/self/maps
}
```

**Rust:**
```rust
if utils::is_library_loaded("libil2cpp.so") {
    // Library is loaded
}
```

#### Wait for Library

**Rust:**
```rust
if utils::wait_for_library("libil2cpp.so", 30000) {
    // Library loaded within 30 seconds
}
```

### Advanced Hooking

#### Inline Hooks with Trampoline

```rust
use cydia_substrate::advanced::{InlineHook, Trampoline};

let mut hook = InlineHook::new(target_addr, 16)?;
let trampoline = hook.create_trampoline()?;
hook.install(replacement_addr)?;
```

#### Hook Chains

```rust
use cydia_substrate::advanced::HookChain;

let mut chain = HookChain::new();
chain.add_hook(hook1);
chain.add_hook(hook2);
chain.install_all(&[replacement1, replacement2])?;
```

### Architecture-Specific Code

#### x86/x64 Instructions

```rust
use cydia_substrate::arch::x86::{X86Instruction, X86Register};

let mut buffer = vec![0u8; 32];
X86Instruction::write_jump_address(&mut buffer, 0, target, source);
```

#### ARM Instructions

```rust
use cydia_substrate::arch::arm::{ArmInstruction, ArmRegister};

let instr = ArmInstruction::mov_rd_rm(ArmRegister::R0, ArmRegister::R1);
```

### Disassembler (x86/x64 only)

```rust
use cydia_substrate::disasm::Disassembler;

let instr = Disassembler::disassemble(code_ptr)?;
println!("Length: {}", instr.len());
println!("Opcode: 0x{:02x}", instr.opcode());
if instr.is_rip_relative() {
    println!("RIP-relative instruction");
}
```

### Memory Protection

```rust
use cydia_substrate::memory::MemoryProtection;

MemoryProtection::make_writable(addr, size)?;
MemoryProtection::make_executable(addr, size)?;
MemoryProtection::clear_cache(addr, size);
```

### Debug and Logging

```rust
use cydia_substrate::debug::Debug;

Debug::enable();
Debug::print_memory(addr, 64, Some("MyFunction"));
Debug::print_hex(&bytes);
Debug::print_hook_info(symbol, replacement, original);
```

Or use macros:

```rust
debug_print!("Hook installed at {:p}", addr);
debug_memory!(addr, 32);
debug_hex!(&bytes);
```

### Macros Reference

#### `substrate_hook!` - Complete Hook Definition

```rust
substrate_hook! {
    fn function_name(arg1: Type1, arg2: Type2) -> RetType {
        // Your hook code
        unsafe { call_original_function_name(arg1, arg2) }
    }
}

// Generates:
// - hooked_function_name: Hook function
// - install_function_name_hook: Installation function
// - call_original_function_name: Original caller
// - ORIGINAL_FUNCTION_NAME: Storage
```

#### `define_hook!` - Flexible Hook

```rust
define_hook! {
    fn malloc(size: usize) -> *mut c_void
}

// Generates:
// - install_malloc: Install function
// - malloc_original: Original caller
// - MALLOC_ORIGINAL: Storage
```

#### `find_and_hook!` - One-liner

```rust
let original = find_and_hook!("libc.so.6", "malloc", my_malloc)?;
```

#### `hook_function!` - Direct Hook

```rust
let original = hook_function!(target_ptr => replacement_ptr);
```

### Complete Game Modding Example

```rust
use cydia_substrate::*;

mod offsets {
    pub const ADD_GOLD: usize = 0x123456;
    pub const ADD_GEMS: usize = 0x234567;
}

substrate_hook! {
    fn add_gold(amount: i32) -> i32 {
        println!("Adding gold: {}", amount);
        unsafe { call_original_add_gold(amount * 100) }
    }
}

substrate_hook! {
    fn add_gems(amount: i32) -> i32 {
        println!("Adding gems: {}", amount);
        unsafe { call_original_add_gems(amount * 50) }
    }
}

fn main() {
    utils::wait_for_library("libgame.so", 30000);

    let gold_addr = utils::get_absolute_address("libgame.so", offsets::ADD_GOLD).unwrap();
    let gems_addr = utils::get_absolute_address("libgame.so", offsets::ADD_GEMS).unwrap();

    install_add_gold_hook(gold_addr as *mut _).unwrap();
    install_add_gems_hook(gems_addr as *mut _).unwrap();

    println!("All hooks installed!");
}
```

### Android JNI Integration

```rust
#[cfg(target_os = "android")]
use jni::JNIEnv;

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_com_example_MainActivity_initHooks(
    env: JNIEnv,
    _class: jni::objects::JClass,
) {
    // Initialize hooks here
}
```

### Cross-Platform Considerations

```rust
#[cfg(target_os = "android")]
{
    // Android-specific code
    use cydia_substrate::android::find_symbol_in_library;
}

#[cfg(target_os = "ios")]
{
    // iOS-specific code
    use cydia_substrate::ios::ObjCHook;
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
{
    // x86-specific code
    use cydia_substrate::disasm::Disassembler;
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
{
    // ARM-specific code
    use cydia_substrate::arch::arm::ArmInstruction;
}
```

### Best Practices

1. **Use macros for cleaner code**: `substrate_hook!` generates all boilerplate
2. **Check library loading**: Use `wait_for_library()` before hooking
3. **Handle errors**: All hooking functions return `Result`
4. **Use utils module**: Replicate your C++ Utils.h functionality
5. **Enable features as needed**: Don't include `disassembler` if not needed
6. **Use debug features in development**: Enable `debug` feature for testing
7. **Prefer safe wrappers**: Use `Image`, `Symbol`, `Hook` over raw FFI
8. **Memory safety**: Original functions stored in static mut need `unsafe`

### Performance

- **Zero overhead**: Direct FFI calls
- **No runtime cost**: Macros expand at compile time
- **LTO enabled**: Link-time optimization in release builds
- **Static linking**: No dynamic dependencies
