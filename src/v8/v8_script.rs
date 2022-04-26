use crate::v8_c_raw::bindings::{v8_FreeScript, v8_Run, v8_local_script};

use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_value::V8LocalValue;

/// JS script object
pub struct V8LocalScript {
    pub(crate) inner_script: *mut v8_local_script,
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
}

impl Drop for V8LocalScript {
    fn drop(&mut self) {
        unsafe { v8_FreeScript(self.inner_script) }
    }
}
