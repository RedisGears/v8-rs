use crate::v8_c_raw::bindings::{
    v8_local_string,
    v8_FreeString,
    v8_StringToValue,
};

use crate::v8::v8_value::V8LocalValue;

/// JS string object
pub struct V8LocalString {
    pub (crate) inner_string: *mut v8_local_string,
}

impl V8LocalString {

    /// Convert the string object into a generic JS object.
    pub fn to_value(&self) -> V8LocalValue {
        let inner_val = unsafe{v8_StringToValue(self.inner_string)};
        V8LocalValue {
            inner_val: inner_val,
        }
    }
}

impl Drop for V8LocalString {
    fn drop(&mut self) {
        unsafe {v8_FreeString(self.inner_string)}
    }
}