use crate::v8_c_raw::bindings::{
    v8_local_resolver,
    v8_FreeResolver,
    v8_ResolverGetPromise,
    v8_ResolverResolve,
    v8_ResolverReject,
    v8_ResolverToValue,
};

use crate::v8::v8_promise::V8LocalPromise;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_value::V8LocalValue;

pub struct V8LocalResolver {
    pub (crate) inner_resolver: *mut v8_local_resolver,
}

impl V8LocalResolver {
    pub fn get_promise(&self) -> V8LocalPromise {
        let inner_promise = unsafe{v8_ResolverGetPromise(self.inner_resolver)};
        V8LocalPromise {
            inner_promise: inner_promise,
        }
    }

    pub fn resolve(&self, ctx_scope: &V8ContextScope, val: &V8LocalValue) {
        unsafe{v8_ResolverResolve(ctx_scope.inner_ctx_ref, self.inner_resolver, val.inner_val)};
    }

    pub fn reject(&self, ctx_scope: &V8ContextScope, val: &V8LocalValue) {
        unsafe{v8_ResolverReject(ctx_scope.inner_ctx_ref, self.inner_resolver, val.inner_val)};
    }

    pub fn to_value(&self) -> V8LocalValue {
        let inner_val = unsafe{v8_ResolverToValue(self.inner_resolver)};
        V8LocalValue {
            inner_val: inner_val,
        }
    }
}

impl Drop for V8LocalResolver {
    fn drop(&mut self) {
        unsafe {v8_FreeResolver(self.inner_resolver)}
    }
}