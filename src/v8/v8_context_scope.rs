use crate::v8_c_raw::bindings::{
    v8_Compile, v8_CompileAsModule, v8_ExitContextRef, v8_FreeContextRef,
    v8_GetPrivateDataFromCtxRef, v8_NewNativeFunction, v8_NewResolver, v8_SetPrivateDataOnCtxRef,
    v8_context_ref,
};

use std::os::raw::c_void;
use std::ptr;

use crate::v8::isolate::V8Isolate;
use crate::v8::v8_module::V8LocalModule;
use crate::v8::v8_native_function::V8LocalNativeFunction;
use crate::v8::v8_native_function_template::native_basic_function;
use crate::v8::v8_native_function_template::V8LocalNativeFunctionArgs;
use crate::v8::v8_resolver::V8LocalResolver;
use crate::v8::v8_script::V8LocalScript;
use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_value::V8LocalValue;

pub struct V8ContextScope {
    pub(crate) inner_ctx_ref: *mut v8_context_ref,
    pub(crate) exit_on_drop: bool,
}

impl V8ContextScope {
    /// Compile the given code into a script object.
    #[must_use]
    pub fn compile(&self, s: &V8LocalString) -> Option<V8LocalScript> {
        let inner_script = unsafe { v8_Compile(self.inner_ctx_ref, s.inner_string) };
        if inner_script.is_null() {
            None
        } else {
            Some(V8LocalScript { inner_script })
        }
    }

    /// Compile the given code as a module.
    #[must_use]
    pub fn compile_as_module(
        &self,
        name: &V8LocalString,
        code: &V8LocalString,
    ) -> Option<V8LocalModule> {
        let inner_module =
            unsafe { v8_CompileAsModule(self.inner_ctx_ref, name.inner_string, code.inner_string) };
        if inner_module.is_null() {
            None
        } else {
            Some(V8LocalModule { inner_module })
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
    pub fn new_resolver(&self) -> V8LocalResolver {
        let inner_resolver = unsafe { v8_NewResolver(self.inner_ctx_ref) };
        V8LocalResolver { inner_resolver }
    }

    #[must_use]
    pub fn new_native_function<
        T: Fn(&V8LocalNativeFunctionArgs, &V8Isolate, &V8ContextScope) -> Option<V8LocalValue>,
    >(
        &self,
        func: T,
    ) -> V8LocalNativeFunction {
        let inner_func = unsafe {
            v8_NewNativeFunction(
                self.inner_ctx_ref,
                Some(native_basic_function::<T>),
                Box::into_raw(Box::new(func)).cast::<c_void>(),
            )
        };
        V8LocalNativeFunction {
            inner_func: inner_func,
        }
    }
}

impl Drop for V8ContextScope {
    fn drop(&mut self) {
        if self.exit_on_drop {
            unsafe { v8_ExitContextRef(self.inner_ctx_ref) }
        }
        unsafe { v8_FreeContextRef(self.inner_ctx_ref) }
    }
}
