use crate::v8_c_raw::bindings::{
    v8_FreeObject, v8_ObjectFreeze, v8_ObjectGet, v8_ObjectSet, v8_ObjectToValue,
    v8_ValueGetPropertyNames, v8_local_object,
};

use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::v8_array::V8LocalArray;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_value::V8LocalValue;

/// JS object
pub struct V8LocalObject<'isolate_scope, 'isolate> {
    pub(crate) inner_obj: *mut v8_local_object,
    pub(crate) isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> V8LocalObject<'isolate_scope, 'isolate> {
    /// Return the value of a given key
    #[must_use]
    pub fn get(
        &self,
        ctx_scope: &V8ContextScope,
        key: &V8LocalValue,
    ) -> Option<V8LocalValue<'isolate_scope, 'isolate>> {
        let inner_val =
            unsafe { v8_ObjectGet(ctx_scope.inner_ctx_ref, self.inner_obj, key.inner_val) };
        if inner_val.is_null() {
            None
        } else {
            Some(V8LocalValue {
                inner_val: inner_val,
                isolate_scope: self.isolate_scope,
            })
        }
    }

    pub fn set(&self, ctx_scope: &V8ContextScope, key: &V8LocalValue, val: &V8LocalValue) {
        unsafe {
            v8_ObjectSet(
                ctx_scope.inner_ctx_ref,
                self.inner_obj,
                key.inner_val,
                val.inner_val,
            )
        };
    }

    /// Convert the object into a generic JS value
    #[must_use]
    pub fn to_value(&self) -> V8LocalValue<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ObjectToValue(self.inner_obj) };
        V8LocalValue {
            inner_val: inner_val,
            isolate_scope: self.isolate_scope,
        }
    }

    pub fn freeze(&self, ctx_scope: &V8ContextScope) {
        unsafe { v8_ObjectFreeze(ctx_scope.inner_ctx_ref, self.inner_obj) };
    }

    /// Convert the object into a generic JS value
    #[must_use]
    pub fn get_property_names(
        &self,
        ctx_scope: &V8ContextScope,
    ) -> V8LocalArray<'isolate_scope, 'isolate> {
        let inner_array =
            unsafe { v8_ValueGetPropertyNames(ctx_scope.inner_ctx_ref, self.inner_obj) };
        V8LocalArray {
            inner_array: inner_array,
            isolate_scope: self.isolate_scope,
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for V8LocalObject<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeObject(self.inner_obj) }
    }
}
