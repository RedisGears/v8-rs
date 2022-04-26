use crate::v8_c_raw::bindings::{v8_FreeObject, v8_ObjectGet, v8_ObjectToValue, v8_local_object};

use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_value::V8LocalValue;

/// JS object
pub struct V8LocalObject {
    pub(crate) inner_obj: *mut v8_local_object,
}

impl V8LocalObject {
    /// Return the value of a given key
    #[must_use]
    pub fn get(&self, ctx_scope: &V8ContextScope, key: &V8LocalValue) -> V8LocalValue {
        let inner_val =
            unsafe { v8_ObjectGet(ctx_scope.inner_ctx_ref, self.inner_obj, key.inner_val) };
        V8LocalValue { inner_val }
    }

    /// Convert the object into a generic JS value
    #[must_use]
    pub fn to_value(&self) -> V8LocalValue {
        let inner_val = unsafe { v8_ObjectToValue(self.inner_obj) };
        V8LocalValue { inner_val }
    }
}

impl Drop for V8LocalObject {
    fn drop(&mut self) {
        unsafe { v8_FreeObject(self.inner_obj) }
    }
}
