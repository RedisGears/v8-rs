/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::LocalValueGeneric;
use crate::v8_c_raw::bindings::{v8_FreeUtf8, v8_Utf8PtrLen, v8_utf8_value};

use std::slice;
use std::str;

/// JS utf8 object
pub struct LocalUtf8<'isolate_scope, 'isolate> {
    pub(crate) inner_val: *mut v8_utf8_value,
    pub(crate) _isolate_scope: &'isolate_scope IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> LocalUtf8<'isolate_scope, 'isolate> {
    /// Get &str from the utf8 object
    /// # Panics
    #[must_use]
    pub fn as_str(&self) -> &str {
        let mut len: usize = 0;
        let buff = unsafe { v8_Utf8PtrLen(self.inner_val, &mut len) };
        let bytes = unsafe { slice::from_raw_parts(buff.cast::<u8>(), len) };
        str::from_utf8(bytes).unwrap()
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalUtf8<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeUtf8(self.inner_val) }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueGeneric<'isolate_scope, 'isolate>>
    for LocalUtf8<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: LocalValueGeneric<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_string() && !val.is_string_object() {
            return Err("Value is not string");
        }

        val.to_utf8().ok_or("Failed converting to utf8")
    }
}
