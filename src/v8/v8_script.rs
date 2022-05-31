use crate::v8_c_raw::bindings::{
    v8_FreePersistedScript, v8_FreeScript, v8_PersistedScriptToLocal, v8_Run, v8_ScriptPersist,
    v8_local_script, v8_persisted_script,
};

use crate::v8::isolate::V8Isolate;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_value::V8LocalValue;

/// JS script object
pub struct V8LocalScript {
    pub(crate) inner_script: *mut v8_local_script,
}

pub struct V8PersistedScript {
    pub(crate) inner_persisted_script: *mut v8_persisted_script,
}

impl V8LocalScript {
    /// Run the script
    #[must_use]
    pub fn run(&self, ctx: &V8ContextScope) -> Option<V8LocalValue> {
        let inner_val = unsafe { v8_Run(ctx.inner_ctx_ref, self.inner_script) };
        if inner_val.is_null() {
            None
        } else {
            Some(V8LocalValue { inner_val })
        }
    }

    pub fn persist(&self, isolate: &V8Isolate) -> V8PersistedScript {
        let inner_persisted_script =
            unsafe { v8_ScriptPersist(isolate.inner_isolate, self.inner_script) };
        V8PersistedScript {
            inner_persisted_script,
        }
    }
}

impl V8PersistedScript {
    pub fn to_local(&self, isolate: &V8Isolate) -> V8LocalScript {
        let inner_script = unsafe {
            v8_PersistedScriptToLocal(isolate.inner_isolate, self.inner_persisted_script)
        };
        V8LocalScript { inner_script }
    }
}

impl Drop for V8LocalScript {
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
