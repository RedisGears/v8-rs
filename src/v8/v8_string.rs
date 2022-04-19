use crate::v8_c_raw::bindings::{
    v8_local_string,
    v8_FreeString,
};

pub struct V8LocalString {
    pub (crate) inner_string: *mut v8_local_string,
}

impl Drop for V8LocalString {
    fn drop(&mut self) {
        unsafe {v8_FreeString(self.inner_string)}
    }
}