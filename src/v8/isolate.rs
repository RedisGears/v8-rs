// An isolate rust wrapper to v8 isolate.

use crate::v8_c_raw::bindings::{
    v8_CancelTerminateExecution, v8_FreeIsolate, v8_IdleNotificationDeadline,
    v8_IsolateRaiseException, v8_IsolateSetFatalErrorHandler, v8_IsolateSetNearOOMHandler,
    v8_IsolateSetOOMErrorHandler, v8_NewArray, v8_NewArrayBuffer, v8_NewBool, v8_NewIsolate,
    v8_NewNativeFunctionTemplate, v8_NewNull, v8_NewObject, v8_NewObjectTemplate, v8_NewSet,
    v8_NewString, v8_NewTryCatch, v8_NewUnlocker, v8_RequestInterrupt, v8_StringToValue,
    v8_TerminateCurrExecution, v8_ValueFromDouble, v8_ValueFromLong, v8_isolate, v8_local_value,
};

use std::os::raw::c_void;

use crate::v8::handler_scope::V8HandlersScope;
use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::try_catch::V8TryCatch;
use crate::v8::v8_array::V8LocalArray;
use crate::v8::v8_array_buffer::V8LocalArrayBuffer;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_native_function_template::{
    free_pd, native_basic_function, V8LocalNativeFunctionArgs, V8LocalNativeFunctionTemplate,
};
use crate::v8::v8_object::V8LocalObject;
use crate::v8::v8_object_template::V8LocalObjectTemplate;
use crate::v8::v8_set::V8LocalSet;
use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_unlocker::V8Unlocker;
use crate::v8::v8_value::V8LocalValue;

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

/// An isolate rust wrapper object.
/// The isolate will not be automatically freed.
/// In order to free an isolate, one must call `free_isolate`.
pub struct V8Isolate {
    pub(crate) inner_isolate: *mut v8_isolate,
    pub(crate) no_release: bool,
}

unsafe impl Sync for V8Isolate {}
unsafe impl Send for V8Isolate {}

pub(crate) extern "C" fn interrupt_callback<T: Fn(&V8Isolate)>(
    inner_isolate: *mut v8_isolate,
    data: *mut ::std::os::raw::c_void,
) {
    let func = unsafe { &*(data.cast::<T>()) };
    func(&V8Isolate {
        inner_isolate: inner_isolate,
        no_release: true,
    });
}

impl Default for V8Isolate {
    fn default() -> Self {
        Self::new()
    }
}

extern "C" fn fatal_error_callback(location: *const c_char, message: *const c_char) {
    if let Some(callback) = unsafe { crate::v8::FATAL_ERROR_CALLBACK.as_ref() } {
        let location = unsafe { CStr::from_ptr(location) }.to_str().unwrap();
        let message = unsafe { CStr::from_ptr(message) }.to_str().unwrap();
        callback(location, message);
    }
}

extern "C" fn oom_error_callback(location: *const c_char, is_heap_oom: c_int) {
    if let Some(callback) = unsafe { crate::v8::OOM_ERROR_CALLBACK.as_ref() } {
        let location = unsafe { CStr::from_ptr(location) }.to_str().unwrap();
        let is_heap_oom = is_heap_oom != 0;
        callback(location, is_heap_oom);
    }
}

extern "C" fn near_oom_callback<F: Fn(usize, usize) -> usize>(
    data: *mut c_void,
    current_heap_limit: usize,
    initial_heap_limit: usize,
) -> usize {
    let callback = unsafe { &*(data as *mut F) };
    callback(current_heap_limit, initial_heap_limit)
}

extern "C" fn near_oom_callback_free_pd<F: Fn(usize, usize) -> usize>(data: *mut c_void) {
    unsafe {
        Box::from_raw(data.cast::<F>());
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
        let inner_isolate = unsafe {
            let res = v8_NewIsolate(initial_heap_size_in_bytes, maximum_heap_size_in_bytes);
            if crate::v8::FATAL_ERROR_CALLBACK.is_some() {
                v8_IsolateSetFatalErrorHandler(res, Some(fatal_error_callback))
            }
            if crate::v8::FATAL_ERROR_CALLBACK.is_some() {
                v8_IsolateSetOOMErrorHandler(res, Some(oom_error_callback))
            }
            res
        };

        Self {
            inner_isolate: inner_isolate,
            no_release: false,
        }
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

    /// Create a new string object.
    #[must_use]
    pub fn new_string(&self, s: &str) -> V8LocalString {
        let inner_string =
            unsafe { v8_NewString(self.inner_isolate, s.as_ptr().cast::<i8>(), s.len()) };
        V8LocalString { inner_string }
    }

    /// Create a new string object.
    #[must_use]
    pub fn new_array(&self, values: &[&V8LocalValue]) -> V8LocalArray {
        let args = values
            .iter()
            .map(|v| v.inner_val)
            .collect::<Vec<*mut v8_local_value>>();
        let ptr = args.as_ptr();
        let inner_array = unsafe { v8_NewArray(self.inner_isolate, ptr, values.len()) };
        V8LocalArray { inner_array }
    }

    #[must_use]
    pub fn new_array_buffer(&self, buff: &[u8]) -> V8LocalArrayBuffer {
        let inner_array_buffer = unsafe {
            v8_NewArrayBuffer(
                self.inner_isolate,
                buff.as_ptr() as *const c_char,
                buff.len(),
            )
        };
        V8LocalArrayBuffer { inner_array_buffer }
    }

    #[must_use]
    pub fn new_object(&self) -> V8LocalObject {
        let inner_obj = unsafe { v8_NewObject(self.inner_isolate) };
        V8LocalObject { inner_obj }
    }

    #[must_use]
    pub fn new_set(&self) -> V8LocalSet {
        let inner_set = unsafe { v8_NewSet(self.inner_isolate) };
        V8LocalSet { inner_set }
    }

    #[must_use]
    pub fn new_bool(&self, val: bool) -> V8LocalValue {
        let inner_val = unsafe { v8_NewBool(self.inner_isolate, val as i32) };
        V8LocalValue { inner_val }
    }

    pub fn new_long(&self, val: i64) -> V8LocalValue {
        let inner_val = unsafe { v8_ValueFromLong(self.inner_isolate, val) };
        V8LocalValue { inner_val }
    }

    pub fn new_double(&self, val: f64) -> V8LocalValue {
        let inner_val = unsafe { v8_ValueFromDouble(self.inner_isolate, val) };
        V8LocalValue { inner_val }
    }

    pub fn new_null(&self) -> V8LocalValue {
        let inner_val = unsafe { v8_NewNull(self.inner_isolate) };
        V8LocalValue { inner_val }
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
                Some(free_pd::<T>),
            )
        };
        V8LocalNativeFunctionTemplate { inner_func }
    }

    /// Create a new unlocker object that releases the isolate global lock.
    /// The lock will be re-aquire when the unlocker will be released.
    #[must_use]
    pub fn new_unlocker(&self) -> V8Unlocker {
        let inner_unlocker = unsafe { v8_NewUnlocker(self.inner_isolate) };
        V8Unlocker { inner_unlocker }
    }

    pub fn set_near_oom_callback<F: Fn(usize, usize) -> usize>(&self, callback: F) {
        unsafe {
            v8_IsolateSetNearOOMHandler(
                self.inner_isolate,
                Some(near_oom_callback::<F>),
                Box::into_raw(Box::new(callback)) as *mut c_void,
                Some(near_oom_callback_free_pd::<F>),
            )
        }
    }

    pub fn terminate_execution(&self) {
        unsafe { v8_TerminateCurrExecution(self.inner_isolate) }
    }

    pub fn cancel_terminate_execution(&self) {
        unsafe { v8_CancelTerminateExecution(self.inner_isolate) }
    }
}

impl Drop for V8Isolate {
    fn drop(&mut self) {
        if !self.no_release {
            unsafe { v8_FreeIsolate(self.inner_isolate) }
        }
    }
}
