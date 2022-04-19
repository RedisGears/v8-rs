use crate::v8_c_raw::bindings::{
    v8_ContextExit,
};

use crate::v8::v8_context::V8Context;

pub struct V8ContextScope<'a> {
    pub (crate) ctx: &'a V8Context,
}

impl<'a> Drop for V8ContextScope<'a> {
    fn drop(&mut self) {
        unsafe {v8_ContextExit(self.ctx.inner_ctx)}
    }
}