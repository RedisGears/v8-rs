use crate::v8_c_raw::bindings::{
    v8_local_script,
    v8_FreeScript,
    v8_Run,
};

use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_value::V8LocalValue;

pub struct V8LocalScript {
    pub (crate) inner_script: *mut v8_local_script,
}

impl V8LocalScript {
    pub fn run(&self, ctx: &V8ContextScope) -> V8LocalValue{
        let inner_val = unsafe{v8_Run(ctx.ctx.inner_ctx, self.inner_script)};
        V8LocalValue {
            inner_val: inner_val,
        }
    }
}

impl Drop for V8LocalScript {
    fn drop(&mut self) {
        unsafe {v8_FreeScript(self.inner_script)}
    }
}