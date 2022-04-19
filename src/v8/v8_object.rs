use crate::v8_c_raw::bindings::{
    v8_local_object,
    v8_FreeObject,
    v8_ObjectSetFunction,
};

use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_native_function::V8LocalNativeFunction;

pub struct V8LocalObject {
    pub (crate) inner_obj: *mut v8_local_object,
}

impl V8LocalObject {
    pub fn set_native_function(&mut self, name: &V8LocalString, func: &V8LocalNativeFunction) {
        unsafe{v8_ObjectSetFunction(self.inner_obj, name.inner_string, func.inner_func)}
    }
}

impl Drop for V8LocalObject {
    fn drop(&mut self) {
        unsafe {v8_FreeObject(self.inner_obj)}
    }
}
