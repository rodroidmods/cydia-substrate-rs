pub mod arm;
pub mod x86;

#[cfg(target_arch = "arm")]
pub use arm::*;

#[cfg(target_arch = "aarch64")]
pub use arm::*;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use x86::*;
