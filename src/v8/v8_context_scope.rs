/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::v8_SetPrivateDataOnCtxRef;
use crate::v8_c_raw::bindings::{
    v8_Compile, v8_CompileAsModule, v8_ContextRefGetGlobals, v8_ExitContextRef, v8_FreeContextRef,
    v8_GetPrivateDataFromCtxRef, v8_JsonStringify, v8_NewNativeFunction,
    v8_NewObjectFromJsonString, v8_NewResolver, v8_ResetPrivateDataOnCtxRef, v8_context_ref,
};
use crate::{RawIndex, UserIndex};

use std::marker::PhantomData;
use std::os::raw::c_void;

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

/// An RAII data guard which resets the private data slot after going
/// out of scope.
pub struct V8ContextScopeDataGuard<'context_scope, 'data, 'isolate_scope, 'isolate, T: 'data> {
    /// Raw Index to reset after the guard goes out of scope.
    index: RawIndex,
    /// The context scope in which the guard should reset the variable.
    context_scope: &'context_scope V8ContextScope<'isolate_scope, 'isolate>,
    _phantom_data: PhantomData<&'data T>,
}
impl<'context_scope, 'data, 'isolate_scope, 'isolate, T: 'data>
    V8ContextScopeDataGuard<'context_scope, 'data, 'isolate_scope, 'isolate, T>
{
    /// Creates a new data guard with the provided index and context scope.
    pub(crate) fn new<I: Into<RawIndex>>(
        index: I,
        context_scope: &'context_scope V8ContextScope<'isolate_scope, 'isolate>,
    ) -> Self {
        let index = index.into();
        Self {
            index,
            context_scope,
            _phantom_data: PhantomData,
        }
    }
}

impl<'context_scope, 'data, 'isolate_scope, 'isolate, T: 'data> Drop
    for V8ContextScopeDataGuard<'context_scope, 'data, 'isolate_scope, 'isolate, T>
{
    fn drop(&mut self) {
        self.context_scope.reset_private_data_raw(self.index);
    }
}

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
                inner_script,
                isolate_scope: self.isolate_scope,
            })
        }
    }

    #[must_use]
    pub fn get_globals(&self) -> V8LocalObject<'isolate_scope, 'isolate> {
        let inner_obj = unsafe { v8_ContextRefGetGlobals(self.inner_ctx_ref) };
        V8LocalObject {
            inner_obj,
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
                i32::from(is_module),
            )
        };
        if inner_module.is_null() {
            None
        } else {
            Some(V8LocalModule {
                inner_module,
                isolate_scope: self.isolate_scope,
            })
        }
    }

    pub(crate) fn get_private_data_raw<T, I: Into<RawIndex>>(&self, index: I) -> Option<&T> {
        let index = index.into();
        let pd = unsafe { v8_GetPrivateDataFromCtxRef(self.inner_ctx_ref, index.0) };
        if pd.is_null() {
            None
        } else {
            Some(unsafe { &*(pd as *const T) })
        }
    }

    pub(crate) fn get_private_data_mut_raw<T, I: Into<RawIndex>>(
        &self,
        index: I,
    ) -> Option<&mut T> {
        let index = index.into();
        let pd = unsafe { v8_GetPrivateDataFromCtxRef(self.inner_ctx_ref, index.0) };
        if pd.is_null() {
            None
        } else {
            Some(unsafe { &mut *(pd.cast::<T>()) })
        }
    }

    /// Return the private data that was set on the context
    #[must_use]
    pub fn get_private_data<T, I: Into<UserIndex>>(&self, index: I) -> Option<&T> {
        let index = index.into();
        self.get_private_data_raw(index)
    }

    /// Return the private data that was set on the context as a mut reference
    #[must_use]
    pub fn get_private_data_mut<T, I: Into<UserIndex>>(&self, index: I) -> Option<&mut T> {
        let index = index.into();
        self.get_private_data_mut_raw(index)
    }

    pub(crate) fn set_private_data_raw<T, I: Into<RawIndex>>(&self, index: I, pd: &T) {
        let index = index.into();
        unsafe {
            v8_SetPrivateDataOnCtxRef(self.inner_ctx_ref, index.0, pd as *const T as *mut c_void)
        }
    }

    pub(crate) fn reset_private_data_raw<I: Into<RawIndex>>(&self, index: I) {
        let index = index.into().0;
        unsafe { v8_ResetPrivateDataOnCtxRef(self.inner_ctx_ref, index) }
    }

    /// Sets the private data at the specified index (V8 data slot).
    /// Returns an RAII guard that takes care of resetting the data
    /// at the specified index.
    #[must_use]
    pub fn set_private_data<'context_scope, 'data, T, I: Into<UserIndex>>(
        &'context_scope self,
        index: I,
        data: &'data T,
    ) -> V8ContextScopeDataGuard<'context_scope, 'data, 'isolate_scope, 'isolate, T> {
        let index = index.into();
        self.set_private_data_raw(index, data);
        V8ContextScopeDataGuard::new(index, self)
    }

    pub fn reset_private_data<I: Into<UserIndex>>(&self, index: I) {
        let index = index.into();
        self.reset_private_data_raw(index)
    }

    /// Create a new resolver object
    #[must_use]
    pub fn new_resolver(&self) -> V8LocalResolver<'isolate_scope, 'isolate> {
        let inner_resolver = unsafe { v8_NewResolver(self.inner_ctx_ref) };
        V8LocalResolver {
            inner_resolver,
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
            inner_val,
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
            inner_string,
            isolate_scope: self.isolate_scope,
        })
    }

    #[must_use]
    pub fn new_native_function<
        T: 'static
            + for<'d, 'c> Fn(
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
            inner_func,
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
