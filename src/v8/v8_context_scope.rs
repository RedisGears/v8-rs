use crate::v8_c_raw::bindings::{
    v8_context_ref,
    v8_Compile,
    v8_FreeContextRef,
    v8_GetPrivateDataFromCtxRef,
};

use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_script::V8LocalScript;

pub struct V8ContextScope {
    pub (crate) inner_ctx_ref: *mut v8_context_ref,
    pub (crate) exit_on_drop: bool,
}

impl V8ContextScope {
    pub fn compile(&self, s: &V8LocalString) -> Option<V8LocalScript>{
        let inner_script = unsafe{v8_Compile(self.inner_ctx_ref, s.inner_string)};
        if inner_script.is_null() {
            None
        } else {
            Some(V8LocalScript{
                inner_script: inner_script,
            })
        }
    }

    pub fn get_private_data<T>(&self, index: usize) -> Option<&T> {
        let pd = unsafe{v8_GetPrivateDataFromCtxRef(self.inner_ctx_ref, index)};
        if pd.is_null() {
            None
        } else {
            Some(unsafe{&*(pd as *const T)})
        }
    }

    pub fn get_private_data_mut<T>(&self, index: usize) -> Option<&mut T> {
        let pd = unsafe{v8_GetPrivateDataFromCtxRef(self.inner_ctx_ref, index)};
        if pd.is_null() {
            None
        } else {
            Some(unsafe{&mut *(pd as *mut T)})
        }
    }
}

impl Drop for V8ContextScope {
    fn drop(&mut self) {
        if self.exit_on_drop {
            unsafe {v8_FreeContextRef(self.inner_ctx_ref)}
        }
    }
}