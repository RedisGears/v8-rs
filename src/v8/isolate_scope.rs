/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_FreeHandlersScope, v8_IsolateEnter, v8_IsolateExit, v8_IsolateRaiseException,
    v8_NewHandlersScope, v8_handlers_scope, v8_isolate_scope,
};

use crate::v8::context::Context;
use crate::v8::isolate::Isolate;
use crate::v8::types::object_template::LocalObjectTemplate;
use crate::v8::types::string::LocalString;

use super::context_scope::ContextScope;
use super::types::any::LocalValueAny;
use super::types::native_function_template::LocalNativeFunctionArgs;
use super::types::Value;

#[derive(Debug)]
pub struct IsolateScope<'isolate> {
    pub(crate) isolate: &'isolate Isolate,
    inner_handlers_scope: *mut v8_handlers_scope,
    inner_isolate_scope: *mut v8_isolate_scope,
}

impl<'isolate> IsolateScope<'isolate> {
    pub(crate) fn new(isolate: &'isolate Isolate) -> IsolateScope<'isolate> {
        let inner_isolate_scope = unsafe { v8_IsolateEnter(isolate.inner_isolate) };
        let inner_handlers_scope = unsafe { v8_NewHandlersScope(isolate.inner_isolate) };
        IsolateScope {
            isolate,
            inner_handlers_scope,
            inner_isolate_scope,
        }
    }

    /// Creating a new context for JS code invocation.
    #[must_use]
    pub fn create_context(&self, globals: Option<&LocalObjectTemplate>) -> Context {
        Context::new(self.isolate, globals)
    }

    /// Raise an exception with the given local generic value.
    pub fn raise_exception(&self, exception: LocalValueAny) {
        unsafe { v8_IsolateRaiseException(self.isolate.inner_isolate, exception.0.inner_val) };
    }

    /// Same as `raise_exception` but raise exception with the given massage.
    pub fn raise_exception_str(&self, msg: &str) {
        let value = LocalValueAny::from(LocalString::new(msg, self));
        unsafe { v8_IsolateRaiseException(self.isolate.inner_isolate, value.0.inner_val) };
    }

    /// Return a new try catch object. The object will catch any exception that was
    /// raised during the JS code invocation.
    #[must_use]
    pub fn create_try_catch<'isolate_scope>(
        &'isolate_scope self,
    ) -> Value<'isolate_scope, 'isolate> {
        Value::new_try_catch(self)
    }

    /// Create a new string object.
    #[must_use]
    pub fn create_string<'isolate_scope>(
        &'isolate_scope self,
        s: &str,
    ) -> Value<'isolate_scope, 'isolate> {
        Value::new_string(s, self)
    }

    /// Create a new array object.
    #[must_use]
    pub fn create_array<'isolate_scope>(
        &'isolate_scope self,
        values: &[&LocalValueAny],
    ) -> Value<'isolate_scope, 'isolate> {
        Value::new_array(values, self)
    }

    #[must_use]
    pub fn create_array_buffer<'isolate_scope>(
        &'isolate_scope self,
        bytes: &[u8],
    ) -> Value<'isolate_scope, 'isolate> {
        Value::new_array_buffer(bytes, self)
    }

    #[must_use]
    pub fn create_object<'isolate_scope>(&'isolate_scope self) -> Value<'isolate_scope, 'isolate> {
        Value::new_object(self)
    }

    #[must_use]
    pub fn create_external_data<'isolate_scope, T>(
        &'isolate_scope self,
        data: T,
    ) -> Value<'isolate_scope, 'isolate> {
        Value::new_external_data(data, self)
    }

    #[must_use]
    pub fn create_set<'isolate_scope>(&'isolate_scope self) -> Value<'isolate_scope, 'isolate> {
        Value::new_set(self)
    }

    #[must_use]
    pub fn create_bool<'isolate_scope>(
        &'isolate_scope self,
        val: bool,
    ) -> Value<'isolate_scope, 'isolate> {
        Value::from_bool(val, self)
    }

    pub fn create_long<'isolate_scope>(
        &'isolate_scope self,
        val: i64,
    ) -> Value<'isolate_scope, 'isolate> {
        Value::from_i64(val, self)
    }

    pub fn create_double<'isolate_scope>(
        &'isolate_scope self,
        val: f64,
    ) -> Value<'isolate_scope, 'isolate> {
        Value::from_f64(val, self)
    }

    pub fn create_null<'isolate_scope>(&'isolate_scope self) -> Value<'isolate_scope, 'isolate> {
        Value::new_null(self)
    }

    /// Create a new JS object template.
    #[must_use]
    pub fn create_object_template<'isolate_scope>(
        &'isolate_scope self,
    ) -> Value<'isolate_scope, 'isolate> {
        Value::new_object_template(self)
    }

    /// Create a new native function template.
    pub fn create_native_function_template<
        'isolate_scope,
        T: for<'d, 'c> Fn(
            &LocalNativeFunctionArgs<'d, 'c>,
            &'d IsolateScope<'c>,
            &ContextScope<'d, 'c>,
        ) -> Option<LocalValueAny<'d, 'c>>,
    >(
        &'isolate_scope self,
        function: T,
    ) -> Value<'isolate_scope, 'isolate> {
        Value::new_native_function_template(function, self)
    }

    /// Create a new unlocker object that releases the isolate global lock.
    /// The lock will be re-aquire when the unlocker will be released.
    #[must_use]
    pub fn create_unlocker<'isolate_scope>(
        &'isolate_scope self,
    ) -> Value<'isolate_scope, 'isolate> {
        Value::new_unlocker(self)
    }
}

impl<'isolate> Drop for IsolateScope<'isolate> {
    fn drop(&mut self) {
        unsafe {
            v8_FreeHandlersScope(self.inner_handlers_scope);
            v8_IsolateExit(self.inner_isolate_scope);
        }
    }
}
