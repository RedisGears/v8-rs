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
use std::ptr::NonNull;

use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::module::UninitialisedLocalModule;
use crate::v8::types::native_function::LocalNativeFunction;
use crate::v8::types::native_function_template::free_pd;
use crate::v8::types::native_function_template::native_basic_function;
use crate::v8::types::native_function_template::LocalNativeFunctionArgs;
use crate::v8::types::object::LocalObject;
use crate::v8::types::resolver::LocalPromiseResolver;
use crate::v8::types::script::LocalScript;
use crate::v8::types::string::LocalString;
use crate::v8::types::ScopedValue;

use super::types::any::LocalValueAny;
use super::types::Value;

/// An RAII data guard which resets the private data slot after going
/// out of scope.
pub struct ContextScopeDataGuard<'context_scope, 'data, 'isolate_scope, 'isolate, T: 'data> {
    /// Raw Index to reset after the guard goes out of scope.
    index: RawIndex,
    /// The context scope in which the guard should reset the variable.
    context_scope: &'context_scope ContextScope<'isolate_scope, 'isolate>,
    _phantom_data: PhantomData<&'data T>,
}
impl<'context_scope, 'data, 'isolate_scope, 'isolate, T: 'data>
    ContextScopeDataGuard<'context_scope, 'data, 'isolate_scope, 'isolate, T>
{
    /// Creates a new data guard with the provided index and context scope.
    pub(crate) fn new<I: Into<RawIndex>>(
        index: I,
        context_scope: &'context_scope ContextScope<'isolate_scope, 'isolate>,
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
    for ContextScopeDataGuard<'context_scope, 'data, 'isolate_scope, 'isolate, T>
{
    fn drop(&mut self) {
        self.context_scope.reset_private_data_raw(self.index);
    }
}

pub struct ContextScope<'isolate_scope, 'isolate> {
    pub(crate) inner_ctx_ref: *mut v8_context_ref,
    pub(crate) exit_on_drop: bool,
    pub(crate) isolate_scope: &'isolate_scope IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> ContextScope<'isolate_scope, 'isolate> {
    /// Compile the given code into a script object.
    pub fn compile(&self, s: &LocalString) -> Option<LocalScript<'isolate_scope, 'isolate>> {
        NonNull::new(unsafe { v8_Compile(self.inner_ctx_ref, s.0.inner_val) }).map(|ptr| {
            LocalScript(ScopedValue {
                inner_val: ptr.as_ptr(),
                isolate_scope: self.isolate_scope,
            })
        })
    }

    pub fn get_globals(&self) -> LocalObject<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ContextRefGetGlobals(self.inner_ctx_ref) };
        LocalObject(ScopedValue {
            inner_val,
            isolate_scope: self.isolate_scope,
        })
    }

    /// Compile the given code as a module.
    pub fn compile_as_module(
        &self,
        name: &LocalString,
        code: &LocalString,
        is_module: bool,
    ) -> Option<UninitialisedLocalModule<'isolate_scope, 'isolate>> {
        let inner_val = unsafe {
            v8_CompileAsModule(
                self.inner_ctx_ref,
                name.0.inner_val,
                code.0.inner_val,
                i32::from(is_module),
            )
        };
        if inner_val.is_null() {
            None
        } else {
            Some(UninitialisedLocalModule(ScopedValue {
                inner_val,
                isolate_scope: self.isolate_scope,
            }))
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
    pub fn get_private_data<T, I: Into<UserIndex>>(&self, index: I) -> Option<&T> {
        let index = index.into();
        self.get_private_data_raw(index)
    }

    /// Return the private data that was set on the context as a mut reference
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
    pub fn set_private_data<'context_scope, 'data, T, I: Into<UserIndex>>(
        &'context_scope self,
        index: I,
        data: &'data T,
    ) -> ContextScopeDataGuard<'context_scope, 'data, 'isolate_scope, 'isolate, T> {
        let index = index.into();
        self.set_private_data_raw(index, data);
        ContextScopeDataGuard::new(index, self)
    }

    pub fn reset_private_data<I: Into<UserIndex>>(&self, index: I) {
        let index = index.into();
        self.reset_private_data_raw(index)
    }

    /// Create a new resolver object
    pub fn create_resolver(&self) -> LocalPromiseResolver<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_NewResolver(self.inner_ctx_ref) };
        LocalPromiseResolver(ScopedValue {
            inner_val,
            isolate_scope: self.isolate_scope,
        })
    }

    pub fn create_object_from_json(
        &self,
        val: &LocalString,
    ) -> Option<Value<'isolate_scope, 'isolate>> {
        let inner_val = unsafe { v8_NewObjectFromJsonString(self.inner_ctx_ref, val.0.inner_val) };
        if inner_val.is_null() {
            return None;
        }
        Some(
            LocalValueAny(ScopedValue {
                inner_val,
                isolate_scope: self.isolate_scope,
            })
            .into(),
        )
    }

    pub fn json_stringify(
        &self,
        val: &LocalValueAny,
    ) -> Option<LocalString<'isolate_scope, 'isolate>> {
        let inner_val = unsafe { v8_JsonStringify(self.inner_ctx_ref, val.0.inner_val) };
        if inner_val.is_null() {
            return None;
        }
        Some(LocalString(ScopedValue {
            inner_val,
            isolate_scope: self.isolate_scope,
        }))
    }

    pub fn create_native_function<
        T: for<'d, 'c> Fn(
            &LocalNativeFunctionArgs<'d, 'c>,
            &'d IsolateScope<'c>,
            &ContextScope<'d, 'c>,
        ) -> Option<LocalValueAny<'d, 'c>>,
    >(
        &self,
        func: T,
    ) -> LocalNativeFunction<'isolate_scope, 'isolate> {
        let inner_val = unsafe {
            v8_NewNativeFunction(
                self.inner_ctx_ref,
                Some(native_basic_function::<T>),
                Box::into_raw(Box::new(func)).cast::<c_void>(),
                Some(free_pd::<T>),
            )
        };
        LocalNativeFunction(ScopedValue {
            inner_val,
            isolate_scope: self.isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> Drop for ContextScope<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        if self.exit_on_drop {
            unsafe { v8_ExitContextRef(self.inner_ctx_ref) }
        }
        unsafe { v8_FreeContextRef(self.inner_ctx_ref) }
    }
}
