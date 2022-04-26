// An isolate rust wrapper to v8 isolate.

use crate::v8_c_raw::bindings::{
    v8_isolate,
    v8_NewIsolate,
    v8_FreeIsolate,
    v8_IsolateRaiseException,
    v8_NewString,
    v8_StringToValue,
    v8_NewTryCatch,
    v8_IdleNotificationDeadline,
    v8_RequestInterrupt,
    v8_NewObjectTemplate,
    v8_NewNativeFunctionTemplate,
};

use std::os::raw::{c_char, c_void};

use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::handler_scope::V8HandlersScope;
use crate::v8::v8_value::V8LocalValue;
use crate::v8::try_catch::V8TryCatch;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_object_template::V8LocalObjectTemplate;
use crate::v8::v8_native_function_template::{
    V8LocalNativeFunctionArgs,
    V8LocalNativeFunctionTemplate,
    native_basic_function,
};

/// An isolate rust wrapper object.
/// The isolate will not be automatically freed.
/// In order to free an isolate, one must call `free_isolate`.
pub struct V8Isolate {
    pub (crate) inner_isolate: *mut v8_isolate,
}

pub (crate)extern "C" fn interrupt_callback<T:Fn(&V8Isolate)>(isolate: *mut v8_isolate, data: *mut ::std::os::raw::c_void) {
    let func = unsafe{&*(data as *mut T)};
    func(&V8Isolate{inner_isolate: isolate});
}

impl V8Isolate {
    /// Createa a new v8 isolate with default heap size (up to 1G).
    pub fn new() -> V8Isolate {
        Self::new_with_limits(0, 1024 * 1024 * 1024) /* default max heap: 1G */
    }

    /// Create a new isolate with the given heap limits
    /// initial_heap_size_in_bytes - heap initial size
    /// maximum_heap_size_in_bytes - heap max size
    pub fn new_with_limits(initial_heap_size_in_bytes: usize, maximum_heap_size_in_bytes: usize) -> V8Isolate {
        let inner_isolate = unsafe{v8_NewIsolate(initial_heap_size_in_bytes, maximum_heap_size_in_bytes)};
        V8Isolate {
            inner_isolate: inner_isolate,
        }
    }

    /// Enter the isolate for code invocation.
    /// Return an V8IsolateScope object, when the returned
    /// object is destroy the code will exit the isolate.
    /// 
    /// An isolate must be entered before running any JS code.
    pub fn enter(&self) -> V8IsolateScope {
        V8IsolateScope::new(self)
    }

    /// Create a new handlers scope. The handler scope will
    /// collect all the local handlers which was created after
    /// the handlers scope creation and will free them when destroyed.
    pub fn new_handlers_scope(&self) -> V8HandlersScope {
        V8HandlersScope::new(self)
    }

    /// Raise an exception with the given local generic value.
    pub fn raise_exception(&self, exception: V8LocalValue) {
        unsafe{v8_IsolateRaiseException(self.inner_isolate, exception.inner_val)};
    }

    /// Same as `raise_exception` but raise exception with the given massage.
    pub fn raise_exception_str(&self, msg: &str) {
        let inner_string = unsafe{v8_NewString(self.inner_isolate, msg.as_ptr() as *const c_char, msg.len())};
        let inner_val = unsafe{v8_StringToValue(inner_string)};
        unsafe{v8_IsolateRaiseException(self.inner_isolate, inner_val)};
    }

    /// Return a new try catch object. The object will catch any exception that was
    /// raised during the JS code invocation.
    pub fn new_try_catch(&self) ->V8TryCatch {
        let inner_trycatch = unsafe{v8_NewTryCatch(self.inner_isolate)};
        V8TryCatch {
            inner_trycatch: inner_trycatch,
        }
    }

    pub fn idle_notification_deadline(&self) {
        unsafe{v8_IdleNotificationDeadline(self.inner_isolate, 1.0)};
    }

    pub fn request_interrupt<T:Fn(&V8Isolate)>(&self, callback: T) {
        unsafe{v8_RequestInterrupt(self.inner_isolate, Some(interrupt_callback::<T>), Box::into_raw(Box::new(callback)) as *mut c_void)};
    }

    /// Free the isolate
    pub fn free_isolate(&self) {
        unsafe {v8_FreeIsolate(self.inner_isolate)}
    }

    /// Create a new string object.
    pub fn new_string(&self, s: &str) -> V8LocalString {
        let inner_string = unsafe{v8_NewString(self.inner_isolate, s.as_ptr() as *const c_char, s.len())};
        V8LocalString{
            inner_string: inner_string,
        }
    }

    /// Create a new JS object template.
    pub fn new_object_template(&self) -> V8LocalObjectTemplate {
        let inner_obj = unsafe{v8_NewObjectTemplate(self.inner_isolate)};
        V8LocalObjectTemplate{
            inner_obj: inner_obj,
        }
    }

    /// Create a new native function template.
    pub fn new_native_function_template<T:Fn(&V8LocalNativeFunctionArgs, &V8Isolate, &V8ContextScope) -> Option<V8LocalValue>>(&self, func: T) -> V8LocalNativeFunctionTemplate {
        let inner_func = unsafe{v8_NewNativeFunctionTemplate(self.inner_isolate, Some(native_basic_function::<T>), Box::into_raw(Box::new(func)) as *mut c_void)};
        V8LocalNativeFunctionTemplate{
            inner_func: inner_func,
        }
    }
}
