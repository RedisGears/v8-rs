use crate::v8_c_raw::bindings::{
    v8_trycatch,
    v8_FreeTryCatch,
    v8_TryCatchGetException,
};

use crate::v8::v8_value::V8LocalValue;

pub struct V8TryCatch {
    pub (crate) inner_trycatch: *mut v8_trycatch,
}

impl V8TryCatch {
    pub fn get_exception(&self) -> V8LocalValue {
        let inner_val = unsafe{v8_TryCatchGetException(self.inner_trycatch)};
        V8LocalValue {
            inner_val: inner_val,
        }
    }
}

impl Drop for V8TryCatch {
    fn drop(&mut self) {
        unsafe {v8_FreeTryCatch(self.inner_trycatch)}
    }
}