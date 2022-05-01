// An isolate rust wrapper to v8 isolate.

use crate::v8_c_raw::bindings::{
    v8_FreeIsolate, v8_IdleNotificationDeadline, v8_IsolateRaiseException, v8_NewIsolate,
    v8_NewNativeFunctionTemplate, v8_NewObject, v8_NewObjectTemplate, v8_NewString, v8_NewTryCatch,
    v8_RequestInterrupt, v8_StringToValue, v8_isolate,
};

use std::os::raw::c_void;

use crate::v8::handler_scope::V8HandlersScope;
use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::try_catch::V8TryCatch;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_native_function_template::{
    native_basic_function, V8LocalNativeFunctionArgs, V8LocalNativeFunctionTemplate,
};
use crate::v8::v8_object::V8LocalObject;
use crate::v8::v8_object_template::V8LocalObjectTemplate;
use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_value::V8LocalValue;

/// An isolate rust wrapper object.
/// The isolate will not be automatically freed.
/// In order to free an isolate, one must call `free_isolate`.
pub struct V8Isolate {
    pub(crate) inner_isolate: *mut v8_isolate,
}

pub(crate) extern "C" fn interrupt_callback<T: Fn(&V8Isolate)>(
    inner_isolate: *mut v8_isolate,
    data: *mut ::std::os::raw::c_void,
) {
    let func = unsafe { &*(data.cast::<T>()) };
    func(&V8Isolate { inner_isolate });
}

impl Default for V8Isolate {
    fn default() -> Self {
        Self::new()
    }
}

impl V8Isolate {
    /// Create a new v8 isolate with default heap size (up to 1G).
    #[must_use]
    pub fn new() -> Self {
        Self::new_with_limits(0, 1024 * 1024 * 1024) /* default max heap: 1G */
    }

    /// Create a new isolate with the given heap limits
    /// `initial_heap_size_in_bytes` - heap initial size
    /// `maximum_heap_size_in_bytes` - heap max size
    #[must_use]
    pub fn new_with_limits(
        initial_heap_size_in_bytes: usize,
        maximum_heap_size_in_bytes: usize,
    ) -> Self {
        let inner_isolate =
            unsafe { v8_NewIsolate(initial_heap_size_in_bytes, maximum_heap_size_in_bytes) };
        Self { inner_isolate }
    }

    /// Enter the isolate for code invocation.
    /// Return an `V8IsolateScope` object, when the returned
    /// object is destroy the code will exit the isolate.
    ///
    /// An isolate must be entered before running any JS code.
    #[must_use]
    pub fn enter(&self) -> V8IsolateScope {
        V8IsolateScope::new(self)
    }

    /// Create a new handlers scope. The handler scope will
    /// collect all the local handlers which was created after
    /// the handlers scope creation and will free them when destroyed.
    #[must_use]
    pub fn new_handlers_scope(&self) -> V8HandlersScope {
        V8HandlersScope::new(self)
    }

    /// Raise an exception with the given local generic value.
    pub fn raise_exception(&self, exception: V8LocalValue) {
        unsafe { v8_IsolateRaiseException(self.inner_isolate, exception.inner_val) };
    }

    /// Same as `raise_exception` but raise exception with the given massage.
    pub fn raise_exception_str(&self, msg: &str) {
        let inner_string =
            unsafe { v8_NewString(self.inner_isolate, msg.as_ptr().cast::<i8>(), msg.len()) };
        let inner_val = unsafe { v8_StringToValue(inner_string) };
        unsafe { v8_IsolateRaiseException(self.inner_isolate, inner_val) };
    }

    /// Return a new try catch object. The object will catch any exception that was
    /// raised during the JS code invocation.
    #[must_use]
    pub fn new_try_catch(&self) -> V8TryCatch {
        let inner_trycatch = unsafe { v8_NewTryCatch(self.inner_isolate) };
        V8TryCatch { inner_trycatch }
    }

    pub fn idle_notification_deadline(&self) {
        unsafe { v8_IdleNotificationDeadline(self.inner_isolate, 1.0) };
    }

    pub fn request_interrupt<T: Fn(&Self)>(&self, callback: T) {
        unsafe {
            v8_RequestInterrupt(
                self.inner_isolate,
                Some(interrupt_callback::<T>),
                Box::into_raw(Box::new(callback)).cast::<c_void>(),
            );
        };
    }

    /// Free the isolate
    pub fn free_isolate(&self) {
        unsafe { v8_FreeIsolate(self.inner_isolate) }
    }

    /// Create a new string object.
    #[must_use]
    pub fn new_string(&self, s: &str) -> V8LocalString {
        let inner_string =
            unsafe { v8_NewString(self.inner_isolate, s.as_ptr().cast::<i8>(), s.len()) };
        V8LocalString { inner_string }
    }

    #[must_use]
    pub fn new_object(&self) -> V8LocalObject {
        let inner_obj = unsafe { v8_NewObject(self.inner_isolate) };
        V8LocalObject { inner_obj }
    }

    /// Create a new JS object template.
    #[must_use]
    pub fn new_object_template(&self) -> V8LocalObjectTemplate {
        let inner_obj = unsafe { v8_NewObjectTemplate(self.inner_isolate) };
        V8LocalObjectTemplate { inner_obj }
    }

    /// Create a new native function template.
    pub fn new_native_function_template<
        T: Fn(&V8LocalNativeFunctionArgs, &Self, &V8ContextScope) -> Option<V8LocalValue>,
    >(
        &self,
        func: T,
    ) -> V8LocalNativeFunctionTemplate {
        let inner_func = unsafe {
            v8_NewNativeFunctionTemplate(
                self.inner_isolate,
                Some(native_basic_function::<T>),
                Box::into_raw(Box::new(func)).cast::<c_void>(),
            )
        };
        V8LocalNativeFunctionTemplate { inner_func }
    }
}
