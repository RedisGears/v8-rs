/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_FreeString, v8_StringToStringObject, v8_StringToValue, v8_local_string,
};

use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::v8_object::V8LocalObject;
use crate::v8::v8_value::V8LocalValue;

/// JS string object
pub struct V8LocalString<'isolate_scope, 'isolate> {
    pub(crate) inner_string: *mut v8_local_string,
    pub(crate) isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> V8LocalString<'isolate_scope, 'isolate> {
    /// Convert the string object into a generic JS object.
    #[must_use]
    pub fn to_value(&self) -> V8LocalValue<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_StringToValue(self.inner_string) };
        V8LocalValue {
            inner_val,
            isolate_scope: self.isolate_scope,
        }
    }

    /// Same as writing 'new String(...)'.
    #[must_use]
    pub fn to_string_object(&self) -> V8LocalObject<'isolate_scope, 'isolate> {
        let inner_obj = unsafe {
            v8_StringToStringObject(self.isolate_scope.isolate.inner_isolate, self.inner_string)
        };
        V8LocalObject {
            inner_obj,
            isolate_scope: self.isolate_scope,
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for V8LocalString<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeString(self.inner_string) }
    }
}
