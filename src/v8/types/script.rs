/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_FreePersistedScript, v8_FreeScript, v8_PersistedScriptToLocal, v8_Run, v8_ScriptPersist,
    v8_local_script, v8_persisted_script,
};

use crate::v8::context_scope::ContextScope;
use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::LocalValueGeneric;

/// JS script object
pub struct LocalScript<'isolate_scope, 'isolate> {
    pub(crate) inner_script: *mut v8_local_script,
    pub(crate) isolate_scope: &'isolate_scope IsolateScope<'isolate>,
}

pub struct PersistedScript {
    pub(crate) inner_persisted_script: *mut v8_persisted_script,
}

impl<'isolate_scope, 'isolate> LocalScript<'isolate_scope, 'isolate> {
    /// Run the script
    #[must_use]
    pub fn run(&self, ctx: &ContextScope) -> Option<LocalValueGeneric<'isolate_scope, 'isolate>> {
        let inner_val = unsafe { v8_Run(ctx.inner_ctx_ref, self.inner_script) };
        if inner_val.is_null() {
            None
        } else {
            Some(LocalValueGeneric {
                inner_val,
                isolate_scope: self.isolate_scope,
            })
        }
    }

    pub fn persist(&self) -> PersistedScript {
        let inner_persisted_script = unsafe {
            v8_ScriptPersist(self.isolate_scope.isolate.inner_isolate, self.inner_script)
        };
        PersistedScript {
            inner_persisted_script,
        }
    }
}

impl PersistedScript {
    pub fn to_local<'isolate_scope, 'isolate>(
        &self,
        isolate_scope: &'isolate_scope IsolateScope<'isolate>,
    ) -> LocalScript<'isolate_scope, 'isolate> {
        let inner_script = unsafe {
            v8_PersistedScriptToLocal(
                isolate_scope.isolate.inner_isolate,
                self.inner_persisted_script,
            )
        };
        LocalScript {
            inner_script,
            isolate_scope,
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalScript<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeScript(self.inner_script) }
    }
}

unsafe impl Sync for PersistedScript {}
unsafe impl Send for PersistedScript {}

impl Drop for PersistedScript {
    fn drop(&mut self) {
        unsafe { v8_FreePersistedScript(self.inner_persisted_script) }
    }
}
