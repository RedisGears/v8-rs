use crate::v8_c_raw::bindings::{
    v8_handlers_scope,
    v8_NewHandlersScope,
    v8_FreeHandlersScope,
};

use crate::v8::isolate::V8Isolate;

pub struct V8HandlersScope<'a> {
    _isolate: &'a V8Isolate,
    inner_handlers_scope: *mut v8_handlers_scope,
}

impl<'a> V8HandlersScope<'a> {
    pub (crate) fn new(isolate: &'a V8Isolate) -> V8HandlersScope<'a> {
        let inner_handlers_scope = unsafe{v8_NewHandlersScope(isolate.inner_isolate)};
        V8HandlersScope {
            _isolate: isolate,
            inner_handlers_scope: inner_handlers_scope,
        }
    }
}

impl<'a> Drop for V8HandlersScope<'a> {
    fn drop(&mut self) {
        unsafe {v8_FreeHandlersScope(self.inner_handlers_scope)}
    }
}
