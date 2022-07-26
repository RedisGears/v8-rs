use crate::v8_c_raw::bindings::{v8_FreeSet, v8_SetAdd, v8_SetToValue, v8_local_set};

use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_value::V8LocalValue;

/// JS object
pub struct V8LocalSet {
    pub(crate) inner_set: *mut v8_local_set,
}

impl V8LocalSet {
    /// Convert the object into a generic JS value
    #[must_use]
    pub fn to_value(&self) -> V8LocalValue {
        let inner_val = unsafe { v8_SetToValue(self.inner_set) };
        V8LocalValue { inner_val }
    }

    pub fn add(&self, ctx_scope: &V8ContextScope, val: &V8LocalValue) {
        unsafe { v8_SetAdd(ctx_scope.inner_ctx_ref, self.inner_set, val.inner_val) };
    }
}

impl Drop for V8LocalSet {
    fn drop(&mut self) {
        unsafe { v8_FreeSet(self.inner_set) }
    }
}
