/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_ContextEnter, v8_FreeContext, v8_GetCurrentCtxRef, v8_GetPrivateData, v8_NewContext,
    v8_ResetPrivateData, v8_SetPrivateData, v8_context, v8_context_ref,
};
use crate::{RawIndex, UserIndex};

use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr::{self, NonNull};

use crate::v8::isolate::V8Isolate;
use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_object_template::V8LocalObjectTemplate;

/// An RAII data guard which resets the private data slot after going
/// out of scope.
pub struct V8ContextDataGuard<'context, 'data, T: 'data> {
    /// A raw index to reset after the guard goes out of scope.
    index: RawIndex,
    /// The context in which the guard should reset the variable.
    context: &'context V8Context,
    _phantom_data: PhantomData<&'data T>,
}
impl<'context, 'data, T: 'data> V8ContextDataGuard<'context, 'data, T> {
    /// Creates a new data guard with the provided index and context scope.
    pub(crate) fn new<I: Into<RawIndex>>(index: I, context: &'context V8Context) -> Self {
        let index = index.into();
        Self {
            index,
            context,
            _phantom_data: PhantomData,
        }
    }
}

impl<'context, 'data, T: 'data> Drop for V8ContextDataGuard<'context, 'data, T> {
    fn drop(&mut self) {
        self.context.reset_private_data_raw(self.index);
    }
}

#[derive(Debug)]
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

    pub(crate) fn get_current_raw_ref_for_isolate(
        isolate: &V8Isolate,
    ) -> Option<NonNull<v8_context_ref>> {
        NonNull::new(unsafe { v8_GetCurrentCtxRef(isolate.inner_isolate) })
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
        V8ContextScope::new(self.inner_ctx, true, isolate_scope, false)
    }

    /// Enter the context for debugging the JS code.
    pub fn debug_enter<'isolate_scope, 'isolate>(
        &self,
        isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
    ) -> V8ContextScope<'isolate_scope, 'isolate> {
        V8ContextScope::new(self.inner_ctx, true, isolate_scope, true)
    }

    /// Sets a private data on the context considering the index as
    /// a real data index.
    pub(crate) fn set_private_data_raw<T, I: Into<RawIndex>>(&self, index: I, data: &T) {
        let index = index.into().0;
        unsafe {
            v8_SetPrivateData(self.inner_ctx, index, data as *const T as *mut c_void);
        };
    }

    /// Sets a private data on the context that can later be retieved with `get_private_data`.
    #[must_use]
    pub fn set_private_data<'context, 'data, T, I: Into<UserIndex>>(
        &'context self,
        index: I,
        data: &'data T,
    ) -> V8ContextDataGuard<'context, 'data, T> {
        let index = index.into();
        self.set_private_data_raw(index, data);
        V8ContextDataGuard::new(index, self)
    }

    /// Resets a private data on the context considering the index as
    /// a real data index.
    pub(crate) fn reset_private_data_raw<I: Into<RawIndex>>(&self, index: I) {
        let index = index.into().0;
        unsafe {
            v8_ResetPrivateData(self.inner_ctx, index);
        };
    }

    /// Resets a private data on the context considering the index as
    /// a user data index.
    pub fn reset_private_data<I: Into<UserIndex>>(&self, index: I) {
        let index = index.into();
        self.reset_private_data_raw(index)
    }

    #[must_use]
    pub(crate) fn get_private_data_raw<T, I: Into<RawIndex>>(&self, index: I) -> Option<&T> {
        let index = index.into().0;
        let pd = unsafe { v8_GetPrivateData(self.inner_ctx, index) } as *const T;
        unsafe { pd.as_ref() }
    }

    /// Return the private data that was set using [`Self::set_private_data`].
    #[must_use]
    pub fn get_private_data<T>(&self, index: UserIndex) -> Option<&T> {
        self.get_private_data_raw(index)
    }
}

impl Drop for V8Context {
    fn drop(&mut self) {
        unsafe { v8_FreeContext(self.inner_ctx) }
    }
}
