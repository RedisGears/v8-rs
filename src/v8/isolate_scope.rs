/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_FreeHandlersScope, v8_IsolateEnter, v8_IsolateExit, v8_IsolateRaiseException, v8_NewArray,
    v8_NewArrayBuffer, v8_NewBool, v8_NewExternalData, v8_NewHandlersScope,
    v8_NewNativeFunctionTemplate, v8_NewNull, v8_NewObject, v8_NewObjectTemplate, v8_NewSet,
    v8_NewString, v8_NewTryCatch, v8_NewUnlocker, v8_StringToValue, v8_ValueFromDouble,
    v8_ValueFromLong, v8_handlers_scope, v8_isolate_scope, v8_local_value,
};

use crate::v8::isolate::V8Isolate;
use crate::v8::try_catch::V8TryCatch;
use crate::v8::v8_array::V8LocalArray;
use crate::v8::v8_array_buffer::V8LocalArrayBuffer;
use crate::v8::v8_context::V8Context;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_external_data::V8LocalExternalData;
use crate::v8::v8_native_function_template::{
    free_pd, native_basic_function, V8LocalNativeFunctionArgs, V8LocalNativeFunctionTemplate,
};
use crate::v8::v8_object::V8LocalObject;
use crate::v8::v8_object_template::V8LocalObjectTemplate;
use crate::v8::v8_set::V8LocalSet;
use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_unlocker::V8Unlocker;
use crate::v8::v8_value::V8LocalValue;

use std::os::raw::{c_char, c_void};

pub struct V8IsolateScope<'isolate> {
    pub(crate) isolate: &'isolate V8Isolate,
    inner_handlers_scope: *mut v8_handlers_scope,
    inner_isolate_scope: *mut v8_isolate_scope,
}

extern "C" fn free_external_data<T>(arg1: *mut ::std::os::raw::c_void) {
    unsafe { Box::from_raw(arg1 as *mut T) };
}

impl<'isolate> V8IsolateScope<'isolate> {
    /// Create an isolate scope by performing the following:
    /// 1. Enter the isolate
    /// 2. Create a scope handler.
    pub(crate) fn new(isolate: &'isolate V8Isolate) -> V8IsolateScope<'isolate> {
        let inner_isolate_scope = unsafe { v8_IsolateEnter(isolate.inner_isolate) };
        let inner_handlers_scope = unsafe { v8_NewHandlersScope(isolate.inner_isolate) };
        V8IsolateScope {
            isolate,
            inner_handlers_scope,
            inner_isolate_scope,
        }
    }

    /// Create a dummy isolate scope. This should be used only in case we know that
    /// the isolate is already entered and we already have a scope handler. For example,
    /// when calling a native function we can create a dummy isolate scope because we
    /// know we already entered the isolate and created a scope handler.
    pub(crate) fn new_dummy(isolate: &'isolate V8Isolate) -> V8IsolateScope<'isolate> {
        V8IsolateScope {
            isolate,
            inner_handlers_scope: std::ptr::null_mut(),
            inner_isolate_scope: std::ptr::null_mut(),
        }
    }

    /// Creating a new context for JS code invocation.
    #[must_use]
    pub fn new_context(&self, globals: Option<&V8LocalObjectTemplate>) -> V8Context {
        V8Context::new(self.isolate, globals)
    }

    /// Raise an exception with the given local generic value.
    pub fn raise_exception(&self, exception: V8LocalValue) {
        unsafe { v8_IsolateRaiseException(self.isolate.inner_isolate, exception.inner_val) };
    }

    /// Same as `raise_exception` but raise exception with the given massage.
    pub fn raise_exception_str(&self, msg: &str) {
        let inner_string = unsafe {
            v8_NewString(
                self.isolate.inner_isolate,
                msg.as_ptr().cast::<c_char>(),
                msg.len(),
            )
        };
        let inner_val = unsafe { v8_StringToValue(inner_string) };
        unsafe { v8_IsolateRaiseException(self.isolate.inner_isolate, inner_val) };
    }

    /// Return a new try catch object. The object will catch any exception that was
    /// raised during the JS code invocation.
    #[must_use]
    pub fn new_try_catch<'isolate_scope>(
        &'isolate_scope self,
    ) -> V8TryCatch<'isolate_scope, 'isolate> {
        let inner_trycatch = unsafe { v8_NewTryCatch(self.isolate.inner_isolate) };
        V8TryCatch {
            inner_trycatch,
            isolate_scope: self,
        }
    }

    /// Create a new string object.
    #[must_use]
    pub fn new_string<'isolate_scope>(
        &'isolate_scope self,
        s: &str,
    ) -> V8LocalString<'isolate_scope, 'isolate> {
        let inner_string = unsafe {
            v8_NewString(
                self.isolate.inner_isolate,
                s.as_ptr().cast::<c_char>(),
                s.len(),
            )
        };
        V8LocalString {
            inner_string,
            isolate_scope: self,
        }
    }

    /// Create a new string object.
    #[must_use]
    pub fn new_array<'isolate_scope>(
        &'isolate_scope self,
        values: &[&V8LocalValue],
    ) -> V8LocalArray<'isolate_scope, 'isolate> {
        let args = values
            .iter()
            .map(|v| v.inner_val)
            .collect::<Vec<*mut v8_local_value>>();
        let ptr = args.as_ptr();
        let inner_array = unsafe { v8_NewArray(self.isolate.inner_isolate, ptr, values.len()) };
        V8LocalArray {
            inner_array,
            isolate_scope: self,
        }
    }

    #[must_use]
    pub fn new_array_buffer<'isolate_scope>(
        &'isolate_scope self,
        buff: &[u8],
    ) -> V8LocalArrayBuffer<'isolate_scope, 'isolate> {
        let inner_array_buffer = unsafe {
            v8_NewArrayBuffer(
                self.isolate.inner_isolate,
                buff.as_ptr() as *const c_char,
                buff.len(),
            )
        };
        V8LocalArrayBuffer {
            inner_array_buffer,
            isolate_scope: self,
        }
    }

    #[must_use]
    pub fn new_object<'isolate_scope>(
        &'isolate_scope self,
    ) -> V8LocalObject<'isolate_scope, 'isolate> {
        let inner_obj = unsafe { v8_NewObject(self.isolate.inner_isolate) };
        V8LocalObject {
            inner_obj,
            isolate_scope: self,
        }
    }

    #[must_use]
    pub fn new_external_data<'isolate_scope, T>(
        &'isolate_scope self,
        data: T,
    ) -> V8LocalExternalData<'isolate_scope, 'isolate> {
        let data = Box::into_raw(Box::new(data));
        let inner_ext = unsafe {
            v8_NewExternalData(
                self.isolate.inner_isolate,
                data as *mut c_void,
                Some(free_external_data::<T>),
            )
        };
        V8LocalExternalData {
            inner_ext,
            isolate_scope: self,
        }
    }

    #[must_use]
    pub fn new_set<'isolate_scope>(&'isolate_scope self) -> V8LocalSet<'isolate_scope, 'isolate> {
        let inner_set = unsafe { v8_NewSet(self.isolate.inner_isolate) };
        V8LocalSet {
            inner_set,
            isolate_scope: self,
        }
    }

    #[must_use]
    pub fn new_bool<'isolate_scope>(
        &'isolate_scope self,
        val: bool,
    ) -> V8LocalValue<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_NewBool(self.isolate.inner_isolate, val as i32) };
        V8LocalValue {
            inner_val,
            isolate_scope: self,
        }
    }

    pub fn new_long<'isolate_scope>(
        &'isolate_scope self,
        val: i64,
    ) -> V8LocalValue<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ValueFromLong(self.isolate.inner_isolate, val) };
        V8LocalValue {
            inner_val,
            isolate_scope: self,
        }
    }

    pub fn new_double<'isolate_scope>(
        &'isolate_scope self,
        val: f64,
    ) -> V8LocalValue<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ValueFromDouble(self.isolate.inner_isolate, val) };
        V8LocalValue {
            inner_val,
            isolate_scope: self,
        }
    }

    pub fn new_null<'isolate_scope>(
        &'isolate_scope self,
    ) -> V8LocalValue<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_NewNull(self.isolate.inner_isolate) };
        V8LocalValue {
            inner_val,
            isolate_scope: self,
        }
    }

    /// Create a new JS object template.
    #[must_use]
    pub fn new_object_template<'isolate_scope>(
        &'isolate_scope self,
    ) -> V8LocalObjectTemplate<'isolate_scope, 'isolate> {
        let inner_obj = unsafe { v8_NewObjectTemplate(self.isolate.inner_isolate) };
        V8LocalObjectTemplate {
            inner_obj,
            isolate_scope: self,
        }
    }

    /// Create a new native function template.
    pub fn new_native_function_template<
        'isolate_scope,
        T: for<'d, 'e> Fn(
            &V8LocalNativeFunctionArgs<'d, 'e>,
            &'d V8IsolateScope<'e>,
            &V8ContextScope<'d, 'e>,
        ) -> Option<V8LocalValue<'d, 'e>>,
    >(
        &'isolate_scope self,
        func: T,
    ) -> V8LocalNativeFunctionTemplate<'isolate_scope, 'isolate> {
        let inner_func = unsafe {
            v8_NewNativeFunctionTemplate(
                self.isolate.inner_isolate,
                Some(native_basic_function::<T>),
                Box::into_raw(Box::new(func)).cast::<c_void>(),
                Some(free_pd::<T>),
            )
        };
        V8LocalNativeFunctionTemplate {
            inner_func,
            isolate_scope: self,
        }
    }

    /// Create a new unlocker object that releases the isolate global lock.
    /// The lock will be re-aquire when the unlocker will be released.
    #[must_use]
    pub fn new_unlocker<'isolate_scope>(
        &'isolate_scope self,
    ) -> V8Unlocker<'isolate_scope, 'isolate> {
        let inner_unlocker = unsafe { v8_NewUnlocker(self.isolate.inner_isolate) };
        V8Unlocker {
            inner_unlocker,
            _isolate_scope: self,
        }
    }
}

impl<'isolate> Drop for V8IsolateScope<'isolate> {
    fn drop(&mut self) {
        unsafe {
            if !self.inner_handlers_scope.is_null() {
                v8_FreeHandlersScope(self.inner_handlers_scope);
            }
            if !self.inner_isolate_scope.is_null() {
                v8_IsolateExit(self.inner_isolate_scope);
            }
        }
    }
}
