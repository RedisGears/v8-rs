/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! The Javascript array facilities, represented by the type
//! [LocalArray].

use crate::v8_c_raw::bindings::{
    v8_ArrayGet, v8_ArrayLen, v8_ArrayToValue, v8_FreeArray, v8_NewArray, v8_local_array,
};

use crate::v8::context_scope::ContextScope;
use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::ScopedValue;

use super::any::LocalValueAny;
use super::Value;

/// JS array.
#[derive(Debug, Clone)]
pub struct LocalArray<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_local_array>,
);

impl<'isolate_scope, 'isolate> LocalArray<'isolate_scope, 'isolate> {
    /// Creates a new array within the provided [IsolateScope].
    pub fn new(
        values: &[&LocalValueAny],
        isolate_scope: &'isolate_scope IsolateScope<'isolate>,
    ) -> Self {
        let args = values
            .iter()
            .map(|v| v.0.inner_val)
            .collect::<Vec<*mut _>>();
        let ptr = args.as_ptr();
        let inner_val =
            unsafe { v8_NewArray(isolate_scope.isolate.inner_isolate, ptr, values.len()) };
        Self(ScopedValue {
            inner_val,
            isolate_scope,
        })
    }

    /// Returns the length of the array.
    pub fn len(&self) -> usize {
        unsafe { v8_ArrayLen(self.0.inner_val) }
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
    ) -> LocalValueAny<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ArrayGet(ctx_scope.inner_ctx_ref, self.0.inner_val, index) };
        LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope: self.0.isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> From<LocalArray<'isolate_scope, 'isolate>>
    for LocalValueAny<'isolate_scope, 'isolate>
{
    fn from(array: LocalArray<'isolate_scope, 'isolate>) -> Self {
        let inner_val = unsafe { v8_ArrayToValue(array.0.inner_val) };
        LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope: array.0.isolate_scope,
        })
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
    type Item = LocalValueAny<'isolate_scope, 'isolate>;

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
        unsafe { v8_FreeArray(self.0.inner_val) }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueAny<'isolate_scope, 'isolate>>
    for LocalArray<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: LocalValueAny<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_array() {
            return Err("Value is not an array");
        }

        Ok(unsafe { val.as_array() })
    }
}

impl<'isolate_scope, 'isolate> TryFrom<Value<'isolate_scope, 'isolate>>
    for LocalArray<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: Value<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        match val {
            Value::Array(array) => Ok(array),
            Value::Other(any) => any.try_into(),
            _ => Err("Value is not an array"),
        }
    }
}
