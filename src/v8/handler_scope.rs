use crate::v8_c_raw::bindings::{
    v8_handlers_scope,
    v8_NewHandlersScope,
    v8_FreeHandlersScope,
    v8_NewString,
    v8_NewObject,
    v8_NewNativeFunction,
};

use std::os::raw::{c_char, c_void};

use crate::v8::isolate::V8Isolate;
use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_object::V8LocalObject;
use crate::v8::v8_native_function::{V8LocalNativeFunction, native_basic_function};

pub struct V8HandlersScope<'a> {
    isolate: &'a V8Isolate,
    inner_handlers_scope: *mut v8_handlers_scope,
}

impl<'a> V8HandlersScope<'a> {
    pub fn new(isolate: &'a V8Isolate) -> V8HandlersScope<'a> {
        let inner_handlers_scope = unsafe{v8_NewHandlersScope(isolate.inner_isolate)};
        V8HandlersScope {
            isolate: isolate,
            inner_handlers_scope: inner_handlers_scope,
        }
    }

    pub fn new_string(&self, s: &str) -> V8LocalString {
        let inner_string = unsafe{v8_NewString(self.isolate.inner_isolate, s.as_ptr() as *const c_char, s.len())};
        V8LocalString{
            inner_string: inner_string,
        }
    }

    pub fn new_object(&self) -> V8LocalObject {
        let inner_obj = unsafe{v8_NewObject(self.isolate.inner_isolate)};
        V8LocalObject{
            inner_obj: inner_obj,
        }
    }

    pub fn new_native_function<T:Fn()>(&self, func: T) -> V8LocalNativeFunction {
        let inner_func = unsafe{v8_NewNativeFunction(self.isolate.inner_isolate, Some(native_basic_function::<T>), Box::into_raw(Box::new(func)) as *mut c_void)};
        V8LocalNativeFunction{
            inner_func: inner_func,
        }
    }
}

impl<'a> Drop for V8HandlersScope<'a> {
    fn drop(&mut self) {
        unsafe {v8_FreeHandlersScope(self.inner_handlers_scope)}
    }
}
