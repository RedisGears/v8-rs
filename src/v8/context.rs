/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! See [Context].

use crate::v8_c_raw::bindings::{
    v8_ContextEnter, v8_FreeContext, v8_GetPrivateData, v8_NewContext, v8_ResetPrivateData,
    v8_SetPrivateData, v8_context,
};
use crate::{RawIndex, UserIndex};

use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr;

use crate::v8::context_scope::ContextScope;
use crate::v8::isolate::Isolate;
use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::object_template::LocalObjectTemplate;

/// An RAII data guard which resets the private data slot after going
/// out of scope.
pub struct ContextDataGuard<'context, 'data, T: 'data> {
    /// A raw index to reset after the guard goes out of scope.
    index: RawIndex,
    /// The context in which the guard should reset the variable.
    context: &'context Context,
    _phantom_data: PhantomData<&'data T>,
}
impl<'context, 'data, T: 'data> ContextDataGuard<'context, 'data, T> {
    /// Creates a new data guard with the provided index and context scope.
    pub(crate) fn new<I: Into<RawIndex>>(index: I, context: &'context Context) -> Self {
        let index = index.into();
        Self {
            index,
            context,
            _phantom_data: PhantomData,
        }
    }
}

impl<'context, 'data, T: 'data> Drop for ContextDataGuard<'context, 'data, T> {
    fn drop(&mut self) {
        self.context.reset_private_data_raw(self.index);
    }
}

/// A sandboxed execution context with its own set of built-in objects
/// and functions.
pub struct Context {
    pub(crate) inner_ctx: *mut v8_context,
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

impl Context {
    pub(crate) fn new(isolate: &Isolate, globals: Option<&LocalObjectTemplate>) -> Self {
        let inner_ctx = match globals {
            Some(g) => unsafe { v8_NewContext(isolate.inner_isolate, g.0.inner_val) },
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
        isolate_scope: &'isolate_scope IsolateScope<'isolate>,
    ) -> ContextScope<'isolate_scope, 'isolate> {
        let inner_ctx_ref = unsafe { v8_ContextEnter(self.inner_ctx) };
        ContextScope {
            inner_ctx_ref,
            exit_on_drop: true,
            isolate_scope,
        }
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
    ) -> ContextDataGuard<'context, 'data, T> {
        let index = index.into();
        self.set_private_data_raw(index, data);
        ContextDataGuard::new(index, self)
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

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { v8_FreeContext(self.inner_ctx) }
    }
}
