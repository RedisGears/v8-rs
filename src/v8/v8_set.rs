/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

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
            inner_val,
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

impl<'isolate_scope, 'isolate> TryFrom<&V8LocalValue<'isolate_scope, 'isolate>>
    for V8LocalSet<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: &V8LocalValue<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_set() {
            return Err("Value is not a set");
        }

        Ok(val.as_set())
    }
}
