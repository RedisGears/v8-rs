use crate::v8_c_raw::bindings::{v8_FreeSet, v8_SetAdd, v8_SetToValue, v8_local_set};

use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_value::V8LocalValue;

/// JS object
pub struct V8LocalSet<'isolate_scope, 'isolate> {
    pub(crate) inner_set: *mut v8_local_set,
    pub(crate) isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> V8LocalSet<'isolate_scope, 'isolate> {
    /// Convert the object into a generic JS value
    #[must_use]
    pub fn to_value(&self) -> V8LocalValue<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_SetToValue(self.inner_set) };
        V8LocalValue {
            inner_val: inner_val,
            isolate_scope: self.isolate_scope,
        }
    }

    pub fn add(&self, ctx_scope: &V8ContextScope, val: &V8LocalValue) {
        unsafe { v8_SetAdd(ctx_scope.inner_ctx_ref, self.inner_set, val.inner_val) };
    }
}

impl<'isolate_scope, 'isolate> Drop for V8LocalSet<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeSet(self.inner_set) }
    }
}
