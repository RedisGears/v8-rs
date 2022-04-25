use crate::v8_c_raw::bindings::{
    v8_context,
    v8_FreeContext,
    v8_NewContext,
    v8_ContextEnter,
    v8_SetPrivateData,
    v8_GetPrivateData,
};

use std::ptr;
use std::os::raw::{c_void};

use crate::v8::isolate::V8Isolate;
use crate::v8::v8_object_template::V8LocalObjectTemplate;
use crate::v8::v8_context_scope::V8ContextScope;

pub struct V8Context {
    pub (crate) inner_ctx: *mut v8_context,
}

impl V8Context {
    pub fn new(isolate: &V8Isolate, globals: Option<&V8LocalObjectTemplate>) -> V8Context {
        let inner_ctx = match globals {
            Some(g) => unsafe{v8_NewContext(isolate.inner_isolate, g.inner_obj)},
            None => unsafe{v8_NewContext(isolate.inner_isolate, ptr::null_mut())},
        };
        V8Context{
            inner_ctx: inner_ctx,
        }
    }

    pub fn enter(&self) -> V8ContextScope {
        let inner_ctx_ref = unsafe{v8_ContextEnter(self.inner_ctx)};
        V8ContextScope{
            inner_ctx_ref: inner_ctx_ref,
            exit_on_drop: true,
        }
    }

    pub fn set_private_data<T>(&self, index: usize, pd: Option<&T>) {
        unsafe{v8_SetPrivateData(self.inner_ctx, index, if pd.is_none() {ptr::null_mut()} else {pd.unwrap() as *const T as *mut c_void})};
    }

    pub fn get_private_data<T>(&self, index: usize) -> Option<&T> {
        let pd = unsafe{v8_GetPrivateData(self.inner_ctx, index)};
        if pd.is_null() {
            None
        } else {
            Some(unsafe{&*(pd as *const T)})
        }
        
    }
}

impl Drop for V8Context {
    fn drop(&mut self) {
        unsafe {v8_FreeContext(self.inner_ctx)}
    }
}