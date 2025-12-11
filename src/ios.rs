use std::ffi::c_void;

#[cfg(target_os = "ios")]
use crate::ffi::MSHookMessageEx;

pub struct ObjCHook {
    original: *mut c_void,
}

impl ObjCHook {
    #[cfg(target_os = "ios")]
    pub fn hook_message(
        class: *mut c_void,
        selector: *mut c_void,
        implementation: *mut c_void,
    ) -> Result<Self, &'static str> {
        if class.is_null() {
            return Err("Class is null");
        }
        if selector.is_null() {
            return Err("Selector is null");
        }
        if implementation.is_null() {
            return Err("Implementation is null");
        }

        let mut original: *mut c_void = std::ptr::null_mut();
        unsafe {
            MSHookMessageEx(class, selector, implementation, &mut original as *mut *mut c_void);
        }

        Ok(ObjCHook { original })
    }

    pub fn original(&self) -> *mut c_void {
        self.original
    }
}

unsafe impl Send for ObjCHook {}
unsafe impl Sync for ObjCHook {}
