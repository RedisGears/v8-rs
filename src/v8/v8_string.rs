use crate::v8_c_raw::bindings::{
    v8_FreeString, v8_StringToStringObject, v8_StringToValue, v8_local_string,
};

use crate::v8::isolate::V8Isolate;
use crate::v8::v8_object::V8LocalObject;
use crate::v8::v8_value::V8LocalValue;

/// JS string object
pub struct V8LocalString {
    pub(crate) inner_string: *mut v8_local_string,
}

impl V8LocalString {
    /// Convert the string object into a generic JS object.
    #[must_use]
    pub fn to_value(&self) -> V8LocalValue {
        let inner_val = unsafe { v8_StringToValue(self.inner_string) };
        V8LocalValue { inner_val }
    }

    /// Same as writing 'new String(...)'.
    #[must_use]
    pub fn to_string_object(&self, isolate: &V8Isolate) -> V8LocalObject {
        let inner_obj =
            unsafe { v8_StringToStringObject(isolate.inner_isolate, self.inner_string) };
        V8LocalObject { inner_obj }
    }
}

impl Drop for V8LocalString {
    fn drop(&mut self) {
        unsafe { v8_FreeString(self.inner_string) }
    }
}
