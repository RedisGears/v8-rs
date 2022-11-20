/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_Compile, v8_CompileAsModule, v8_ContextRefGetGlobals, v8_ExitContextRef, v8_FreeContextRef,
    v8_GetPrivateDataFromCtxRef, v8_JsonStringify, v8_NewNativeFunction,
    v8_NewObjectFromJsonString, v8_NewResolver, v8_SetPrivateDataOnCtxRef, v8_context_ref,
};

use std::os::raw::c_void;
use std::ptr;

use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::v8_module::V8LocalModule;
use crate::v8::v8_native_function::V8LocalNativeFunction;
use crate::v8::v8_native_function_template::free_pd;
use crate::v8::v8_native_function_template::native_basic_function;
use crate::v8::v8_native_function_template::V8LocalNativeFunctionArgs;
use crate::v8::v8_object::V8LocalObject;
use crate::v8::v8_resolver::V8LocalResolver;
use crate::v8::v8_script::V8LocalScript;
use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_value::V8LocalValue;

pub struct V8ContextScope<'isolate_scope, 'isolate> {
    pub(crate) inner_ctx_ref: *mut v8_context_ref,
    pub(crate) exit_on_drop: bool,
    pub(crate) isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> V8ContextScope<'isolate_scope, 'isolate> {
    /// Compile the given code into a script object.
    #[must_use]
    pub fn compile(&self, s: &V8LocalString) -> Option<V8LocalScript<'isolate_scope, 'isolate>> {
        let inner_script = unsafe { v8_Compile(self.inner_ctx_ref, s.inner_string) };
        if inner_script.is_null() {
            None
        } else {
            Some(V8LocalScript {
                inner_script: inner_script,
                isolate_scope: self.isolate_scope,
            })
        }
    }

    #[must_use]
    pub fn get_globals(&self) -> V8LocalObject<'isolate_scope, 'isolate> {
        let inner_obj = unsafe { v8_ContextRefGetGlobals(self.inner_ctx_ref) };
        V8LocalObject {
            inner_obj: inner_obj,
            isolate_scope: self.isolate_scope,
        }
    }

    /// Compile the given code as a module.
    #[must_use]
    pub fn compile_as_module(
        &self,
        name: &V8LocalString,
        code: &V8LocalString,
        is_module: bool,
    ) -> Option<V8LocalModule<'isolate_scope, 'isolate>> {
        let inner_module = unsafe {
            v8_CompileAsModule(
                self.inner_ctx_ref,
                name.inner_string,
                code.inner_string,
                if is_module { 1 } else { 0 },
            )
        };
        if inner_module.is_null() {
            None
        } else {
            Some(V8LocalModule {
                inner_module: inner_module,
                isolate_scope: self.isolate_scope,
            })
        }
    }

    pub(crate) fn get_private_data_raw<T>(&self, index: usize) -> Option<&T> {
        let pd = unsafe { v8_GetPrivateDataFromCtxRef(self.inner_ctx_ref, index) };
        if pd.is_null() {
            None
        } else {
            Some(unsafe { &*(pd as *const T) })
        }
    }

    pub(crate) fn get_private_data_mut_raw<T>(&self, index: usize) -> Option<&mut T> {
        let pd = unsafe { v8_GetPrivateDataFromCtxRef(self.inner_ctx_ref, index) };
        if pd.is_null() {
            None
        } else {
            Some(unsafe { &mut *(pd.cast::<T>()) })
        }
    }

    /// Return the private data that was set on the context
    #[must_use]
    pub fn get_private_data<T>(&self, index: usize) -> Option<&T> {
        self.get_private_data_raw(index + 1)
    }

    /// Return the private data that was set on the context as a mut reference
    #[must_use]
    pub fn get_private_data_mut<T>(&self, index: usize) -> Option<&mut T> {
        self.get_private_data_mut_raw(index + 1)
    }

    pub(crate) fn set_private_data_raw<T>(&self, index: usize, pd: Option<&T>) {
        unsafe {
            v8_SetPrivateDataOnCtxRef(
                self.inner_ctx_ref,
                index,
                pd.map_or(ptr::null_mut(), |p| p as *const T as *mut c_void),
            );
        };
    }

    pub fn set_private_data<T>(&self, index: usize, pd: Option<&T>) {
        self.set_private_data_raw(index + 1, pd)
    }

    /// Create a new resolver object
    #[must_use]
    pub fn new_resolver(&self) -> V8LocalResolver<'isolate_scope, 'isolate> {
        let inner_resolver = unsafe { v8_NewResolver(self.inner_ctx_ref) };
        V8LocalResolver {
            inner_resolver: inner_resolver,
            isolate_scope: self.isolate_scope,
        }
    }

    #[must_use]
    pub fn new_object_from_json(
        &self,
        val: &V8LocalString,
    ) -> Option<V8LocalValue<'isolate_scope, 'isolate>> {
        let inner_val = unsafe { v8_NewObjectFromJsonString(self.inner_ctx_ref, val.inner_string) };
        if inner_val.is_null() {
            return None;
        }
        Some(V8LocalValue {
            inner_val: inner_val,
            isolate_scope: self.isolate_scope,
        })
    }

    #[must_use]
    pub fn json_stringify(
        &self,
        val: &V8LocalValue,
    ) -> Option<V8LocalString<'isolate_scope, 'isolate>> {
        let inner_string = unsafe { v8_JsonStringify(self.inner_ctx_ref, val.inner_val) };
        if inner_string.is_null() {
            return None;
        }
        Some(V8LocalString {
            inner_string: inner_string,
            isolate_scope: self.isolate_scope,
        })
    }

    #[must_use]
    pub fn new_native_function<
        T: for<'d, 'c> Fn(
            &V8LocalNativeFunctionArgs<'d, 'c>,
            &'d V8IsolateScope<'c>,
            &V8ContextScope<'d, 'c>,
        ) -> Option<V8LocalValue<'d, 'c>>,
    >(
        &self,
        func: T,
    ) -> V8LocalNativeFunction<'isolate_scope, 'isolate> {
        let inner_func = unsafe {
            v8_NewNativeFunction(
                self.inner_ctx_ref,
                Some(native_basic_function::<T>),
                Box::into_raw(Box::new(func)).cast::<c_void>(),
                Some(free_pd::<T>),
            )
        };
        V8LocalNativeFunction {
            inner_func: inner_func,
            isolate_scope: self.isolate_scope,
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for V8ContextScope<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        if self.exit_on_drop {
            unsafe { v8_ExitContextRef(self.inner_ctx_ref) }
        }
        unsafe { v8_FreeContextRef(self.inner_ctx_ref) }
    }
}
