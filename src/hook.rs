use std::ffi::c_void;
use std::marker::PhantomData;

use crate::MSHookFunction;

pub struct Hook<F> {
    original: *mut c_void,
    _phantom: PhantomData<F>,
}

impl<F> Hook<F> {
    pub fn new(symbol: *mut c_void, replacement: *mut c_void) -> Result<Self, &'static str> {
        if symbol.is_null() {
            return Err("Symbol is null");
        }
        if replacement.is_null() {
            return Err("Replacement is null");
        }

        let mut original: *mut c_void = std::ptr::null_mut();
        unsafe {
            MSHookFunction(symbol, replacement, &mut original as *mut *mut c_void);
        }

        Ok(Hook {
            original,
            _phantom: PhantomData,
        })
    }

    pub fn original(&self) -> *mut c_void {
        self.original
    }

    pub fn original_fn(&self) -> Option<F>
    where
        F: Copy,
    {
        if self.original.is_null() {
            None
        } else {
            unsafe { Some(std::mem::transmute_copy(&self.original)) }
        }
    }
}

unsafe impl<F> Send for Hook<F> {}
unsafe impl<F> Sync for Hook<F> {}

#[macro_export]
macro_rules! ms_hook {
    ($name:ident, $type:ty) => {
        static mut $name: Option<$type> = None;
    };
}

#[macro_export]
macro_rules! ms_hook_function {
    ($symbol:expr, $replace:expr, $original:expr) => {{
        let sym = $symbol as *mut std::ffi::c_void;
        let rep = $replace as *mut std::ffi::c_void;
        let mut orig: *mut std::ffi::c_void = std::ptr::null_mut();
        unsafe {
            $crate::MSHookFunction(sym, rep, &mut orig as *mut *mut std::ffi::c_void);
            $original = Some(std::mem::transmute(orig));
        }
    }};
}
