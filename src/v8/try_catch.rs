/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_FreeTryCatch, v8_TryCatchGetException, v8_TryCatchGetTrace, v8_TryCatchHasTerminated,
    v8_trycatch,
};

use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_value::V8LocalValue;

/// An object that responsible to catch any exception which raised
/// during the JS code invocation.
pub struct V8TryCatch<'isolate_scope, 'isolate> {
    pub(crate) inner_trycatch: *mut v8_trycatch,
    pub(crate) isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> V8TryCatch<'isolate_scope, 'isolate> {
    /// Return the exception that was raise during the JS code invocation.
    #[must_use]
    pub fn get_exception(&self) -> V8LocalValue<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_TryCatchGetException(self.inner_trycatch) };
        assert!(!inner_val.is_null());
        V8LocalValue {
            inner_val,
            isolate_scope: self.isolate_scope,
        }
    }

    /// Return the trace of the catch exception, the function returns Option because trace is not always provided.
    #[must_use]
    pub fn get_trace(
        &self,
        ctx_scope: &V8ContextScope,
    ) -> Option<V8LocalValue<'isolate_scope, 'isolate>> {
        let inner_val =
            unsafe { v8_TryCatchGetTrace(self.inner_trycatch, ctx_scope.inner_ctx_ref) };
        if inner_val.is_null() {
            return None;
        }
        Some(V8LocalValue {
            inner_val,
            isolate_scope: self.isolate_scope,
        })
    }

    #[must_use]
    pub fn has_terminated(&self) -> bool {
        let res = unsafe { v8_TryCatchHasTerminated(self.inner_trycatch) };
        res > 0
    }
}

impl<'isolate_scope, 'isolate> Drop for V8TryCatch<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeTryCatch(self.inner_trycatch) }
    }
}
