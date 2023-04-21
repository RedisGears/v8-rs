/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! Contains the JavaScript set facilities.

use crate::v8_c_raw::bindings::{v8_FreeSet, v8_NewSet, v8_SetAdd, v8_SetToValue, v8_local_set};

use crate::v8::context_scope::ContextScope;
use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::ScopedValue;

use super::any::LocalValueAny;
use super::Value;

/// A javascript set.
#[derive(Debug, Clone)]
pub struct LocalSet<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_local_set>,
);

impl<'isolate_scope, 'isolate> LocalSet<'isolate_scope, 'isolate> {
    /// Creates a new local set for the passed [IsolateScope].
    pub fn new(isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        let inner_val = unsafe { v8_NewSet(isolate_scope.isolate.inner_isolate) };
        Self(ScopedValue {
            inner_val,
            isolate_scope,
        })
    }

    /// Adds new element to the set.
    pub fn add(&self, ctx_scope: &ContextScope, val: &LocalValueAny) {
        unsafe { v8_SetAdd(ctx_scope.inner_ctx_ref, self.0.inner_val, val.0.inner_val) };
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalSet<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeSet(self.0.inner_val) }
    }
}

impl<'isolate_scope, 'isolate> From<LocalSet<'isolate_scope, 'isolate>>
    for LocalValueAny<'isolate_scope, 'isolate>
{
    fn from(value: LocalSet<'isolate_scope, 'isolate>) -> Self {
        let inner_val = unsafe { v8_SetToValue(value.0.inner_val) };
        LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope: value.0.isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueAny<'isolate_scope, 'isolate>>
    for LocalSet<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: LocalValueAny<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_set() {
            return Err("Value is not a set");
        }

        Ok(unsafe { val.as_set() })
    }
}

impl<'isolate_scope, 'isolate> TryFrom<Value<'isolate_scope, 'isolate>>
    for LocalSet<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: Value<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        match val {
            Value::Set(set) => Ok(set),
            Value::Other(any) => any.try_into(),
            _ => Err("Value is not a set"),
        }
    }
}
