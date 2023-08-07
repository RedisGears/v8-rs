/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! The [Inspector] is a helpful facility for remote debugging of the
//! JavaScript code provided by the V8 engine.
//!
//! To be able to debug, an inspector must be created and supplied with
//! the remote debugger's requests. To answer back, the `on_response`
//! callback must be properly set.
//!
//! # Sessions
//!
//! To be able to debug remotely, a server is required. Here one may
//! find the [server::WebSocketServer] useful. The
//! [server::WebSocketServer] is a very simple WebSocket server, which
//! can be used with an [Inspector].
//!
//! To start a debugging server session, it is required to have a
//! [V8ContextScope], a [server::WebSocketServer] and an [Inspector].
//! All of those are gently packed within an easy-to-use
//! [server::DebuggerSession] struct which abstracts all the work. It is
//! a scoped object, meaning that once an object of
//! [server::DebuggerSession] leaves its scope, the debugger server and
//! the debugging session are stopped.
//!
//! In case the `"debug-server"` feature isn't enabled, the user of the
//! crate must manually provide a way to receive and send messages over
//! the network and feed the [Inspector] with data.
use std::{ops::Deref, ptr::NonNull, rc::Rc, sync::Arc};

pub mod messages;
#[cfg(feature = "debug-server")]
pub mod server;

use crate::{
    v8::inspector::messages::MethodCallInformation,
    v8_c_raw::bindings::{v8_context_ref, v8_isolate},
};

use super::{isolate::V8Isolate, isolate_scope::V8IsolateScope, v8_context_scope::V8ContextScope};

/// The debugging inspector, carefully wrapping the
/// [`v8_inspector::Inspector`](https://chromium.googlesource.com/v8/v8/+/refs/heads/main/src/inspector)
/// API. An inspector is tied to the [V8Isolate] it was created for.
#[derive(Debug)]
pub struct RawInspector {
    raw: *mut crate::v8_c_raw::bindings::v8_inspector_c_wrapper,
}

impl RawInspector {
    /// Creates a new inspector for the provided isolate.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new(raw_isolate: *mut v8_isolate) -> Self {
        let raw = unsafe {
            crate::v8_c_raw::bindings::v8_InspectorCreate(
                raw_isolate,
                None,
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
            )
        };
        Self { raw }
    }

    /// Sets a new (possibly another) [crate::v8::isolate::V8Isolate].
    pub fn set_isolate(&self, raw_isolate: NonNull<v8_isolate>) {
        unsafe { crate::v8_c_raw::bindings::v8_InspectorSetIsolate(self.raw, raw_isolate.as_ptr()) }
    }

    /// Returns the isolate this inspector is bound to. The isolate
    /// returned won't be released automatically.
    pub fn get_isolate(&self) -> V8Isolate {
        let isolate = unsafe { crate::v8_c_raw::bindings::v8_InspectorGetIsolate(self.raw) };
        V8Isolate {
            inner_isolate: isolate,
            no_release: true,
        }
    }

    /// Returns a [V8ContextScope] of this inspector.
    pub fn get_context_scope_ptr(&self) -> *mut v8_context_ref {
        unsafe { crate::v8_c_raw::bindings::v8_InspectorGetContext(self.raw) }
    }

    // pub fn get_scope<'a>(&'a self) -> (V8Isolate, V8IsolateScope<'a>, V8ContextScope<'a, 'a>) {
    //     let isolate = self.get_isolate();
    //     let isolate_scope = V8IsolateScope::new_dummy(&isolate);
    //     let context_scope = isolate_scope
    //         .get_current_context_scope()
    //         .expect("No context scope was created");
    //     (isolate, isolate_scope, context_scope)
    // }

    /// Dispatches the Chrome Developer Tools (CDT) protocol message.
    pub fn dispatch_protocol_message<T: AsRef<str>>(&self, message: T) {
        let message = message.as_ref();
        log::trace!("Dispatching incoming message: {message}");

        let string = match std::ffi::CString::new(message) {
            Ok(string) => string,
            _ => return,
        };
        unsafe {
            crate::v8_c_raw::bindings::v8_InspectorDispatchProtocolMessage(
                self.raw,
                string.as_ptr(),
            )
        }
    }

    /// Schedules a debugger pause (sets a breakpoint) for the next
    /// statement.
    pub fn schedule_pause_on_next_statement<T: AsRef<str>>(&self, reason: T) {
        let string = match std::ffi::CString::new(reason.as_ref()) {
            Ok(string) => string,
            _ => return,
        };
        unsafe {
            crate::v8_c_raw::bindings::v8_InspectorSchedulePauseOnNextStatement(
                self.raw,
                string.as_ptr(),
            )
        }
    }

    /// Enables the debugger main loop.
    pub fn wait_frontend_message_on_pause(&self) {
        unsafe { crate::v8_c_raw::bindings::v8_InspectorWaitFrontendMessageOnPause(self.raw) }
    }
}

impl Drop for RawInspector {
    fn drop(&mut self) {
        unsafe {
            crate::v8_c_raw::bindings::v8_FreeInspector(self.raw as *mut _);
        }
    }
}

// TODO remove and rewrite so that we don't use it.
/// Currently, we rely on the thread-safety guarantees of V8, until it
/// shoots us in the leg.
unsafe impl Sync for RawInspector {}
unsafe impl Send for RawInspector {}

/// The callback which is invoked when the V8 Inspector needs to reply
/// to the client.
type OnResponseCallback = dyn FnMut(String);

/// The callback which is invoked when the V8 Inspector requires more
/// data from the front-end (the client) and, therefore, this callback
/// must attempt to read more data and dispatch it to the inspector.
///
/// The callback should return `1` when it is possible to operate (read,
/// write, send, receive messages) and `0` when not, to indicate the
/// impossibility of the further action, in which case, the inspector
/// will stop.
type OnWaitFrontendMessageOnPauseCallback =
    dyn FnMut(*mut crate::v8_c_raw::bindings::v8_inspector_c_wrapper) -> std::os::raw::c_int;

/// The debugging inspector, carefully wrapping the
/// [`v8_inspector::Inspector`](https://chromium.googlesource.com/v8/v8/+/refs/heads/main/src/inspector)
/// API. An inspector is tied to the [V8Isolate] it was created for.
pub struct Inspector {
    raw: Arc<RawInspector>,
    /// This callback is stored to preserve the lifetime, it is never
    /// called by this object, but by the C++ side.
    _on_response_callback: Box<Box<OnResponseCallback>>,
    /// This callback is stored to preserve the lifetime, it is never
    /// called by this object, but by the C++ side.
    _on_wait_frontend_message_on_pause_callback: Box<Box<OnWaitFrontendMessageOnPauseCallback>>,
}

impl std::fmt::Debug for Inspector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let on_response_str = {
            let callback = &self._on_response_callback;
            format!("Some({callback:p})")
        };

        let on_wait_str = {
            let callback = &self._on_wait_frontend_message_on_pause_callback;
            format!("Some({callback:p})")
        };

        f.debug_struct("Inspector")
            .field("raw", &self.raw)
            .field("_on_response_callback", &on_response_str)
            .field("_on_wait_frontend_message_on_pause_callback", &on_wait_str)
            .finish()
    }
}

/// The callback function used to send the messages back to the client
/// of the inspector (to the one who is debugging).
extern "C" fn on_response(
    string: *const ::std::os::raw::c_char,
    rust_callback: *mut ::std::os::raw::c_void,
) {
    let string = unsafe { std::ffi::CStr::from_ptr(string) }.to_string_lossy();
    log::trace!("Outgoing message: {string}");
    let rust_callback: &mut Box<OnResponseCallback> = unsafe {
        &mut *(rust_callback as *mut std::boxed::Box<dyn std::ops::FnMut(std::string::String)>)
    };
    rust_callback(string.to_string())
}

extern "C" fn on_wait_frontend_message_on_pause(
    raw: *mut crate::v8_c_raw::bindings::v8_inspector_c_wrapper,
    rust_callback: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    log::trace!("on_wait_frontend_message_on_pause");
    let rust_callback: &mut Box<OnWaitFrontendMessageOnPauseCallback> = unsafe {
        &mut *(rust_callback
            as *mut std::boxed::Box<
                dyn std::ops::FnMut(*mut crate::v8_c_raw::bindings::v8_inspector_c_wrapper) -> i32,
            >)
    };
    rust_callback(raw)
}

impl Inspector {
    /// Creates a new [Inspector].
    pub fn new(
        raw: Arc<RawInspector>,
        on_response_callback: Box<OnResponseCallback>,
        on_wait_frontend_message_on_pause_callback: Box<OnWaitFrontendMessageOnPauseCallback>,
    ) -> Self {
        let on_response_callback = Self::set_on_response_callback(&raw, on_response_callback);
        let on_wait_callback = Self::set_on_wait_frontend_message_on_pause_callback(
            &raw,
            on_wait_frontend_message_on_pause_callback,
        );
        Self {
            raw,
            _on_response_callback: on_response_callback,
            _on_wait_frontend_message_on_pause_callback: on_wait_callback,
        }
    }

    /// Sets the callback which is used by the debugger to send messages
    /// to the remote client.
    fn set_on_response_callback(
        raw: &RawInspector,
        on_response_callback: Box<OnResponseCallback>,
    ) -> Box<Box<OnResponseCallback>> {
        let on_response_callback = Box::new(on_response_callback);
        let on_response_callback = Box::into_raw(on_response_callback);

        unsafe {
            crate::v8_c_raw::bindings::v8_InspectorSetOnResponseCallback(
                raw.raw,
                Some(on_response),
                on_response_callback as _,
            );
        }

        unsafe { Box::from_raw(on_response_callback as *mut _) }
    }

    /// Sets the callback when the debugger needs to wait for the
    /// remote client's message.
    fn set_on_wait_frontend_message_on_pause_callback(
        raw: &RawInspector,
        on_wait_frontend_message_on_pause_callback: Box<OnWaitFrontendMessageOnPauseCallback>,
    ) -> Box<Box<OnWaitFrontendMessageOnPauseCallback>> {
        let on_wait_frontend_message_on_pause_callback =
            Box::new(on_wait_frontend_message_on_pause_callback);
        let on_wait_frontend_message_on_pause_callback =
            Box::into_raw(on_wait_frontend_message_on_pause_callback);

        unsafe {
            crate::v8_c_raw::bindings::v8_InspectorSetOnWaitFrontendMessageOnPauseCallback(
                raw.raw,
                Some(on_wait_frontend_message_on_pause),
                on_wait_frontend_message_on_pause_callback as _,
            );
        }

        unsafe { Box::from_raw(on_wait_frontend_message_on_pause_callback as *mut _) }
    }

    /// Resets the `onResponse` callback. See
    /// [Self::set_on_response_callback].
    fn reset_on_response_callback(raw: &RawInspector) {
        unsafe {
            crate::v8_c_raw::bindings::v8_InspectorSetOnResponseCallback(
                raw.raw,
                None,
                std::ptr::null_mut(),
            );
        }
    }

    /// Resets the `onWaitFrontendMessageOnPause` callback. See
    /// [Self::set_on_wait_frontend_message_on_pause_callback].
    fn reset_on_wait_frontend_message_on_pause_callback(raw: &RawInspector) {
        unsafe {
            crate::v8_c_raw::bindings::v8_InspectorSetOnWaitFrontendMessageOnPauseCallback(
                raw.raw,
                None,
                std::ptr::null_mut(),
            );
        }
    }
}

impl Deref for Inspector {
    type Target = RawInspector;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}
