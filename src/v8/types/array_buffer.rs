/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_ArrayBufferGetData, v8_ArrayBufferToValue, v8_FreeArrayBuffer, v8_local_array_buff,
};

use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::LocalValueGeneric;

/// JS object
pub struct LocalArrayBuffer<'isolate_scope, 'isolate> {
    pub(crate) inner_array_buffer: *mut v8_local_array_buff,
    pub(crate) isolate_scope: &'isolate_scope IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> LocalArrayBuffer<'isolate_scope, 'isolate> {
    pub fn data(&self) -> &[u8] {
        let mut size = 0;
        let data =
            unsafe { v8_ArrayBufferGetData(self.inner_array_buffer, &mut size as *mut usize) };
        unsafe { std::slice::from_raw_parts(data.cast::<u8>(), size) }
    }

    pub fn to_value(&self) -> LocalValueGeneric<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ArrayBufferToValue(self.inner_array_buffer) };
        LocalValueGeneric {
            inner_val,
            isolate_scope: self.isolate_scope,
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalArrayBuffer<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeArrayBuffer(self.inner_array_buffer) }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueGeneric<'isolate_scope, 'isolate>>
    for LocalArrayBuffer<'isolate_scope, 'isolate>
{
    type Error = &'static str;
    fn try_from(val: LocalValueGeneric<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_array_buffer() {
            return Err("Value is not an array buffer");
        }

        Ok(val.as_array_buffer())
    }
}
