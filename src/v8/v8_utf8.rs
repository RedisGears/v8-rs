use crate::v8_c_raw::bindings::{
    v8_utf8_value,
    v8_FreeUtf8,
    v8_Utf8PtrLen,
};

use std::slice;
use std::str;

/// JS utf8 object
pub struct V8LocalUtf8 {
    pub (crate) inner_val: *mut v8_utf8_value,
}

impl V8LocalUtf8 {

    /// Get &str from the utf8 object
    pub fn as_str(&self) -> &str {
        let mut len: usize = 0;
        let buff = unsafe{v8_Utf8PtrLen(self.inner_val, &mut len)};
        let bytes = unsafe{slice::from_raw_parts(buff as *const u8, len)};
        str::from_utf8(bytes).unwrap()
    }
}

impl Drop for V8LocalUtf8 {
    fn drop(&mut self) {
        unsafe {v8_FreeUtf8(self.inner_val)}
    }
}