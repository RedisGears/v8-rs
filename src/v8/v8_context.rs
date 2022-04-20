use crate::v8_c_raw::bindings::{
    v8_context,
    v8_FreeContext,
    v8_NewContext,
    v8_Compile,
    v8_ContextEnter,
};

use std::ptr;

use crate::v8::isolate::V8Isolate;
use crate::v8::v8_object::V8LocalObject;
use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_script::V8LocalScript;

pub struct V8Context {
    pub (crate) inner_ctx: *mut v8_context,
}

impl V8Context {
    pub fn new(isolate: &V8Isolate, globals: Option<&V8LocalObject>) -> V8Context {
        let inner_ctx = match globals {
            Some(g) => unsafe{v8_NewContext(isolate.inner_isolate, g.inner_obj)},
            None => unsafe{v8_NewContext(isolate.inner_isolate, ptr::null_mut())},
        };
        V8Context{
            inner_ctx: inner_ctx,
        }
    }

    pub fn enter<'a>(&'a self) -> V8ContextScope<'a> {
        unsafe{v8_ContextEnter(self.inner_ctx)};
        V8ContextScope{ctx: self}
    }

    pub fn compile(&self, s: &V8LocalString) -> Option<V8LocalScript>{
        let inner_script = unsafe{v8_Compile(self.inner_ctx, s.inner_string)};
        if inner_script.is_null() {
            None
        } else {
            Some(V8LocalScript{
                inner_script: inner_script,
            })
        }
    }
}

impl Drop for V8Context {
    fn drop(&mut self) {
        unsafe {v8_FreeContext(self.inner_ctx)}
    }
}