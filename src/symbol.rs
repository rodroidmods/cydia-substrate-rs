use std::ffi::{c_void, CString};
use std::marker::PhantomData;

use crate::{MSFindSymbol, MSImageRef};

pub struct Symbol<T> {
    ptr: *mut c_void,
    _phantom: PhantomData<T>,
}

impl<T> Symbol<T> {
    pub fn find(image: MSImageRef, name: &str) -> Option<Self> {
        let c_name = CString::new(name).ok()?;
        let ptr = unsafe { MSFindSymbol(image, c_name.as_ptr()) };

        if ptr.is_null() {
            None
        } else {
            Some(Symbol {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    pub fn find_global(name: &str) -> Option<Self> {
        Self::find(std::ptr::null(), name)
    }

    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr
    }

    pub fn as_fn(&self) -> Option<T>
    where
        T: Copy,
    {
        if self.ptr.is_null() {
            None
        } else {
            unsafe { Some(std::mem::transmute_copy(&self.ptr)) }
        }
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
}

unsafe impl<T> Send for Symbol<T> {}
unsafe impl<T> Sync for Symbol<T> {}

impl<T> Clone for Symbol<T> {
    fn clone(&self) -> Self {
        Symbol {
            ptr: self.ptr,
            _phantom: PhantomData,
        }
    }
}

impl<T> Copy for Symbol<T> {}
