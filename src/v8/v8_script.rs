use crate::v8_c_raw::bindings::{
    v8_FreePersistedScript, v8_FreeScript, v8_PersistedScriptToLocal, v8_Run, v8_ScriptPersist,
    v8_local_script, v8_persisted_script,
};

use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_value::V8LocalValue;

/// JS script object
pub struct V8LocalScript<'isolate_scope, 'isolate> {
    pub(crate) inner_script: *mut v8_local_script,
    pub(crate) isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
}

pub struct V8PersistedScript {
    pub(crate) inner_persisted_script: *mut v8_persisted_script,
}

impl<'isolate_scope, 'isolate> V8LocalScript<'isolate_scope, 'isolate> {
    /// Run the script
    #[must_use]
    pub fn run(&self, ctx: &V8ContextScope) -> Option<V8LocalValue<'isolate_scope, 'isolate>> {
        let inner_val = unsafe { v8_Run(ctx.inner_ctx_ref, self.inner_script) };
        if inner_val.is_null() {
            None
        } else {
            Some(V8LocalValue {
                inner_val: inner_val,
                isolate_scope: self.isolate_scope,
            })
        }
    }

    pub fn persist(&self) -> V8PersistedScript {
        let inner_persisted_script = unsafe {
            v8_ScriptPersist(self.isolate_scope.isolate.inner_isolate, self.inner_script)
        };
        V8PersistedScript {
            inner_persisted_script,
        }
    }
}

impl V8PersistedScript {
    pub fn to_local<'isolate_scope, 'isolate>(
        &self,
        isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
    ) -> V8LocalScript<'isolate_scope, 'isolate> {
        let inner_script = unsafe {
            v8_PersistedScriptToLocal(
                isolate_scope.isolate.inner_isolate,
                self.inner_persisted_script,
            )
        };
        V8LocalScript {
            inner_script,
            isolate_scope: isolate_scope,
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for V8LocalScript<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeScript(self.inner_script) }
    }
}

unsafe impl Sync for V8PersistedScript {}
unsafe impl Send for V8PersistedScript {}

impl Drop for V8PersistedScript {
    fn drop(&mut self) {
        unsafe { v8_FreePersistedScript(self.inner_persisted_script) }
    }
}
