/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

// An isolate rust wrapper to v8 isolate.

use crate::v8_c_raw::bindings::{
    v8_CancelTerminateExecution, v8_FreeIsolate, v8_IdleNotificationDeadline, v8_IsolateGetCurrent,
    v8_IsolateNotifyMemoryPressure, v8_IsolateSetFatalErrorHandler, v8_IsolateSetNearOOMHandler,
    v8_IsolateSetOOMErrorHandler, v8_IsolateTotalHeapSize, v8_IsolateUsedHeapSize, v8_NewIsolate,
    v8_RequestInterrupt, v8_TerminateCurrExecution, v8_isolate,
};

use std::os::raw::c_void;

use crate::v8::isolate_scope::V8IsolateScope;
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
        inner_isolate,
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
        let _val = Box::from_raw(data.cast::<F>());
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
            inner_isolate,
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

    pub fn used_heap_size(&self) -> usize {
        unsafe { v8_IsolateUsedHeapSize(self.inner_isolate) }
    }

    pub fn total_heap_size(&self) -> usize {
        unsafe { v8_IsolateTotalHeapSize(self.inner_isolate) }
    }

    pub fn memory_pressure_notification(&self) {
        unsafe { v8_IsolateNotifyMemoryPressure(self.inner_isolate) }
    }

    pub fn current_isolate() -> Option<Self> {
        let inner_isolate = unsafe { v8_IsolateGetCurrent() };

        if inner_isolate.is_null() {
            None
        } else {
            Some(Self {
                inner_isolate,
                no_release: true,
            })
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
