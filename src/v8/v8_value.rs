use crate::v8_c_raw::bindings::{
    v8_local_value,
    v8_FreeValue,
    v8_ToUtf8,
};

use crate::v8::isolate::V8Isolate;
use crate::v8::v8_utf8::V8LocalUtf8;

pub struct V8LocalValue {
    pub (crate) inner_val: *mut v8_local_value,
}

impl V8LocalValue {
    pub fn to_utf8(&self, isolate: &V8Isolate) -> V8LocalUtf8 {
        let inner_val = unsafe{v8_ToUtf8(isolate.inner_isolate, self.inner_val)};
        V8LocalUtf8{
            inner_val: inner_val,
        }
    }
}

impl Drop for V8LocalValue {
    fn drop(&mut self) {
        unsafe {v8_FreeValue(self.inner_val)}
    }
}