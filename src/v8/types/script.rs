/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! The JavaScript script facilities.

use crate::v8_c_raw::bindings::{
    v8_FreePersistedScript, v8_FreeScript, v8_PersistedScriptToLocal, v8_Run, v8_ScriptPersist,
    v8_local_script, v8_persisted_script,
};

use crate::v8::context_scope::ContextScope;
use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::ScopedValue;

use super::any::LocalValueAny;
use super::Value;

/// A JavaScript script object.
pub struct LocalScript<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_local_script>,
);

/// A persisted script is script that isn't tied to the [IsolateScope]'s
/// lifetime it was created for.
pub struct PersistedScript {
    pub(crate) inner_persisted_script: *mut v8_persisted_script,
}

impl<'isolate_scope, 'isolate> LocalScript<'isolate_scope, 'isolate> {
    /// Run the script
    #[must_use]
    pub fn run(&self, ctx: &ContextScope) -> Option<Value<'isolate_scope, 'isolate>> {
        let inner_val = unsafe { v8_Run(ctx.inner_ctx_ref, self.0.inner_val) };
        if inner_val.is_null() {
            None
        } else {
            Some(
                LocalValueAny(ScopedValue {
                    inner_val,
                    isolate_scope: self.0.isolate_scope,
                })
                .into(),
            )
        }
    }

    /// Persists the [LocalScript] so that it can outlive the
    /// [IsolateScope] it was tied to.
    pub fn persist(&self) -> PersistedScript {
        let inner_persisted_script = unsafe {
            v8_ScriptPersist(self.0.isolate_scope.isolate.inner_isolate, self.0.inner_val)
        };
        PersistedScript {
            inner_persisted_script,
        }
    }
}

impl PersistedScript {
    /// Converts the [PersistedScript] into a [LocalScript] for the
    /// provided [IsolateScope].
    pub fn to_local<'isolate_scope, 'isolate>(
        &self,
        isolate_scope: &'isolate_scope IsolateScope<'isolate>,
    ) -> LocalScript<'isolate_scope, 'isolate> {
        let inner_val = unsafe {
            v8_PersistedScriptToLocal(
                isolate_scope.isolate.inner_isolate,
                self.inner_persisted_script,
            )
        };
        LocalScript(ScopedValue {
            inner_val,
            isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalScript<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeScript(self.0.inner_val) }
    }
}

unsafe impl Sync for PersistedScript {}
unsafe impl Send for PersistedScript {}

impl Drop for PersistedScript {
    fn drop(&mut self) {
        unsafe { v8_FreePersistedScript(self.inner_persisted_script) }
    }
}
