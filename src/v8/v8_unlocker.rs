/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

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
