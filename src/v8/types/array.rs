/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_ArrayGet, v8_ArrayLen, v8_ArrayToValue, v8_FreeArray, v8_local_array,
};

use crate::v8::context_scope::ContextScope;
use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::LocalValueGeneric;

/// JS object
pub struct LocalArray<'isolate_scope, 'isolate> {
    pub(crate) inner_array: *mut v8_local_array,
    pub(crate) isolate_scope: &'isolate_scope IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> LocalArray<'isolate_scope, 'isolate> {
    /// Returns the length of the array.
    pub fn len(&self) -> usize {
        unsafe { v8_ArrayLen(self.inner_array) }
    }

    /// Returns true if the array is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator to the array's objects.
    pub fn iter<'array, 'context_scope>(
        &'array self,
        context_scope: &'context_scope ContextScope<'isolate_scope, 'isolate>,
    ) -> V8LocalArrayIterator<'context_scope, 'array, 'isolate_scope, 'isolate> {
        V8LocalArrayIterator {
            index: 0,
            array: self,
            context: context_scope,
        }
    }

    /// Returns a single object stored within the array.
    pub fn get(
        &self,
        ctx_scope: &ContextScope,
        index: usize,
    ) -> LocalValueGeneric<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ArrayGet(ctx_scope.inner_ctx_ref, self.inner_array, index) };
        LocalValueGeneric {
            inner_val,
            isolate_scope: self.isolate_scope,
        }
    }

    /// Converts the array to a value object.
    pub fn to_value(&self) -> LocalValueGeneric<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ArrayToValue(self.inner_array) };
        LocalValueGeneric {
            inner_val,
            isolate_scope: self.isolate_scope,
        }
    }
}

impl<'isolate_scope, 'isolate> From<LocalArray<'isolate_scope, 'isolate>>
    for LocalValueGeneric<'isolate_scope, 'isolate>
{
    fn from(array: LocalArray<'isolate_scope, 'isolate>) -> Self {
        array.to_value()
    }
}

/// An iterator over the objects stored within the [`V8LocalArray`].
pub struct V8LocalArrayIterator<'context_scope, 'array, 'isolate_scope, 'isolate> {
    index: usize,
    array: &'array LocalArray<'isolate_scope, 'isolate>,
    context: &'context_scope ContextScope<'isolate_scope, 'isolate>,
}

impl<'context_scope, 'array, 'isolate_scope, 'isolate> Iterator
    for V8LocalArrayIterator<'context_scope, 'array, 'isolate_scope, 'isolate>
{
    type Item = LocalValueGeneric<'isolate_scope, 'isolate>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.array.len() {
            return None;
        }

        let value = self.array.get(self.context, self.index);
        self.index += 1;
        Some(value)
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalArray<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeArray(self.inner_array) }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueGeneric<'isolate_scope, 'isolate>>
    for LocalArray<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: LocalValueGeneric<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_array() {
            return Err("Value is not an array");
        }

        Ok(val.as_array())
    }
}
