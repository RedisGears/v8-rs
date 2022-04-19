use crate::v8_c_raw::bindings::{
    v8_isolate_scope,
    v8_IsolateEnter,
    v8_IsolateExit,
};

use crate::v8::isolate::V8Isolate;
use crate::v8::v8_context::V8Context;
use crate::v8::v8_object::V8LocalObject;

pub struct V8IsolateScope<'a> {
    isolate: &'a V8Isolate,
    inner_isolate_scope: *mut v8_isolate_scope,
}

impl<'a> V8IsolateScope<'a> {
    pub fn new(isolate: &'a V8Isolate) -> V8IsolateScope<'a> {
        let inner_isolate_scope = unsafe{v8_IsolateEnter(isolate.inner_isolate)};
        V8IsolateScope {
            isolate: isolate,
            inner_isolate_scope: inner_isolate_scope,
        }
    }

    pub fn new_context(&self, globals: Option<&V8LocalObject>) -> V8Context {
        V8Context::new(self.isolate, globals)
    }
}

impl<'a> Drop for V8IsolateScope<'a> {
    fn drop(&mut self) {
        unsafe {v8_IsolateExit(self.inner_isolate_scope)}
    }
}
