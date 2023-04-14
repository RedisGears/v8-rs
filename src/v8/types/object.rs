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

use crate::v8::context_scope::ContextScope;
use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::native_function_template::LocalNativeFunctionArgs;
use crate::v8::types::LocalArray;
use crate::v8::types::LocalValueGeneric;

/// JS object
pub struct LocalObject<'isolate_scope, 'isolate> {
    pub(crate) inner_obj: *mut v8_local_object,
    pub(crate) isolate_scope: &'isolate_scope IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> LocalObject<'isolate_scope, 'isolate> {
    /// Return the value of a given key
    #[must_use]
    pub fn get(
        &self,
        ctx_scope: &ContextScope,
        key: &LocalValueGeneric,
    ) -> Option<LocalValueGeneric<'isolate_scope, 'isolate>> {
        let inner_val =
            unsafe { v8_ObjectGet(ctx_scope.inner_ctx_ref, self.inner_obj, key.inner_val) };
        if inner_val.is_null() {
            None
        } else {
            Some(LocalValueGeneric {
                inner_val,
                isolate_scope: self.isolate_scope,
            })
        }
    }

    /// Sugar for get that recieve the field name as &str
    #[must_use]
    pub fn get_str_field(
        &self,
        ctx_scope: &ContextScope,
        key: &str,
    ) -> Option<LocalValueGeneric<'isolate_scope, 'isolate>> {
        let key = self.isolate_scope.new_string(key);
        self.get(ctx_scope, &key.to_value())
    }

    pub fn set(&self, ctx_scope: &ContextScope, key: &LocalValueGeneric, val: &LocalValueGeneric) {
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
            &LocalNativeFunctionArgs<'d, 'e>,
            &'d IsolateScope<'e>,
            &ContextScope<'d, 'e>,
        ) -> Option<LocalValueGeneric<'d, 'e>>,
    >(
        &self,
        ctx_scope: &ContextScope,
        key: &str,
        func: T,
    ) {
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

    pub fn set_internal_field(&self, index: usize, val: &LocalValueGeneric) {
        unsafe { v8_ObjectSetInternalField(self.inner_obj, index, val.inner_val) };
    }

    #[must_use]
    pub fn get_internal_field(&self, index: usize) -> LocalValueGeneric<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ObjectGetInternalField(self.inner_obj, index) };
        LocalValueGeneric {
            inner_val,
            isolate_scope: self.isolate_scope,
        }
    }

    #[must_use]
    pub fn get_internal_field_count(&self) -> usize {
        unsafe { v8_GetInternalFieldCount(self.inner_obj) }
    }

    /// Convert the object into a generic JS value
    #[must_use]
    pub fn to_value(&self) -> LocalValueGeneric<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ObjectToValue(self.inner_obj) };
        LocalValueGeneric {
            inner_val,
            isolate_scope: self.isolate_scope,
        }
    }

    pub fn freeze(&self, ctx_scope: &ContextScope) {
        unsafe { v8_ObjectFreeze(ctx_scope.inner_ctx_ref, self.inner_obj) };
    }

    /// Convert the object into a generic JS value
    #[must_use]
    pub fn get_property_names(
        &self,
        ctx_scope: &ContextScope,
    ) -> LocalArray<'isolate_scope, 'isolate> {
        let inner_array =
            unsafe { v8_ValueGetPropertyNames(ctx_scope.inner_ctx_ref, self.inner_obj) };
        LocalArray {
            inner_array,
            isolate_scope: self.isolate_scope,
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalObject<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeObject(self.inner_obj) }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueGeneric<'isolate_scope, 'isolate>>
    for LocalObject<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: LocalValueGeneric<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_object() {
            return Err("Value is not an object");
        }

        Ok(val.as_object())
    }
}
