/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_FreeResolver, v8_ResolverGetPromise, v8_ResolverReject, v8_ResolverResolve,
    v8_ResolverToValue, v8_local_resolver,
};

use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_promise::V8LocalPromise;
use crate::v8::v8_value::V8LocalValue;

/// JS resolver object
pub struct V8LocalResolver<'isolate_scope, 'isolate> {
    pub(crate) inner_resolver: *mut v8_local_resolver,
    pub(crate) isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> V8LocalResolver<'isolate_scope, 'isolate> {
    /// Get the promise object assosiated with this resolver.
    #[must_use]
    pub fn get_promise(&self) -> V8LocalPromise<'isolate_scope, 'isolate> {
        let inner_promise = unsafe { v8_ResolverGetPromise(self.inner_resolver) };
        V8LocalPromise {
            inner_promise,
            isolate_scope: self.isolate_scope,
        }
    }

    /// Resolve the resolver with the given JS value.
    pub fn resolve(&self, ctx_scope: &V8ContextScope, val: &V8LocalValue) {
        unsafe { v8_ResolverResolve(ctx_scope.inner_ctx_ref, self.inner_resolver, val.inner_val) };
    }

    /// Reject the resolver with the given JS value.
    pub fn reject(&self, ctx_scope: &V8ContextScope, val: &V8LocalValue) {
        unsafe { v8_ResolverReject(ctx_scope.inner_ctx_ref, self.inner_resolver, val.inner_val) };
    }

    /// Convert the resolver into a generic JS value.
    #[must_use]
    pub fn to_value(&self) -> V8LocalValue<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ResolverToValue(self.inner_resolver) };
        V8LocalValue {
            inner_val,
            isolate_scope: self.isolate_scope,
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for V8LocalResolver<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeResolver(self.inner_resolver) }
    }
}
