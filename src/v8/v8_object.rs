/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_FreeObject, v8_GetInternalFieldCount, v8_ObjectFreeze, v8_ObjectGet,
    v8_ObjectGetInternalField, v8_ObjectSet, v8_ObjectSetInternalField, v8_ObjectToValue,
    v8_ValueGetPropertyNames, v8_local_object,
};

use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::v8_array::V8LocalArray;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_value::V8LocalValue;
use crate::v8::v8_native_function_template::V8LocalNativeFunctionArgs;

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

    pub fn set_native_function<
    T: for<'d, 'e> Fn(
        &V8LocalNativeFunctionArgs<'d, 'e>,
        &'d V8IsolateScope<'e>,
        &V8ContextScope<'d, 'e>,
    ) -> Option<V8LocalValue<'d, 'e>>>
    (&self, ctx_scope: &V8ContextScope, key: &str, func: T) {
        let native_function = ctx_scope.new_native_function(func).to_value();
        let name = self.isolate_scope.new_string(key).to_value();
        unsafe {
            v8_ObjectSet(
                ctx_scope.inner_ctx_ref,
                self.inner_obj,
                name.inner_val,
                native_function.inner_val,
            )
        };
    }

    pub fn set_internal_field(&self, index: usize, val: &V8LocalValue) {
        unsafe { v8_ObjectSetInternalField(self.inner_obj, index, val.inner_val) };
    }

    #[must_use]
    pub fn get_internal_field(&self, index: usize) -> V8LocalValue<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ObjectGetInternalField(self.inner_obj, index) };
        V8LocalValue {
            inner_val: inner_val,
            isolate_scope: self.isolate_scope,
        }
    }

    #[must_use]
    pub fn get_internal_field_count(&self) -> usize {
        unsafe { v8_GetInternalFieldCount(self.inner_obj) }
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

impl<'isolate_scope, 'isolate> From<V8LocalValue<'isolate_scope, 'isolate>> for Result<V8LocalObject<'isolate_scope, 'isolate>, String> {
    fn from(val: V8LocalValue<'isolate_scope, 'isolate>) -> Self {
        if !val.is_object() {
            return Err("Value is not an object".to_string());
        }

        Ok(val.as_object())
    }
}
