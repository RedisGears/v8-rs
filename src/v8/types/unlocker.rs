/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8::isolate_scope::IsolateScope;
use crate::v8_c_raw::bindings::{v8_FreeUnlocker, v8_NewUnlocker, v8_unlocker};

use super::ScopedValue;

/// TODO add comment.
#[derive(Debug, Clone)]
pub struct Unlocker<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_unlocker>,
);

impl<'isolate_scope, 'isolate> Unlocker<'isolate_scope, 'isolate> {
    /// Creates a new [Unlocker] within the provided [IsolateScope].
    pub fn new(isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        let inner_val = unsafe { v8_NewUnlocker(isolate_scope.isolate.inner_isolate) };
        Self(ScopedValue {
            inner_val,
            isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> Drop for Unlocker<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeUnlocker(self.0.inner_val) };
    }
}
