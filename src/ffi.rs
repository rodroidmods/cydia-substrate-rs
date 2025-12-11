use std::ffi::c_void;

#[cfg(feature = "substrate-internal")]
use std::os::raw::c_int;

#[cfg(feature = "substrate-internal")]
use crate::{SubstrateAllocatorRef, SubstrateMemoryRef, SubstrateProcessRef};

extern "C" {
    #[cfg(feature = "substrate-internal")]
    pub fn SubstrateProcessCreate(
        allocator: SubstrateAllocatorRef,
        pid: c_int,
    ) -> SubstrateProcessRef;

    #[cfg(feature = "substrate-internal")]
    pub fn SubstrateProcessRelease(process: SubstrateProcessRef);

    #[cfg(feature = "substrate-internal")]
    pub fn SubstrateMemoryCreate(
        allocator: SubstrateAllocatorRef,
        process: SubstrateProcessRef,
        data: *mut c_void,
        size: usize,
    ) -> SubstrateMemoryRef;

    #[cfg(feature = "substrate-internal")]
    pub fn SubstrateMemoryRelease(memory: SubstrateMemoryRef);
}

#[cfg(target_os = "ios")]
extern "C" {
    pub fn MSHookMessageEx(
        class: *mut c_void,
        sel: *mut c_void,
        imp: *mut c_void,
        result: *mut *mut c_void,
    );
}

pub type MSHookFunctionPtr = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut *mut c_void);
