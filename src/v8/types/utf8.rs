/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8::types::ScopedValue;
use crate::v8_c_raw::bindings::{v8_FreeUtf8, v8_ToUtf8, v8_Utf8PtrLen, v8_utf8_value};

use std::ptr::NonNull;
use std::slice;
use std::str;

use super::any::LocalValueAny;
use super::string::LocalString;

/// JS utf8 object
pub struct LocalUtf8<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_utf8_value>,
);

impl<'isolate_scope, 'isolate> LocalUtf8<'isolate_scope, 'isolate> {
    /// Get a [str] slice from the object.
    ///
    /// # Panics
    ///
    /// Panics when the [std::str::from_utf8] fails.
    pub fn as_str(&self) -> &str {
        let mut len: usize = 0;
        let buff = unsafe { v8_Utf8PtrLen(self.0.inner_val, &mut len) };
        let bytes = unsafe { slice::from_raw_parts(buff.cast(), len) };
        str::from_utf8(bytes).expect("Couldn't create a string")
    }
}

impl<'isolate_scope, 'isolate> AsRef<str> for LocalUtf8<'isolate_scope, 'isolate> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalUtf8<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeUtf8(self.0.inner_val) }
    }
}

impl<'isolate_scope, 'isolate> From<LocalString<'isolate_scope, 'isolate>>
    for LocalUtf8<'isolate_scope, 'isolate>
{
    fn from(value: LocalString<'isolate_scope, 'isolate>) -> Self {
        let value = LocalValueAny::from(value);
        // Note that given the implementation of the `TryFrom` below,
        // this code should never fail as we always check for the actual
        // type of the value stored within the `LocalValueAny` to be
        // a string or a string object, and since in this case it was
        // a string and we knew that fact, it must never fail.
        Self::try_from(value).expect("Failed to convert a string to Utf8.")
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueAny<'isolate_scope, 'isolate>>
    for LocalUtf8<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(value: LocalValueAny<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !value.is_string() && !value.is_string_object() {
            return Err("Value is not string");
        }

        NonNull::new(unsafe {
            v8_ToUtf8(
                value.0.isolate_scope.isolate.inner_isolate,
                value.0.inner_val,
            )
        })
        .map(|ptr| {
            LocalUtf8(ScopedValue {
                inner_val: ptr.as_ptr(),
                isolate_scope: value.0.isolate_scope,
            })
        })
        .ok_or("Failed converting to utf8")
    }
}

impl<'isolate_scope, 'isolate> From<LocalUtf8<'isolate_scope, 'isolate>> for String {
    fn from(value: LocalUtf8<'isolate_scope, 'isolate>) -> Self {
        value.as_str().to_owned()
    }
}
