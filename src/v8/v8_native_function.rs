use crate::v8_c_raw::bindings::{
    v8_local_native_function,
    v8_local_value_arr,
    v8_FreeNativeFunction,
    v8_ArgsGet,
    v8_GetCurrentIsolate,
};

use std::os::raw::{c_void};

use crate::v8::v8_value::V8LocalValue;
use crate::v8::isolate::V8Isolate;

pub struct V8LocalNativeFunction {
    pub (crate) inner_func: *mut v8_local_native_function,
}

pub struct V8LocalNativeFunctionArgs {
    pub (crate) inner_arr: *mut v8_local_value_arr,
    len: usize,
}

pub (crate)extern "C" fn native_basic_function<T:Fn(&V8LocalNativeFunctionArgs)>(args: *mut v8_local_value_arr, len: usize, pd: *mut c_void) {
    let func = unsafe{&*(pd as *mut T)};
    let args = V8LocalNativeFunctionArgs{
        inner_arr: args,
        len: len,
    };
    func(&args);
}

impl V8LocalNativeFunctionArgs {
    pub fn get(&self, i: usize) -> V8LocalValue {
        assert!(i <= self.len);
        let val = unsafe{v8_ArgsGet(self.inner_arr, i)};
        V8LocalValue{
            inner_val: val,
        }
    }

    pub fn get_current_isolate(&self) -> V8Isolate {
        let inner_isolate = unsafe{v8_GetCurrentIsolate(self.inner_arr)};
        V8Isolate {
            inner_isolate: inner_isolate,
        }
    }
}

impl Drop for V8LocalNativeFunction {
    fn drop(&mut self) {
        unsafe {v8_FreeNativeFunction(self.inner_func)}
    }
}