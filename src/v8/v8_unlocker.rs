use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8_c_raw::bindings::{v8_FreeUnlocker, v8_unlocker};

pub struct V8Unlocker<'isolate_scope, 'isolate> {
    pub(crate) inner_unlocker: *mut v8_unlocker,
    pub(crate) _isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> Drop for V8Unlocker<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeUnlocker(self.inner_unlocker) };
    }
}
