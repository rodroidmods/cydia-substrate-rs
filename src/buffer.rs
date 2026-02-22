pub struct CodeWriter {
    ptr: *mut u8,
}

impl CodeWriter {
    pub unsafe fn new(ptr: *mut u8) -> Self {
        Self { ptr }
    }

    pub fn position(&self) -> *mut u8 {
        self.ptr
    }

    pub fn address(&self) -> usize {
        self.ptr as usize
    }

    pub unsafe fn write_u8(&mut self, value: u8) {
        unsafe {
            core::ptr::write(self.ptr, value);
            self.ptr = self.ptr.add(1);
        }
    }

    pub unsafe fn write_u16(&mut self, value: u16) {
        unsafe {
            core::ptr::write_unaligned(self.ptr as *mut u16, value);
            self.ptr = self.ptr.add(2);
        }
    }

    pub unsafe fn write_u32(&mut self, value: u32) {
        unsafe {
            core::ptr::write_unaligned(self.ptr as *mut u32, value);
            self.ptr = self.ptr.add(4);
        }
    }

    pub unsafe fn write_u64(&mut self, value: u64) {
        unsafe {
            core::ptr::write_unaligned(self.ptr as *mut u64, value);
            self.ptr = self.ptr.add(8);
        }
    }

    pub unsafe fn write_i32(&mut self, value: i32) {
        unsafe {
            core::ptr::write_unaligned(self.ptr as *mut i32, value);
            self.ptr = self.ptr.add(4);
        }
    }

    pub unsafe fn write_bytes(&mut self, data: &[u8]) {
        unsafe {
            core::ptr::copy_nonoverlapping(data.as_ptr(), self.ptr, data.len());
            self.ptr = self.ptr.add(data.len());
        }
    }
}
