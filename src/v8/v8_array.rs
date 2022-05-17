use crate::v8_c_raw::bindings::{
    v8_ArrayGet, v8_ArrayLen, v8_ArrayToValue, v8_FreeArray, v8_local_array,
};

use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_value::V8LocalValue;

/// JS object
pub struct V8LocalArray {
    pub(crate) inner_array: *mut v8_local_array,
}

impl V8LocalArray {
    pub fn len(&self) -> usize {
        unsafe { v8_ArrayLen(self.inner_array) }
    }

    pub fn get(&self, ctx_scope: &V8ContextScope, index: usize) -> V8LocalValue {
        let inner_val = unsafe { v8_ArrayGet(ctx_scope.inner_ctx_ref, self.inner_array, index) };
        V8LocalValue { inner_val }
    }

    pub fn to_value(&self) -> V8LocalValue {
        let inner_val = unsafe { v8_ArrayToValue(self.inner_array) };
        V8LocalValue { inner_val }
    }
}

impl Drop for V8LocalArray {
    fn drop(&mut self) {
        unsafe { v8_FreeArray(self.inner_array) }
    }
}
