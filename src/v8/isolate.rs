use crate::v8_c_raw::bindings::{
    v8_isolate,
    v8_NewIsolate,
    v8_FreeIsolate,
};

use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::handler_scope::V8HandlersScope;

pub struct V8Isolate {
    pub (crate) inner_isolate: *mut v8_isolate,
}

impl V8Isolate {
    pub fn new() -> V8Isolate {
        let inner_isolate = unsafe{v8_NewIsolate()};
        V8Isolate {
            inner_isolate: inner_isolate,
        }
    }

    pub fn enter(&self) -> V8IsolateScope {
        V8IsolateScope::new(self)
    }

    pub fn new_handlers_scope(&self) -> V8HandlersScope {
        V8HandlersScope::new(self)
    }
}

impl Drop for V8Isolate {
    fn drop(&mut self) {
        unsafe {v8_FreeIsolate(self.inner_isolate)}
    }
}
