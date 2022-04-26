use crate::v8_c_raw::bindings::{
    v8_FreeResolver, v8_ResolverGetPromise, v8_ResolverReject, v8_ResolverResolve,
    v8_ResolverToValue, v8_local_resolver,
};

use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_promise::V8LocalPromise;
use crate::v8::v8_value::V8LocalValue;

/// JS resolver object
pub struct V8LocalResolver {
    pub(crate) inner_resolver: *mut v8_local_resolver,
}

impl V8LocalResolver {
    /// Get the promise object assosiated with this resolver.
    #[must_use]
    pub fn get_promise(&self) -> V8LocalPromise {
        let inner_promise = unsafe { v8_ResolverGetPromise(self.inner_resolver) };
        V8LocalPromise { inner_promise }
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
    pub fn to_value(&self) -> V8LocalValue {
        let inner_val = unsafe { v8_ResolverToValue(self.inner_resolver) };
        V8LocalValue { inner_val }
    }
}

impl Drop for V8LocalResolver {
    fn drop(&mut self) {
        unsafe { v8_FreeResolver(self.inner_resolver) }
    }
}
