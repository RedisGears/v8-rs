/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::data_index;
use crate::v8_c_raw::bindings::{
    v8_ContextEnter, v8_FreeContext, v8_GetPrivateData, v8_NewContext, v8_ResetPrivateData,
    v8_SetPrivateData, v8_context,
};

use std::os::raw::c_void;
use std::ptr;

use crate::v8::isolate::V8Isolate;
use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_object_template::V8LocalObjectTemplate;

pub struct V8Context {
    pub(crate) inner_ctx: *mut v8_context,
}

unsafe impl Sync for V8Context {}
unsafe impl Send for V8Context {}

impl V8Context {
    pub(crate) fn new(isolate: &V8Isolate, globals: Option<&V8LocalObjectTemplate>) -> Self {
        let inner_ctx = match globals {
            Some(g) => unsafe { v8_NewContext(isolate.inner_isolate, g.inner_obj) },
            None => unsafe { v8_NewContext(isolate.inner_isolate, ptr::null_mut()) },
        };
        Self { inner_ctx }
    }

    /// Enter the context for JS code invocation.
    /// Returns a `V8ContextScope` object. The context will
    /// be automatically exit when the returned `V8ContextScope`
    /// will be destroyed.
    #[must_use]
    pub fn enter<'isolate_scope, 'isolate>(
        &self,
        isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
    ) -> V8ContextScope<'isolate_scope, 'isolate> {
        let inner_ctx_ref = unsafe { v8_ContextEnter(self.inner_ctx) };
        V8ContextScope {
            inner_ctx_ref,
            exit_on_drop: true,
            isolate_scope,
        }
    }

    /// Set a private data on the context that can later be retieved with `get_private_data`.
    pub fn set_private_data<T>(&self, index: usize, data: &T) {
        unsafe {
            v8_SetPrivateData(
                self.inner_ctx,
                data_index!(index),
                data as *const T as *mut c_void,
            );
        };
    }

    /// Reset a private data on the context.
    pub fn reset_private_data(&self, index: usize) {
        unsafe {
            v8_ResetPrivateData(self.inner_ctx, data_index!(index));
        };
    }

    /// Return the private data that was set using `set_private_data`
    #[must_use]
    pub fn get_private_data<T>(&self, index: usize) -> Option<&T> {
        let pd = unsafe { v8_GetPrivateData(self.inner_ctx, data_index!(index)) } as *const T;
        unsafe { pd.as_ref() }
    }
}

impl Drop for V8Context {
    fn drop(&mut self) {
        unsafe { v8_FreeContext(self.inner_ctx) }
    }
}
