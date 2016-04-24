use libc::{c_char, c_void};

#[repr(C)]
#[unsafe_no_drop_flag]
pub struct VarLenString {
    ptr: *mut c_char,
}

impl Drop for VarLenString {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ::libc::free(self.ptr as *mut c_void);
            }
        }
    }
}
