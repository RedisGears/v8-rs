/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_ArrayGet, v8_ArrayLen, v8_ArrayToValue, v8_FreeArray, v8_local_array,
};

use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_value::V8LocalValue;

/// JS object
pub struct V8LocalArray<'isolate_scope, 'isolate> {
    pub(crate) inner_array: *mut v8_local_array,
    pub(crate) isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> V8LocalArray<'isolate_scope, 'isolate> {
    pub fn len(&self) -> usize {
        unsafe { v8_ArrayLen(self.inner_array) }
    }

    pub fn get(
        &self,
        ctx_scope: &V8ContextScope,
        index: usize,
    ) -> V8LocalValue<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ArrayGet(ctx_scope.inner_ctx_ref, self.inner_array, index) };
        V8LocalValue {
            inner_val: inner_val,
            isolate_scope: self.isolate_scope,
        }
    }

    pub fn to_value(&self) -> V8LocalValue<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ArrayToValue(self.inner_array) };
        V8LocalValue {
            inner_val: inner_val,
            isolate_scope: self.isolate_scope,
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for V8LocalArray<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeArray(self.inner_array) }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<V8LocalValue<'isolate_scope, 'isolate>>
    for V8LocalArray<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: V8LocalValue<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_array() {
            return Err("Value is not an array");
        }

        Ok(val.as_array())
    }
}
