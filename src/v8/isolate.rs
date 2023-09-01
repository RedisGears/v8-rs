/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! An isolate rust wrapper to v8 isolate.

use crate::v8_c_raw::bindings::{
    v8_CancelTerminateExecution, v8_FreeIsolate, v8_GetIsolateId, v8_IdleNotificationDeadline,
    v8_IsolateGetCurrent, v8_IsolateHeapSizeLimit, v8_IsolateNotifyMemoryPressure,
    v8_IsolateSetFatalErrorHandler, v8_IsolateSetNearOOMHandler, v8_IsolateSetOOMErrorHandler,
    v8_IsolateTotalHeapSize, v8_IsolateUsedHeapSize, v8_NewIsolate, v8_RequestInterrupt,
    v8_TerminateCurrExecution, v8_isolate, ISOLATE_ID_INVALID,
};

use std::os::raw::c_void;

use crate::v8::isolate_scope::V8IsolateScope;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

/// An ID type for an isolate.
/// IDs are set for each new isolate created automatically.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct IsolateId(pub(crate) u64);

impl From<u64> for IsolateId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// An isolate rust wrapper object.
/// The isolate will not be automatically freed.
/// In order to free an isolate, one must call [`V8Isolate::free_isolate`].
#[derive(Debug)]
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

impl From<*mut v8_isolate> for V8Isolate {
    fn from(value: *mut v8_isolate) -> Self {
        Self {
            inner_isolate: value,
            no_release: false,
        }
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
        unsafe {
            let res = v8_NewIsolate(initial_heap_size_in_bytes, maximum_heap_size_in_bytes);
            if crate::v8::FATAL_ERROR_CALLBACK.is_some() {
                v8_IsolateSetFatalErrorHandler(res, Some(fatal_error_callback))
            }
            if crate::v8::FATAL_ERROR_CALLBACK.is_some() {
                v8_IsolateSetOOMErrorHandler(res, Some(oom_error_callback))
            }
            res
        }
        .into()
    }

    /// Enter the isolate for code invocation.
    /// Return an `V8IsolateScope` object, when the returned
    /// object is destroy the code will exit the isolate.
    ///
    /// An isolate must be entered before running any JS code.
    pub fn enter(&self) -> V8IsolateScope {
        V8IsolateScope::new(self)
    }

    /// Sets an idle notification that the embedder is idle for longer
    /// than one second.
    /// V8 uses the notification to perform garbage collection.
    /// This call can be used repeatedly if the embedder remains idle.
    pub fn idle_notification_deadline(&self) {
        unsafe { v8_IdleNotificationDeadline(self.inner_isolate, 1.0) };
    }

    /// Requests V8 to interrupt long running JavaScript code and invoke
    /// the given [`callback`] to it. After [`callback`]
    /// returns control will be returned to the JavaScript code.
    /// There may be a number of interrupt requests in flight.
    /// Can be called from another thread without acquiring a |Locker|.
    /// Registered |callback| must not reenter interrupted Isolate.
    pub fn request_interrupt<T: Fn(&Self)>(&self, callback: T) {
        unsafe {
            v8_RequestInterrupt(
                self.inner_isolate,
                Some(interrupt_callback::<T>),
                Box::into_raw(Box::new(callback)).cast::<c_void>(),
            );
        };
    }

    /// Sets a callback to invoke in case the heap size is close to the heap limit.
    /// If multiple callbacks are added, only the most recently added callback is
    /// invoked.
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

    /// Returns the statistics about the heap memory usage.
    /// The number returned is the amount of bytes allocated and used.
    pub fn used_heap_size(&self) -> usize {
        unsafe { v8_IsolateUsedHeapSize(self.inner_isolate) }
    }

    /// Returns the statistics about the heap memory usage.
    /// The number returned is the total amount of bytes allocated,
    /// including the regions of memory which haven't been yet
    /// garbage-collected.
    pub fn total_heap_size(&self) -> usize {
        unsafe { v8_IsolateTotalHeapSize(self.inner_isolate) }
    }

    /// Returns the statistics about the heap memory usage.
    /// The number returned is current heap size limit.
    pub fn heap_size_limit(&self) -> usize {
        unsafe { v8_IsolateHeapSizeLimit(self.inner_isolate) }
    }

    /// Sets the notification that the system is running low on memory.
    /// V8 uses these notifications to guide heuristics.
    /// It is allowed to call this function from another thread while
    /// the isolate is executing long running JavaScript code.
    ///
    /// # Note
    ///
    /// The memory pressure notification this function sets is "critical".
    pub fn memory_pressure_notification(&self) {
        unsafe { v8_IsolateNotifyMemoryPressure(self.inner_isolate) }
    }

    /// Returns the entered isolate for the current thread or `None` in
    /// case there is no current isolate.
    ///
    /// # Note
    ///
    /// This method must not be invoked before the engine initialization.
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

    /// Forcefully terminates the current thread of JavaScript execution
    /// in the given isolate.
    ///
    /// # Safety
    ///
    /// This method can be used by any thread even if that thread has not
    /// acquired the V8 lock with a Locker object.
    pub fn terminate_execution(&self) {
        unsafe { v8_TerminateCurrExecution(self.inner_isolate) }
    }

    /// Resumes execution capability in this isolate, whose execution
    /// was previously forcefully terminated using [`Self::terminate_execution`].
    ///
    /// When execution is forcefully terminated using TerminateExecution(),
    /// the isolate can not resume execution until all JavaScript frames
    /// have propagated the uncatchable exception which is generated.  This
    /// method allows the program embedding the engine to handle the
    /// termination event and resume execution capability, even if
    /// JavaScript frames remain on the stack.
    ///
    /// # Safety
    ///
    /// This method can be used by any thread even if that thread has not
    /// acquired the V8 lock with a Locker object.
    pub fn cancel_terminate_execution(&self) {
        unsafe { v8_CancelTerminateExecution(self.inner_isolate) }
    }

    /// Returns a raw pointer to a [v8_isolate].
    pub fn get_raw(&self) -> *mut v8_isolate {
        self.inner_isolate
    }

    /// Returns the unique ID of this isolate.
    pub fn get_id(&self) -> Option<IsolateId> {
        let raw_id = unsafe { v8_GetIsolateId(self.inner_isolate) };
        if raw_id == ISOLATE_ID_INVALID {
            None
        } else {
            Some(raw_id.into())
        }
    }
}

impl Drop for V8Isolate {
    fn drop(&mut self) {
        if !self.no_release {
            unsafe { v8_FreeIsolate(self.inner_isolate) }
        }
    }
}
