/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! The JavaScript string facilities.

use crate::v8_c_raw::bindings::{
    v8_FreeString, v8_NewString, v8_StringToStringObject, v8_StringToValue, v8_local_string,
};

use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::object::LocalObject;
use crate::v8::types::ScopedValue;

use super::any::LocalValueAny;

/// A JavaScript string object.
#[derive(Debug, Clone)]
pub struct LocalString<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_local_string>,
);

impl<'isolate_scope, 'isolate> LocalString<'isolate_scope, 'isolate> {
    /// Creates a new JavaScript string within the passed [IsolateScope].
    pub fn new(s: &str, isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        let inner_val = unsafe {
            v8_NewString(
                isolate_scope.isolate.inner_isolate,
                s.as_ptr().cast(),
                s.len(),
            )
        };

        Self(ScopedValue {
            inner_val,
            isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalString<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeString(self.0.inner_val) }
    }
}

impl<'isolate_scope, 'isolate> From<LocalString<'isolate_scope, 'isolate>>
    for LocalValueAny<'isolate_scope, 'isolate>
{
    fn from(value: LocalString<'isolate_scope, 'isolate>) -> Self {
        let inner_val = unsafe { v8_StringToValue(value.0.inner_val) };
        LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope: value.0.isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> From<LocalString<'isolate_scope, 'isolate>>
    for LocalObject<'isolate_scope, 'isolate>
{
    fn from(value: LocalString<'isolate_scope, 'isolate>) -> Self {
        let inner_val = unsafe {
            v8_StringToStringObject(
                value.0.isolate_scope.isolate.inner_isolate,
                value.0.inner_val,
            )
        };
        LocalObject(ScopedValue {
            inner_val,
            isolate_scope: value.0.isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueAny<'isolate_scope, 'isolate>>
    for LocalString<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: LocalValueAny<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_string() {
            return Err("Value is not a string");
        }

        Ok(unsafe { val.as_string() })
    }
}
