/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8::isolate_scope::IsolateScope;
use crate::v8_c_raw::bindings::{
    v8_FreeTryCatch, v8_NewTryCatch, v8_TryCatchGetException, v8_TryCatchGetTrace,
    v8_TryCatchHasTerminated, v8_trycatch,
};

use crate::v8::context_scope::ContextScope;
use crate::v8::types::ScopedValue;

use super::any::LocalValueAny;

/// An object that responsible to catch any exception which raised
/// during the JS code invocation.
#[derive(Debug, Clone)]
pub struct TryCatch<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_trycatch>,
);

impl<'isolate_scope, 'isolate> TryCatch<'isolate_scope, 'isolate> {
    /// Creates a new JavaScript try-catch within the passed [IsolateScope].
    pub fn new(isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        let inner_val = unsafe { v8_NewTryCatch(isolate_scope.isolate.inner_isolate) };

        Self(ScopedValue {
            inner_val,
            isolate_scope,
        })
    }

    /// Return the exception that was raise during the JS code invocation.
    ///
    /// # Panics
    ///
    /// Panics when the exception pointer is null.
    pub fn get_exception(&self) -> LocalValueAny<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_TryCatchGetException(self.0.inner_val) };
        assert!(!inner_val.is_null());
        LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope: self.0.isolate_scope,
        })
    }

    /// Return the trace of the catch exception, the function returns Option because trace is not always provided.
    pub fn get_trace(
        &self,
        ctx_scope: &ContextScope,
    ) -> Option<LocalValueAny<'isolate_scope, 'isolate>> {
        let inner_val = unsafe { v8_TryCatchGetTrace(self.0.inner_val, ctx_scope.inner_ctx_ref) };
        if inner_val.is_null() {
            return None;
        }
        Some(LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope: self.0.isolate_scope,
        }))
    }

    pub fn has_terminated(&self) -> bool {
        let res = unsafe { v8_TryCatchHasTerminated(self.0.inner_val) };
        res > 0
    }
}

impl<'isolate_scope, 'isolate> Drop for TryCatch<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeTryCatch(self.0.inner_val) }
    }
}
