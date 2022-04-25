use crate::v8_c_raw::bindings::{
    v8_local_value,
    v8_persisted_value,
    v8_FreeValue,
    v8_ToUtf8,
    v8_ValueIsFunction,
    v8_ValueIsAsyncFunction,
    v8_ValueIsString,
    v8_ValueIsNumber,
    v8_ValueIsPromise,
    v8_ValueIsObject,
    v8_ValueAsString,
    v8_FreePersistedValue,
    v8_PersistValue,
    v8_PersistedValueToLocal,
    v8_FunctionCall,
    v8_ValueAsPromise,
};

use std::ptr;

use crate::v8::isolate::V8Isolate;
use crate::v8::v8_utf8::V8LocalUtf8;
use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_promise::V8LocalPromise;

pub struct V8LocalValue {
    pub (crate) inner_val: *mut v8_local_value,
}

pub struct V8PersistValue {
    pub (crate) inner_val: *mut v8_persisted_value,
}

impl V8LocalValue {
    pub fn to_utf8(&self, isolate: &V8Isolate) -> V8LocalUtf8 {
        let inner_val = unsafe{v8_ToUtf8(isolate.inner_isolate, self.inner_val)};
        V8LocalUtf8{
            inner_val: inner_val,
        }
    }

    pub fn is_string(&self) -> bool {
        if unsafe{v8_ValueIsString(self.inner_val)} != 0 {true} else {false}
    }

    pub fn as_string(&self) -> V8LocalString {
        let inner_str = unsafe{v8_ValueAsString(self.inner_val)};
        V8LocalString {
            inner_string: inner_str,
        }
    }

    pub fn is_function(&self) -> bool {
        if unsafe{v8_ValueIsFunction(self.inner_val)} != 0 {true} else {false}
    }

    pub fn is_async_function(&self) -> bool {
        if unsafe{v8_ValueIsAsyncFunction(self.inner_val)} != 0 {true} else {false}
    }

    pub fn is_number(&self) -> bool {
        if unsafe{v8_ValueIsNumber(self.inner_val)} != 0 {true} else {false}
    }

    pub fn is_promise(&self) -> bool {
        if unsafe{v8_ValueIsPromise(self.inner_val)} != 0 {true} else {false}
    }

    pub fn as_promise(&self) -> V8LocalPromise {
        let inner_promise = unsafe{v8_ValueAsPromise(self.inner_val)};
        V8LocalPromise {
            inner_promise: inner_promise,
        }
    }

    pub fn is_object(&self) -> bool {
        if unsafe{v8_ValueIsObject(self.inner_val)} != 0 {true} else {false}
    }

    pub fn persist(&self, isolate: &V8Isolate) -> V8PersistValue {
        let inner_val = unsafe{v8_PersistValue(isolate.inner_isolate, self.inner_val)};
        V8PersistValue {
            inner_val: inner_val,
        }
    }

    pub fn call(&self, ctx: &V8ContextScope, args: Option<&[&V8LocalValue]>) -> Option<V8LocalValue> {
        let res = match args {
            Some(args) => {
                let args = args.into_iter().map(|v| v.inner_val).collect::<Vec<*mut v8_local_value>>();
                let ptr = args.as_ptr();
                unsafe{v8_FunctionCall(ctx.inner_ctx_ref, self.inner_val, args.len(), ptr)}
            }
            None => {
                unsafe{v8_FunctionCall(ctx.inner_ctx_ref, self.inner_val, 0, ptr::null())}
            }
        };

        if res.is_null() {
            None
        } else {
            Some(V8LocalValue {
                inner_val: res,
            })
        }
    }
}

impl V8PersistValue {
    pub fn as_local(&self, isolate: &V8Isolate) -> V8LocalValue {
        let inner_val = unsafe{v8_PersistedValueToLocal(isolate.inner_isolate, self.inner_val)};
        V8LocalValue {
            inner_val: inner_val,
        }
    }
}

impl Drop for V8LocalValue {
    fn drop(&mut self) {
        if !self.inner_val.is_null() {
            unsafe {v8_FreeValue(self.inner_val)}
        }
    }
}

impl Drop for V8PersistValue {
    fn drop(&mut self) {
        unsafe {v8_FreePersistedValue(self.inner_val)}
    }
}