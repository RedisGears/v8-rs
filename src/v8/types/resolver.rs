/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_FreeResolver, v8_ResolverGetPromise, v8_ResolverReject, v8_ResolverResolve,
    v8_ResolverToValue, v8_local_resolver,
};

use crate::v8::context_scope::ContextScope;
use crate::v8::types::promise::LocalPromise;
use crate::v8::types::ScopedValue;

use super::any::LocalValueAny;

/// JS resolver object
pub struct LocalResolver<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_local_resolver>,
);

impl<'isolate_scope, 'isolate> LocalResolver<'isolate_scope, 'isolate> {
    /// Get the promise object assosiated with this resolver.
    pub fn get_promise(&self) -> LocalPromise<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ResolverGetPromise(self.0.inner_val) };
        LocalPromise(ScopedValue {
            inner_val,
            isolate_scope: self.0.isolate_scope,
        })
    }

    /// Resolve the resolver with the given JS value.
    pub fn resolve(&self, ctx_scope: &ContextScope, val: &LocalValueAny) {
        unsafe { v8_ResolverResolve(ctx_scope.inner_ctx_ref, self.0.inner_val, val.0.inner_val) };
    }

    /// Reject the resolver with the given JS value.
    pub fn reject(&self, ctx_scope: &ContextScope, val: &LocalValueAny) {
        unsafe { v8_ResolverReject(ctx_scope.inner_ctx_ref, self.0.inner_val, val.0.inner_val) };
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalResolver<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeResolver(self.0.inner_val) }
    }
}

impl<'isolate_scope, 'isolate> From<LocalResolver<'isolate_scope, 'isolate>>
    for LocalValueAny<'isolate_scope, 'isolate>
{
    fn from(value: LocalResolver<'isolate_scope, 'isolate>) -> Self {
        let inner_val = unsafe { v8_ResolverToValue(value.0.inner_val) };
        Self(ScopedValue {
            inner_val,
            isolate_scope: value.0.isolate_scope,
        })
    }
}
