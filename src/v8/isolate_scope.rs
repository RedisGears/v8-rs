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

use crate::v8::context::Context;
use crate::v8::context_scope::ContextScope;
use crate::v8::isolate::Isolate;
use crate::v8::types::array::LocalArray;
use crate::v8::types::array_buffer::LocalArrayBuffer;
use crate::v8::types::external_data::LocalExternalData;
use crate::v8::types::native_function_template::{
    free_pd, native_basic_function, LocalNativeFunctionArgs, LocalNativeFunctionTemplate,
};
use crate::v8::types::object::LocalObject;
use crate::v8::types::object_template::LocalObjectTemplate;
use crate::v8::types::set::LocalSet;
use crate::v8::types::string::LocalString;
use crate::v8::types::try_catch::TryCatch;
use crate::v8::types::unlocker::Unlocker;
use crate::v8::types::LocalValueGeneric;

use std::os::raw::{c_char, c_void};

pub struct IsolateScope<'isolate> {
    pub(crate) isolate: &'isolate Isolate,
    inner_handlers_scope: *mut v8_handlers_scope,
    inner_isolate_scope: *mut v8_isolate_scope,
}

extern "C" fn free_external_data<T>(arg1: *mut ::std::os::raw::c_void) {
    unsafe { Box::from_raw(arg1 as *mut T) };
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
    pub fn new_context(&self, globals: Option<&LocalObjectTemplate>) -> Context {
        Context::new(self.isolate, globals)
    }

    /// Raise an exception with the given local generic value.
    pub fn raise_exception(&self, exception: LocalValueGeneric) {
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
    ) -> TryCatch<'isolate_scope, 'isolate> {
        let inner_trycatch = unsafe { v8_NewTryCatch(self.isolate.inner_isolate) };
        TryCatch {
            inner_trycatch,
            isolate_scope: self,
        }
    }

    /// Create a new string object.
    #[must_use]
    pub fn new_string<'isolate_scope>(
        &'isolate_scope self,
        s: &str,
    ) -> LocalString<'isolate_scope, 'isolate> {
        let inner_string = unsafe {
            v8_NewString(
                self.isolate.inner_isolate,
                s.as_ptr().cast::<c_char>(),
                s.len(),
            )
        };
        LocalString {
            inner_string,
            isolate_scope: self,
        }
    }

    /// Create a new string object.
    #[must_use]
    pub fn new_array<'isolate_scope>(
        &'isolate_scope self,
        values: &[&LocalValueGeneric],
    ) -> LocalArray<'isolate_scope, 'isolate> {
        let args = values
            .iter()
            .map(|v| v.inner_val)
            .collect::<Vec<*mut v8_local_value>>();
        let ptr = args.as_ptr();
        let inner_array = unsafe { v8_NewArray(self.isolate.inner_isolate, ptr, values.len()) };
        LocalArray {
            inner_array,
            isolate_scope: self,
        }
    }

    #[must_use]
    pub fn new_array_buffer<'isolate_scope>(
        &'isolate_scope self,
        buff: &[u8],
    ) -> LocalArrayBuffer<'isolate_scope, 'isolate> {
        let inner_array_buffer = unsafe {
            v8_NewArrayBuffer(
                self.isolate.inner_isolate,
                buff.as_ptr() as *const c_char,
                buff.len(),
            )
        };
        LocalArrayBuffer {
            inner_array_buffer,
            isolate_scope: self,
        }
    }

    #[must_use]
    pub fn new_object<'isolate_scope>(
        &'isolate_scope self,
    ) -> LocalObject<'isolate_scope, 'isolate> {
        let inner_obj = unsafe { v8_NewObject(self.isolate.inner_isolate) };
        LocalObject {
            inner_obj,
            isolate_scope: self,
        }
    }

    #[must_use]
    pub fn new_external_data<'isolate_scope, T>(
        &'isolate_scope self,
        data: T,
    ) -> LocalExternalData<'isolate_scope, 'isolate> {
        let data = Box::into_raw(Box::new(data));
        let inner_ext = unsafe {
            v8_NewExternalData(
                self.isolate.inner_isolate,
                data as *mut c_void,
                Some(free_external_data::<T>),
            )
        };
        LocalExternalData {
            inner_ext,
            isolate_scope: self,
        }
    }

    #[must_use]
    pub fn new_set<'isolate_scope>(&'isolate_scope self) -> LocalSet<'isolate_scope, 'isolate> {
        let inner_set = unsafe { v8_NewSet(self.isolate.inner_isolate) };
        LocalSet {
            inner_set,
            isolate_scope: self,
        }
    }

    #[must_use]
    pub fn new_bool<'isolate_scope>(
        &'isolate_scope self,
        val: bool,
    ) -> LocalValueGeneric<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_NewBool(self.isolate.inner_isolate, val as i32) };
        LocalValueGeneric {
            inner_val,
            isolate_scope: self,
        }
    }

    pub fn new_long<'isolate_scope>(
        &'isolate_scope self,
        val: i64,
    ) -> LocalValueGeneric<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ValueFromLong(self.isolate.inner_isolate, val) };
        LocalValueGeneric {
            inner_val,
            isolate_scope: self,
        }
    }

    pub fn new_double<'isolate_scope>(
        &'isolate_scope self,
        val: f64,
    ) -> LocalValueGeneric<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ValueFromDouble(self.isolate.inner_isolate, val) };
        LocalValueGeneric {
            inner_val,
            isolate_scope: self,
        }
    }

    pub fn new_null<'isolate_scope>(
        &'isolate_scope self,
    ) -> LocalValueGeneric<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_NewNull(self.isolate.inner_isolate) };
        LocalValueGeneric {
            inner_val,
            isolate_scope: self,
        }
    }

    /// Create a new JS object template.
    #[must_use]
    pub fn new_object_template<'isolate_scope>(
        &'isolate_scope self,
    ) -> LocalObjectTemplate<'isolate_scope, 'isolate> {
        let inner_obj = unsafe { v8_NewObjectTemplate(self.isolate.inner_isolate) };
        LocalObjectTemplate {
            inner_obj,
            isolate_scope: self,
        }
    }

    /// Create a new native function template.
    pub fn new_native_function_template<
        'isolate_scope,
        T: for<'d, 'e> Fn(
            &LocalNativeFunctionArgs<'d, 'e>,
            &'d IsolateScope<'e>,
            &ContextScope<'d, 'e>,
        ) -> Option<LocalValueGeneric<'d, 'e>>,
    >(
        &'isolate_scope self,
        func: T,
    ) -> LocalNativeFunctionTemplate<'isolate_scope, 'isolate> {
        let inner_func = unsafe {
            v8_NewNativeFunctionTemplate(
                self.isolate.inner_isolate,
                Some(native_basic_function::<T>),
                Box::into_raw(Box::new(func)).cast::<c_void>(),
                Some(free_pd::<T>),
            )
        };
        LocalNativeFunctionTemplate {
            inner_func,
            isolate_scope: self,
        }
    }

    /// Create a new unlocker object that releases the isolate global lock.
    /// The lock will be re-aquire when the unlocker will be released.
    #[must_use]
    pub fn new_unlocker<'isolate_scope>(
        &'isolate_scope self,
    ) -> Unlocker<'isolate_scope, 'isolate> {
        let inner_unlocker = unsafe { v8_NewUnlocker(self.isolate.inner_isolate) };
        Unlocker {
            inner_unlocker,
            _isolate_scope: self,
        }
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
