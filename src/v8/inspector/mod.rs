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
use std::marker::PhantomData;

pub mod messages;
#[cfg(feature = "debug-server")]
pub mod server;

use crate::v8::v8_context_scope::V8ContextScope;

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
/// API.
pub struct Inspector<'context_scope, 'isolate_scope, 'isolate> {
    raw: *mut crate::v8_c_raw::bindings::v8_inspector_c_wrapper,
    /// This callback is stored to preserve the lifetime, it is never
    /// called by this object, but by the C++ side.
    _on_response_callback: Option<Box<Box<OnResponseCallback>>>,
    /// This callback is stored to preserve the lifetime, it is never
    /// called by this object, but by the C++ side.
    _on_wait_frontend_message_on_pause_callback:
        Option<Box<Box<OnWaitFrontendMessageOnPauseCallback>>>,
    /// The lifetime holder.
    _phantom_data: PhantomData<&'context_scope V8ContextScope<'isolate_scope, 'isolate>>,
}

impl<'context_scope, 'isolate_scope, 'isolate> std::fmt::Debug
    for Inspector<'context_scope, 'isolate_scope, 'isolate>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let on_response_str = if let Some(ref callback) = self._on_response_callback {
            format!("Some({callback:p})")
        } else {
            "None".to_owned()
        };

        let on_wait_str =
            if let Some(ref callback) = self._on_wait_frontend_message_on_pause_callback {
                format!("Some({callback:p})")
            } else {
                "None".to_owned()
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

impl<'context_scope, 'isolate_scope, 'isolate> Inspector<'context_scope, 'isolate_scope, 'isolate> {
    /// Creates a new [Inspector].
    pub fn new(
        context: &'context_scope V8ContextScope<'isolate_scope, 'isolate>,
        on_response_callback: Box<OnResponseCallback>,
        on_wait_frontend_message_on_pause_callback: Box<OnWaitFrontendMessageOnPauseCallback>,
    ) -> Self {
        let on_response_callback = Box::new(on_response_callback);
        let on_response_callback = Box::into_raw(on_response_callback);
        let on_wait_frontend_message_on_pause_callback =
            Box::new(on_wait_frontend_message_on_pause_callback);
        let on_wait_frontend_message_on_pause_callback =
            Box::into_raw(on_wait_frontend_message_on_pause_callback);

        let raw = unsafe {
            crate::v8_c_raw::bindings::v8_InspectorCreate(
                context.inner_ctx_ref,
                Some(on_response),
                on_response_callback as _,
                Some(on_wait_frontend_message_on_pause),
                on_wait_frontend_message_on_pause_callback as _,
            )
        };

        let on_response_callback: Box<Box<dyn FnMut(String)>> =
            unsafe { Box::from_raw(on_response_callback as *mut _) };
        let on_wait_frontend_message_on_pause_callback =
            unsafe { Box::from_raw(on_wait_frontend_message_on_pause_callback as *mut _) };
        Self {
            raw,
            _on_response_callback: Some(on_response_callback),
            _on_wait_frontend_message_on_pause_callback: Some(
                on_wait_frontend_message_on_pause_callback,
            ),
            _phantom_data: PhantomData,
        }
    }

    /// Creates a new [Inspector] without any callbacks set. Such an
    /// inspector can't be used for debugging purposes, but the
    /// callbacks can be set later via
    /// [Inspector::set_on_wait_frontend_message_on_pause_callback] and
    /// [Inspector::set_on_response_callback]. Without the callbacks,
    /// the inspector attaches to the `V8Platform` and the `V8Context`,
    /// while the proper hooks (callbacks) can be set later when the
    /// debugging process should actually take place.
    pub fn new_without_callbacks(
        context: &'context_scope V8ContextScope<'isolate_scope, 'isolate>,
    ) -> Self {
        let raw = unsafe {
            crate::v8_c_raw::bindings::v8_InspectorCreate(
                context.inner_ctx_ref,
                None,
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
            )
        };
        Self {
            raw,
            _on_response_callback: None,
            _on_wait_frontend_message_on_pause_callback: None,
            _phantom_data: PhantomData,
        }
    }

    /// Sets the callback which is used by the debugger to send messages
    /// to the remote client.
    pub fn set_on_response_callback(&mut self, on_response_callback: Box<OnResponseCallback>) {
        let on_response_callback = Box::new(on_response_callback);
        let on_response_callback = Box::into_raw(on_response_callback);

        unsafe {
            crate::v8_c_raw::bindings::v8_InspectorSetOnResponseCallback(
                self.raw,
                Some(on_response),
                on_response_callback as _,
            );
        }

        let on_response_callback = unsafe { Box::from_raw(on_response_callback as *mut _) };
        self._on_response_callback = Some(on_response_callback);
    }

    /// Sets the callback when the debugger needs to wait for the
    /// remote client's message.
    pub fn set_on_wait_frontend_message_on_pause_callback(
        &mut self,
        on_wait_frontend_message_on_pause_callback: Box<OnWaitFrontendMessageOnPauseCallback>,
    ) {
        let on_wait_frontend_message_on_pause_callback =
            Box::new(on_wait_frontend_message_on_pause_callback);
        let on_wait_frontend_message_on_pause_callback =
            Box::into_raw(on_wait_frontend_message_on_pause_callback);

        unsafe {
            crate::v8_c_raw::bindings::v8_InspectorSetOnWaitFrontendMessageOnPauseCallback(
                self.raw,
                Some(on_wait_frontend_message_on_pause),
                on_wait_frontend_message_on_pause_callback as _,
            );
        }

        let on_wait_frontend_message_on_pause_callback =
            unsafe { Box::from_raw(on_wait_frontend_message_on_pause_callback as *mut _) };
        self._on_wait_frontend_message_on_pause_callback =
            Some(on_wait_frontend_message_on_pause_callback);
    }

    /// Resets the `onResponse` callback. See
    /// [Self::set_on_response_callback].
    pub fn reset_on_response_callback(&mut self) {
        unsafe {
            crate::v8_c_raw::bindings::v8_InspectorSetOnResponseCallback(
                self.raw,
                None,
                std::ptr::null_mut(),
            );
        }
        self._on_response_callback = None;
    }

    /// Resets the `onWaitFrontendMessageOnPause` callback. See
    /// [Self::set_on_wait_frontend_message_on_pause_callback].
    pub fn reset_on_wait_frontend_message_on_pause_callback(&mut self) {
        unsafe {
            crate::v8_c_raw::bindings::v8_InspectorSetOnWaitFrontendMessageOnPauseCallback(
                self.raw,
                None,
                std::ptr::null_mut(),
            );
        }
        self._on_wait_frontend_message_on_pause_callback = None;
    }

    /// Dispatches the Chrome Developer Tools (CDT) protocol message.
    pub fn dispatch_protocol_message<T: AsRef<str>>(&self, message: T) {
        let message = message.as_ref();

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

impl<'context_scope, 'isolate_scope, 'isolate> Drop
    for Inspector<'context_scope, 'isolate_scope, 'isolate>
{
    fn drop(&mut self) {
        unsafe {
            crate::v8_c_raw::bindings::v8_FreeInspector(self.raw as *mut _);
        }
    }
}
