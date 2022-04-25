use crate::v8_c_raw::bindings::{
    v8_local_native_function_template,
    v8_local_value,
    v8_local_value_arr,
    v8_FreeNativeFunctionTemplate,
    v8_ArgsGet,
    v8_GetCurrentIsolate,
};

use std::ptr;
use std::os::raw::{c_void};

use crate::v8::v8_value::V8LocalValue;
use crate::v8::isolate::V8Isolate;

pub struct V8LocalNativeFunctionTemplate {
    pub (crate) inner_func: *mut v8_local_native_function_template,
}

pub struct V8LocalNativeFunctionArgs {
    pub (crate) inner_arr: *mut v8_local_value_arr,
    len: usize,
}

pub (crate)extern "C" fn native_basic_function<T:Fn(&V8LocalNativeFunctionArgs) -> Option<V8LocalValue>>(args: *mut v8_local_value_arr, len: usize, pd: *mut c_void) -> *mut v8_local_value {
    let func = unsafe{&*(pd as *mut T)};
    let args = V8LocalNativeFunctionArgs{
        inner_arr: args,
        len: len,
    };
    let res = func(&args);
    match res {
        Some(mut r) => {
            let inner_val = r.inner_val;
            r.inner_val = ptr::null_mut();
            inner_val
        }
        None => ptr::null_mut(),
    }
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

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Drop for V8LocalNativeFunctionTemplate {
    fn drop(&mut self) {
        unsafe {v8_FreeNativeFunctionTemplate(self.inner_func)}
    }
}