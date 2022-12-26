/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_ArgsGet, v8_ArgsGetSelf, v8_FreeNativeFunctionTemplate, v8_GetCurrentCtxRef,
    v8_GetCurrentIsolate, v8_NativeFunctionTemplateToFunction, v8_local_native_function_template,
    v8_local_value, v8_local_value_arr,
};

use std::os::raw::c_void;
use std::ptr;

use crate::v8::isolate::V8Isolate;
use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_native_function::V8LocalNativeFunction;
use crate::v8::v8_object::V8LocalObject;
use crate::v8::v8_value::V8LocalValue;

/// Native function template object
pub struct V8LocalNativeFunctionTemplate<'isolate_scope, 'isolate> {
    pub(crate) inner_func: *mut v8_local_native_function_template,
    pub(crate) isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
}

/// Native function args
pub struct V8LocalNativeFunctionArgs<'isolate_scope, 'isolate> {
    pub(crate) inner_arr: *mut v8_local_value_arr,
    len: usize,
    isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
}

pub(crate) extern "C" fn free_pd<
    T: for<'d, 'c> Fn(
        &V8LocalNativeFunctionArgs<'d, 'c>,
        &'d V8IsolateScope<'c>,
        &V8ContextScope<'d, 'c>,
    ) -> Option<V8LocalValue<'d, 'c>>,
>(
    pd: *mut c_void,
) {
    unsafe {
        let _ = Box::from_raw(pd.cast::<T>());
    }
}

pub(crate) extern "C" fn native_basic_function<
    T: for<'d, 'c> Fn(
        &V8LocalNativeFunctionArgs<'d, 'c>,
        &'d V8IsolateScope<'c>,
        &V8ContextScope<'d, 'c>,
    ) -> Option<V8LocalValue<'d, 'c>>,
>(
    args: *mut v8_local_value_arr,
    len: usize,
    pd: *mut c_void,
) -> *mut v8_local_value {
    let func = unsafe { &*(pd.cast::<T>()) };

    let inner_isolate = unsafe { v8_GetCurrentIsolate(args) };
    let isolate = V8Isolate {
        inner_isolate,
        no_release: true,
    };

    let isolate_scope = V8IsolateScope::new(&isolate);

    let inner_ctx_ref = unsafe { v8_GetCurrentCtxRef(inner_isolate) };
    let ctx_scope = V8ContextScope {
        inner_ctx_ref,
        exit_on_drop: false,
        isolate_scope: &isolate_scope,
    };

    let args = V8LocalNativeFunctionArgs {
        inner_arr: args,
        len,
        isolate_scope: &isolate_scope,
    };

    let res = func(&args, &isolate_scope, &ctx_scope);

    match res {
        Some(mut r) => {
            let inner_val = r.inner_val;
            r.inner_val = ptr::null_mut();
            inner_val
        }
        None => ptr::null_mut(),
    }
}

impl<'isolate_scope, 'isolate> V8LocalNativeFunctionTemplate<'isolate_scope, 'isolate> {
    pub fn to_function(
        &self,
        ctx_scope: &V8ContextScope,
    ) -> V8LocalNativeFunction<'isolate_scope, 'isolate> {
        let inner_func = unsafe {
            v8_NativeFunctionTemplateToFunction(ctx_scope.inner_ctx_ref, self.inner_func)
        };
        V8LocalNativeFunction {
            inner_func,
            isolate_scope: self.isolate_scope,
        }
    }
}

impl<'isolate_scope, 'isolate> V8LocalNativeFunctionArgs<'isolate_scope, 'isolate> {
    /// Return the i-th argument from the native function args
    /// # Panics
    #[must_use]
    pub fn get(&self, i: usize) -> V8LocalValue<'isolate_scope, 'isolate> {
        assert!(i <= self.len);
        let val = unsafe { v8_ArgsGet(self.inner_arr, i) };
        V8LocalValue {
            inner_val: val,
            isolate_scope: self.isolate_scope,
        }
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
    pub fn get_self(&self) -> V8LocalObject<'isolate_scope, 'isolate> {
        let val = unsafe { v8_ArgsGetSelf(self.inner_arr) };
        V8LocalObject {
            inner_obj: val,
            isolate_scope: self.isolate_scope,
        }
    }

    pub const fn persist(&self) {}

    pub fn iter<'a>(&'a self) -> V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a> {
        V8LocalNativeFunctionArgsIter {
            args: self,
            index: 0,
        }
    }
}

pub struct V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a> {
    args: &'a V8LocalNativeFunctionArgs<'isolate_scope, 'isolate>,
    index: usize,
}

impl<'isolate_scope, 'isolate, 'a> Iterator
    for V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a>
{
    type Item = V8LocalValue<'isolate_scope, 'isolate>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.args.len() {
            return None;
        }
        let res = self.args.get(self.index);
        self.index += 1;
        Some(res)
    }
}

impl<'isolate_scope, 'isolate> Drop for V8LocalNativeFunctionTemplate<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeNativeFunctionTemplate(self.inner_func) }
    }
}
