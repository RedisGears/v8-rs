use crate::v8_c_raw::bindings::{
    v8_ArgsGet, v8_FreeNativeFunctionTemplate, v8_GetCurrentCtxRef, v8_GetCurrentIsolate,
    v8_local_native_function_template, v8_local_value, v8_local_value_arr,
};

use std::os::raw::c_void;
use std::ptr;

use crate::v8::isolate::V8Isolate;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_value::V8LocalValue;

/// Native function template object
pub struct V8LocalNativeFunctionTemplate {
    pub(crate) inner_func: *mut v8_local_native_function_template,
}

/// Native function args
pub struct V8LocalNativeFunctionArgs {
    pub(crate) inner_arr: *mut v8_local_value_arr,
    len: usize,
}

pub(crate) extern "C" fn native_basic_function<
    T: Fn(&V8LocalNativeFunctionArgs, &V8Isolate, &V8ContextScope) -> Option<V8LocalValue>,
>(
    args: *mut v8_local_value_arr,
    len: usize,
    pd: *mut c_void,
) -> *mut v8_local_value {
    let func = unsafe { &*(pd.cast::<T>()) };
    let args = V8LocalNativeFunctionArgs {
        inner_arr: args,
        len,
    };

    let inner_isolate = unsafe { v8_GetCurrentIsolate(args.inner_arr) };
    let isolate = V8Isolate { inner_isolate };

    let inner_ctx_ref = unsafe { v8_GetCurrentCtxRef(inner_isolate) };
    let ctc_scope = V8ContextScope {
        inner_ctx_ref,
        exit_on_drop: false,
    };

    let res = func(&args, &isolate, &ctc_scope);

    match res {
        Some(mut r) => {
            let inner_val = r.inner_val;
            r.inner_val = ptr::null_mut();
            inner_val
        }
        None => ptr::null_mut(),
    }
}

impl V8LocalNativeFunctionArgs {
    /// Return the i-th argument from the native function args
    /// # Panics
    #[must_use]
    pub fn get(&self, i: usize) -> V8LocalValue {
        assert!(i <= self.len);
        let val = unsafe { v8_ArgsGet(self.inner_arr, i) };
        V8LocalValue { inner_val: val }
    }

    /// Return the amount of arguments passed to the native function
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Checks if the list of args is empty
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len > 0
    }
}

impl Drop for V8LocalNativeFunctionTemplate {
    fn drop(&mut self) {
        unsafe { v8_FreeNativeFunctionTemplate(self.inner_func) }
    }
}
