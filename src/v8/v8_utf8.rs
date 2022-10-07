use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8_c_raw::bindings::{v8_FreeUtf8, v8_Utf8PtrLen, v8_utf8_value};

use std::slice;
use std::str;

/// JS utf8 object
pub struct V8LocalUtf8<'isolate_scope, 'isolate> {
    pub(crate) inner_val: *mut v8_utf8_value,
    pub(crate) _isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> V8LocalUtf8<'isolate_scope, 'isolate> {
    /// Get &str from the utf8 object
    /// # Panics
    #[must_use]
    pub fn as_str(&self) -> &str {
        let mut len: usize = 0;
        let buff = unsafe { v8_Utf8PtrLen(self.inner_val, &mut len) };
        let bytes = unsafe { slice::from_raw_parts(buff.cast::<u8>(), len) };
        str::from_utf8(bytes).unwrap()
    }
}

impl<'isolate_scope, 'isolate> Drop for V8LocalUtf8<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeUtf8(self.inner_val) }
    }
}
