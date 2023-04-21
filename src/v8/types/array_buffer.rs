/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_ArrayBufferGetData, v8_ArrayBufferToValue, v8_FreeArrayBuffer, v8_NewArrayBuffer,
    v8_local_array_buff,
};

use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::ScopedValue;

use super::any::LocalValueAny;
use super::Value;

/// JavaScript array buffer.
#[derive(Debug, Clone)]
pub struct LocalArrayBuffer<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_local_array_buff>,
);

impl<'isolate_scope, 'isolate> LocalArrayBuffer<'isolate_scope, 'isolate> {
    /// Creates a new local array buffer within the provided [IsolateScope].
    pub fn new(bytes: &[u8], isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        let inner_val = unsafe {
            v8_NewArrayBuffer(
                isolate_scope.isolate.inner_isolate,
                bytes.as_ptr() as *const _,
                bytes.len(),
            )
        };

        Self(ScopedValue {
            inner_val,
            isolate_scope,
        })
    }

    pub fn data(&self) -> &[u8] {
        let mut size = 0;
        let data = unsafe { v8_ArrayBufferGetData(self.0.inner_val, &mut size as *mut usize) };
        unsafe { std::slice::from_raw_parts(data.cast::<u8>(), size) }
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalArrayBuffer<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeArrayBuffer(self.0.inner_val) }
    }
}

impl<'isolate_scope, 'isolate> From<LocalArrayBuffer<'isolate_scope, 'isolate>>
    for LocalValueAny<'isolate_scope, 'isolate>
{
    fn from(value: LocalArrayBuffer<'isolate_scope, 'isolate>) -> Self {
        let inner_val = unsafe { v8_ArrayBufferToValue(value.0.inner_val) };
        LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope: value.0.isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueAny<'isolate_scope, 'isolate>>
    for LocalArrayBuffer<'isolate_scope, 'isolate>
{
    type Error = &'static str;
    fn try_from(val: LocalValueAny<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_array_buffer() {
            return Err("Value is not an array buffer");
        }

        Ok(unsafe { val.as_array_buffer() })
    }
}

impl<'isolate_scope, 'isolate> TryFrom<Value<'isolate_scope, 'isolate>>
    for LocalArrayBuffer<'isolate_scope, 'isolate>
{
    type Error = &'static str;
    fn try_from(val: Value<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        match val {
            Value::ArrayBuffer(array_buffer) => Ok(array_buffer),
            Value::Other(any) => any.try_into(),
            _ => Err("Value is not an array buffer"),
        }
    }
}
