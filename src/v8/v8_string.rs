/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_CloneString, v8_FreeString, v8_StringToStringObject, v8_StringToValue, v8_local_string,
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
    /// Creates a new string within the provided isolate.
    pub fn new(isolate_scope: &'isolate_scope V8IsolateScope<'isolate>, string: &str) -> Self {
        let inner_string = Self::new_string(isolate_scope, string);
        Self {
            inner_string,
            isolate_scope,
        }
    }

    /// Creates a new JS string for the provided isolate and returns
    /// a raw pointer to it.
    pub(crate) fn new_string(
        isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
        string: &str,
    ) -> *mut v8_local_string {
        unsafe {
            crate::v8_c_raw::bindings::v8_NewString(
                isolate_scope.isolate.inner_isolate,
                string.as_ptr().cast(),
                string.len(),
            )
        }
    }

    /// Convert the string object into a generic JS object.
    #[must_use]
    pub fn to_value(&self) -> V8LocalValue<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_StringToValue(self.inner_string) };
        V8LocalValue {
            inner_val,
            isolate_scope: self.isolate_scope,
        }
    }

    /// Same as writing `new String(...)` in JavaScript.
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

impl<'isolate_scope, 'isolate> std::fmt::Debug for V8LocalString<'isolate_scope, 'isolate> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string_value = String::try_from(self.to_owned()).map_err(|_| std::fmt::Error)?;
        #[allow(dead_code)]
        #[derive(Debug)]
        struct V8StringDebugPrinter {
            address: *mut v8_local_string,
            value: String,
        }
        let inner_string = V8StringDebugPrinter {
            address: self.inner_string,
            value: string_value,
        };
        f.debug_struct("V8LocalString")
            .field("inner_string", &inner_string)
            .field("isolate_scope", self.isolate_scope)
            .finish()
    }
}

impl<'isolate_scope, 'isolate> TryFrom<V8LocalString<'isolate_scope, 'isolate>> for String {
    type Error = &'static str;

    fn try_from(value: V8LocalString<'isolate_scope, 'isolate>) -> Result<String, Self::Error> {
        String::try_from(&value)
    }
}

impl<'isolate_scope, 'isolate> TryFrom<&V8LocalString<'isolate_scope, 'isolate>> for String {
    type Error = &'static str;

    fn try_from(value: &V8LocalString<'isolate_scope, 'isolate>) -> Result<String, Self::Error> {
        value
            .to_value()
            .to_utf8()
            .map(|utf| utf.as_str().to_owned())
            .ok_or("The V8LocalString isn't a valid UTF8 string.")
    }
}

impl<'isolate_scope, 'isolate> Clone for V8LocalString<'isolate_scope, 'isolate> {
    fn clone(&self) -> Self {
        Self {
            isolate_scope: self.isolate_scope,
            inner_string: unsafe { v8_CloneString(self.inner_string) },
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for V8LocalString<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeString(self.inner_string) }
    }
}
