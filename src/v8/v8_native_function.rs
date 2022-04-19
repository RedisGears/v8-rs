use crate::v8_c_raw::bindings::{
    v8_local_native_function,
    v8_local_value_arr,
    v8_FreeNativeFunction,
};

use std::os::raw::{c_void};

pub struct V8LocalNativeFunction {
    pub (crate) inner_func: *mut v8_local_native_function,
}

pub (crate)extern "C" fn native_basic_function<T:Fn()>(_args: *mut v8_local_value_arr, _len: usize, pd: *mut c_void) {
    let func = unsafe{&*(pd as *mut T)};
    func();
}

impl Drop for V8LocalNativeFunction {
    fn drop(&mut self) {
        unsafe {v8_FreeNativeFunction(self.inner_func)}
    }
}