/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_FreeObject, v8_GetInternalFieldCount, v8_NewObject, v8_ObjectFreeze, v8_ObjectGet,
    v8_ObjectGetInternalField, v8_ObjectSet, v8_ObjectSetInternalField, v8_ObjectToValue,
    v8_ValueGetPropertyNames, v8_local_object,
};

use crate::v8::context_scope::ContextScope;
use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::native_function_template::LocalNativeFunctionArgs;
use crate::v8::types::LocalArray;
use crate::v8::types::ScopedValue;

use super::string::LocalString;
use super::{LocalValueAny, Value};

/// A JavaScript object.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct LocalObject<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_local_object>,
);

impl<'isolate_scope, 'isolate> LocalObject<'isolate_scope, 'isolate> {
    /// Creates a new local object within the provided [IsolateScope].
    pub fn new(isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        let inner_val = unsafe { v8_NewObject(isolate_scope.isolate.inner_isolate) };
        Self(ScopedValue {
            inner_val,
            isolate_scope,
        })
    }

    /// Returns the value of a given key.
    #[must_use]
    pub fn get(
        &self,
        ctx_scope: &ContextScope,
        key: &LocalValueAny<'isolate_scope, 'isolate>,
    ) -> Option<Value<'isolate_scope, 'isolate>> {
        let inner_val =
            unsafe { v8_ObjectGet(ctx_scope.inner_ctx_ref, self.0.inner_val, key.0.inner_val) };
        if inner_val.is_null() {
            None
        } else {
            Some(
                LocalValueAny(ScopedValue {
                    inner_val,
                    isolate_scope: self.0.isolate_scope,
                })
                .into(),
            )
        }
    }

    /// Get value of a field by string.
    #[must_use]
    pub fn get_str_field(
        &self,
        ctx_scope: &ContextScope,
        key: &str,
    ) -> Option<Value<'isolate_scope, 'isolate>> {
        let key: LocalString = self.0.isolate_scope.create_string(key).try_into().unwrap();
        let key = LocalValueAny::from(key);
        self.get(ctx_scope, &key)
    }

    pub fn set(&self, ctx_scope: &ContextScope, key: &LocalValueAny, val: &LocalValueAny) {
        unsafe {
            v8_ObjectSet(
                ctx_scope.inner_ctx_ref,
                self.0.inner_val,
                key.0.inner_val,
                val.0.inner_val,
            )
        };
    }

    pub fn set_native_function<
        T: for<'d, 'e> Fn(
            &LocalNativeFunctionArgs<'d, 'e>,
            &'d IsolateScope<'e>,
            &ContextScope<'d, 'e>,
        ) -> Option<LocalValueAny<'d, 'e>>,
    >(
        &self,
        ctx_scope: &ContextScope,
        key: &str,
        func: T,
    ) {
        let native_function =
            LocalValueAny::try_from(ctx_scope.create_native_function(func)).unwrap();
        let name: LocalString = self.0.isolate_scope.create_string(key).try_into().unwrap();
        let name = LocalValueAny::from(name);
        unsafe {
            v8_ObjectSet(
                ctx_scope.inner_ctx_ref,
                self.0.inner_val,
                name.0.inner_val,
                native_function.0.inner_val,
            )
        };
    }

    pub fn set_internal_field(&self, index: usize, val: &LocalValueAny) {
        unsafe { v8_ObjectSetInternalField(self.0.inner_val, index, val.0.inner_val) };
    }

    pub fn get_internal_field(&self, index: usize) -> Value<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ObjectGetInternalField(self.0.inner_val, index) };
        LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope: self.0.isolate_scope,
        })
        .into()
    }

    pub fn get_internal_field_count(&self) -> usize {
        unsafe { v8_GetInternalFieldCount(self.0.inner_val) }
    }

    pub fn freeze(&self, ctx_scope: &ContextScope) {
        unsafe { v8_ObjectFreeze(ctx_scope.inner_ctx_ref, self.0.inner_val) };
    }

    /// Convert the object into a generic JS value
    pub fn get_property_names(
        &self,
        ctx_scope: &ContextScope,
    ) -> LocalArray<'isolate_scope, 'isolate> {
        let inner_val =
            unsafe { v8_ValueGetPropertyNames(ctx_scope.inner_ctx_ref, self.0.inner_val) };
        LocalArray(ScopedValue {
            inner_val,
            isolate_scope: self.0.isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalObject<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeObject(self.0.inner_val) }
    }
}

impl<'isolate_scope, 'isolate> From<LocalObject<'isolate_scope, 'isolate>>
    for LocalValueAny<'isolate_scope, 'isolate>
{
    fn from(value: LocalObject<'isolate_scope, 'isolate>) -> Self {
        let inner_val = unsafe { v8_ObjectToValue(value.0.inner_val) };
        LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope: value.0.isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueAny<'isolate_scope, 'isolate>>
    for LocalObject<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: LocalValueAny<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_object() {
            return Err("Value is not an object");
        }

        Ok(unsafe { val.as_object() })
    }
}

impl<'isolate_scope, 'isolate> TryFrom<Value<'isolate_scope, 'isolate>>
    for LocalObject<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: Value<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        match val {
            Value::Object(object) => Ok(object),
            Value::Other(any) => Self::try_from(any),
            _ => Err("Value is not an object"),
        }
    }
}
