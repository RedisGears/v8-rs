/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! Contains the native function template facilities.

use crate::v8_c_raw::bindings::v8_NewNativeFunctionTemplate;
use crate::v8_c_raw::bindings::{
    v8_ArgsGet, v8_ArgsGetSelf, v8_FreeNativeFunctionTemplate, v8_GetCurrentCtxRef,
    v8_GetCurrentIsolate, v8_NativeFunctionTemplateToFunction, v8_local_native_function_template,
    v8_local_value, v8_local_value_arr,
};

use std::os::raw::c_void;
use std::ptr;

use crate::v8::context_scope::ContextScope;
use crate::v8::isolate::Isolate;
use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::native_function::LocalNativeFunction;
use crate::v8::types::LocalObject;
use crate::v8::types::ScopedValue;

use super::any::LocalValueAny;
use super::Value;

/// Native function template object.
///
/// A [LocalNativeFunctionTemplate] is used to create functions at
/// runtime. There can only be one function created from a
/// [LocalNativeFunctionTemplate] in a context. The lifetime of the
/// created function is equal to the lifetime of the context. So in case
/// the embedder needs to create temporary functions that can be
/// collected using Scripts is preferred.
#[derive(Debug, Clone)]
pub struct LocalNativeFunctionTemplate<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_local_native_function_template>,
);

impl<'isolate_scope, 'isolate> LocalNativeFunctionTemplate<'isolate_scope, 'isolate> {
    /// Creates a new local native function template within the
    /// provided [IsolateScope].
    pub fn new<
        T: for<'d, 'e> Fn(
            &LocalNativeFunctionArgs<'d, 'e>,
            &'d IsolateScope<'e>,
            &ContextScope<'d, 'e>,
        ) -> Option<LocalValueAny<'d, 'e>>,
    >(
        function: T,
        isolate_scope: &'isolate_scope IsolateScope<'isolate>,
    ) -> Self {
        let inner_val = unsafe {
            v8_NewNativeFunctionTemplate(
                isolate_scope.isolate.inner_isolate,
                Some(native_basic_function::<T>),
                Box::into_raw(Box::new(function)).cast(),
                Some(free_pd::<T>),
            )
        };

        Self(ScopedValue {
            inner_val,
            isolate_scope,
        })
    }
}

/// An array of arguments for a [LocalNativeFunctionTemplate].
pub struct LocalNativeFunctionArgs<'isolate_scope, 'isolate> {
    pub(crate) inner_arr: *mut v8_local_value_arr,
    len: usize,
    isolate_scope: &'isolate_scope IsolateScope<'isolate>,
}

pub(crate) extern "C" fn free_pd<
    T: for<'d, 'c> Fn(
        &LocalNativeFunctionArgs<'d, 'c>,
        &'d IsolateScope<'c>,
        &ContextScope<'d, 'c>,
    ) -> Option<LocalValueAny<'d, 'c>>,
>(
    pd: *mut c_void,
) {
    unsafe {
        let _ = Box::from_raw(pd.cast::<T>());
    }
}

pub(crate) extern "C" fn native_basic_function<
    T: for<'d, 'c> Fn(
        &LocalNativeFunctionArgs<'d, 'c>,
        &'d IsolateScope<'c>,
        &ContextScope<'d, 'c>,
    ) -> Option<LocalValueAny<'d, 'c>>,
>(
    args: *mut v8_local_value_arr,
    len: usize,
    pd: *mut c_void,
) -> *mut v8_local_value {
    let func = unsafe { &*(pd.cast::<T>()) };

    let inner_isolate = unsafe { v8_GetCurrentIsolate(args) };
    let isolate = Isolate {
        inner_isolate,
        no_release: true,
    };

    let isolate_scope = IsolateScope::new(&isolate);

    let inner_ctx_ref = unsafe { v8_GetCurrentCtxRef(inner_isolate) };
    let ctx_scope = ContextScope {
        inner_ctx_ref,
        exit_on_drop: false,
        isolate_scope: &isolate_scope,
    };

    let args = LocalNativeFunctionArgs {
        inner_arr: args,
        len,
        isolate_scope: &isolate_scope,
    };

    let res = func(&args, &isolate_scope, &ctx_scope);

    match res {
        Some(mut r) => {
            let inner_val = r.0.inner_val;
            r.0.inner_val = ptr::null_mut();
            inner_val
        }
        None => ptr::null_mut(),
    }
}

impl<'isolate_scope, 'isolate> LocalNativeFunctionTemplate<'isolate_scope, 'isolate> {
    /// Builds (instantiates) a [LocalNativeFunction] out of this
    /// template.
    pub fn build(&self, ctx_scope: &ContextScope) -> LocalNativeFunction<'isolate_scope, 'isolate> {
        let inner_val = unsafe {
            v8_NativeFunctionTemplateToFunction(ctx_scope.inner_ctx_ref, self.0.inner_val)
        };
        LocalNativeFunction(ScopedValue {
            inner_val,
            isolate_scope: self.0.isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> LocalNativeFunctionArgs<'isolate_scope, 'isolate> {
    /// Return the i-th argument from the native function args
    /// # Panics
    #[must_use]
    pub fn get(&self, i: usize) -> Value<'isolate_scope, 'isolate> {
        assert!(i <= self.len);
        let val = unsafe { v8_ArgsGet(self.inner_arr, i) };
        LocalValueAny(ScopedValue {
            inner_val: val,
            isolate_scope: self.isolate_scope,
        })
        .into()
    }

    /// Return the amount of arguments passed to the native function
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Checks if the list of args is empty
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Checks if the list of args is empty
    #[must_use]
    pub fn get_self(&self) -> LocalObject<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ArgsGetSelf(self.inner_arr) };
        LocalObject(ScopedValue {
            inner_val,
            isolate_scope: self.isolate_scope,
        })
    }

    // pub const fn persist(&self) {}

    /// Returns an iterator [LocalNativeFunctionArgsIter] over the
    /// function arguments.
    pub fn iter<'a>(&'a self) -> LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a> {
        LocalNativeFunctionArgsIter {
            args: self,
            index: 0,
        }
    }
}

/// An iterator over the function arguments of a [LocalNativeFunction].
pub struct LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a> {
    args: &'a LocalNativeFunctionArgs<'isolate_scope, 'isolate>,
    index: usize,
}

impl<'isolate_scope, 'isolate, 'a> Iterator
    for LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a>
{
    type Item = Value<'isolate_scope, 'isolate>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.args.len() {
            return None;
        }
        let res = self.args.get(self.index);
        self.index += 1;
        Some(res)
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalNativeFunctionTemplate<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeNativeFunctionTemplate(self.0.inner_val) }
    }
}
