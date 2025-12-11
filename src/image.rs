use std::ffi::CString;

use crate::{MSGetImageByName, MSImageRef};

pub struct Image {
    handle: MSImageRef,
}

impl Image {
    pub fn by_name(name: &str) -> Option<Self> {
        let c_name = CString::new(name).ok()?;
        let handle = unsafe { MSGetImageByName(c_name.as_ptr()) };

        if handle.is_null() {
            None
        } else {
            Some(Image { handle })
        }
    }

    pub fn handle(&self) -> MSImageRef {
        self.handle
    }

    pub fn is_null(&self) -> bool {
        self.handle.is_null()
    }
}

unsafe impl Send for Image {}
unsafe impl Sync for Image {}
