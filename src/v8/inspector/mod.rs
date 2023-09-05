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
use std::{ops::Deref, ptr::NonNull, sync::Arc};

#[cfg(feature = "debug-server")]
pub mod messages;
#[cfg(feature = "debug-server")]
pub mod server;

use crate::v8_c_raw::bindings::v8_context_ref;

use super::{isolate::V8Isolate, v8_context_scope::V8ContextScope};

/// The debugging inspector, carefully wrapping the
/// [`v8_inspector::Inspector`](https://chromium.googlesource.com/v8/v8/+/refs/heads/main/src/inspector)
/// API. An inspector is tied to the [V8Isolate] it was created for.
///
/// # Example
///
/// ```rust
/// use v8_rs::v8::*;
/// use v8_rs::v8::inspector::RawInspector;
///
/// // Initialise the V8 engine:
/// v8_init_platform(1, Some("--expose-gc")).unwrap();
/// v8_init().unwrap();
///
/// // Create a new isolate:
/// let isolate = isolate::V8Isolate::new();
///
/// // Enter the isolate created:
/// let i_scope = isolate.enter();
///
/// // Create a JS execution context for code invocation:""
/// let ctx = i_scope.new_context(None);
///
/// // Enter the created execution context for debugging:
/// let ctx_scope = ctx.enter(&i_scope);
///
/// // Create an inspector.
/// let _inspector = RawInspector::new(&ctx_scope);
/// ```
#[derive(Debug)]
pub struct RawInspector {
    raw: NonNull<crate::v8_c_raw::bindings::v8_inspector_c_wrapper>,
}

impl RawInspector {
    /// Creates a new inspector for the provided isolate.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new(context_scope: &V8ContextScope<'_, '_>) -> Self {
        let raw_context = context_scope.get_inner();
        let raw = unsafe {
            NonNull::new_unchecked(crate::v8_c_raw::bindings::v8_InspectorCreate(
                raw_context,
                None,
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
            ))
        };
        Self { raw }
    }

    /// Returns the isolate this inspector is bound to. The isolate
    /// returned won't be released automatically.
    pub fn get_isolate(&self) -> V8Isolate {
        let isolate =
            unsafe { crate::v8_c_raw::bindings::v8_InspectorGetIsolate(self.raw.as_ptr()) };
        V8Isolate {
            inner_isolate: isolate,
            no_release: true,
        }
    }

    /// Returns a raw mutable pointer of the underlying object of
    /// [`crate::v8::v8_context_scope::V8ContextScope`] of this
    /// inspector.
    pub fn get_context_scope_ptr(&self) -> *mut v8_context_ref {
        unsafe { crate::v8_c_raw::bindings::v8_InspectorGetContext(self.raw.as_ptr()) }
    }

    /// Dispatches the Chrome Developer Tools (CDT) protocol message.
    /// The message must be a valid stringified JSON object with no NUL
    /// symbols, and the message must be allowed by the V8 Inspector
    /// Protocol.
    pub fn dispatch_protocol_message<T: AsRef<str>>(
        &self,
        message: T,
    ) -> Result<(), std::io::Error> {
        let message = message.as_ref();
        log::trace!("Dispatching incoming message: {message}",);

        let string = std::ffi::CString::new(message).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "The V8 Inspector Protocol message shouldn't contain nul symbols.",
            )
        })?;

        unsafe {
            crate::v8_c_raw::bindings::v8_InspectorDispatchProtocolMessage(
                self.raw.as_ptr(),
                string.as_ptr(),
            );
        }

        Ok(())
    }

    /// Schedules a debugger pause (sets a breakpoint) for the next
    /// statement. The `reason` argument may be any string, helpful to
    /// the user.
    pub fn schedule_pause_on_next_statement<T: AsRef<str>>(
        &self,
        reason: T,
    ) -> Result<(), std::io::Error> {
        let string = std::ffi::CString::new(reason.as_ref()).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "The V8 Inspector Protocol breakpoint reason shouldn't contain nul symbols.",
            )
        })?;

        unsafe {
            crate::v8_c_raw::bindings::v8_InspectorSchedulePauseOnNextStatement(
                self.raw.as_ptr(),
                string.as_ptr(),
            );
        }

        Ok(())
    }

    /// Enables the debugger main loop. Only useful when the debugger
    /// is on pause. It is usually called automatically by the inspector
    /// but may also be called from here to wait for a certain event
    /// on the client side.
    pub fn wait_frontend_message_on_pause(&self) {
        unsafe {
            crate::v8_c_raw::bindings::v8_InspectorWaitFrontendMessageOnPause(self.raw.as_ptr())
        }
    }
}

impl Drop for RawInspector {
    fn drop(&mut self) {
        unsafe {
            crate::v8_c_raw::bindings::v8_FreeInspector(self.raw.as_ptr());
        }
    }
}

// TODO remove and rewrite so that we don't use it.
/// Currently, we rely on the thread-safety of V8 which is said to not
/// exist.
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
///
/// This is a more user-friendly version of the inspector, which, at
/// the same time, can be used for a remote debugging session. The
/// reason why this [`Inspector`] exists is to help managing the
/// debugging sessions, as for the V8 to properly debug, it requires
/// to be able to send the responses back to the client. The callbacks
/// which this version of the `Inspector` has are mandatory for a
/// properly working session.
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
                raw.raw.as_ptr(),
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
                raw.raw.as_ptr(),
                Some(on_wait_frontend_message_on_pause),
                on_wait_frontend_message_on_pause_callback as _,
            );
        }

        unsafe { Box::from_raw(on_wait_frontend_message_on_pause_callback as *mut _) }
    }
}

impl Deref for Inspector {
    type Target = RawInspector;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v8::*;

    #[test]
    fn set_breakpoint() {
        // Initialise the V8 engine:
        crate::test_utils::initialize();

        // Create a new isolate:
        let isolate = isolate::V8Isolate::new();

        // Enter the isolate created:
        let i_scope = isolate.enter();

        // Create a JS execution context for code invocation:""
        let ctx = i_scope.new_context(None);

        // Enter the created execution context for debugging:
        let ctx_scope = ctx.enter(&i_scope);

        // Create an inspector.
        let inspector = RawInspector::new(&ctx_scope);

        // Set a "good" breakpoint.
        assert!(inspector
            .schedule_pause_on_next_statement("Test breakpoint")
            .is_ok());

        // Set a "bad" breakpoint.
        assert!(inspector
            .schedule_pause_on_next_statement("Test\0breakpoint")
            .is_err());
    }

    // TODO: Unfortunately, this test crashes the V8 engine, it requires
    // a proper research as to why.
    // // The idea of this test is to dispatch two messages, a correct and
    // // an incorrect one and check that the dispatch method works
    // // as expected. The only test case is that we can't dispatch
    // // invalid messages. A valid message is a JSON-string, without NUL
    // // symbols.
    // #[cfg(feature = "debug-server")]
    // #[test]
    // fn dispatch_message() {
    //     // Initialise the V8 engine:
    //     crate::test_utils::initialize();

    //     // Create a new isolate:
    //     let isolate = isolate::V8Isolate::new();

    //     // Enter the isolate created:
    //     let i_scope = isolate.enter();

    //     // Create the code string object:
    //     let code_str = i_scope.new_string("1+1");

    //     // Create a JS execution context for code invocation:""
    //     let ctx = i_scope.new_context(None);

    //     // Enter the created execution context for debugging:
    //     let ctx_scope = ctx.enter(&i_scope);

    //     // Create an inspector.
    //     let inspector = RawInspector::new(&ctx_scope);

    //     let script = ctx_scope.compile(&code_str).unwrap();
    //     let _res = script.run(&ctx_scope).unwrap();

    //     let mut message = messages::ClientMessage::new_runtime_enable(0);

    //     // Send a good message.
    //     assert!(inspector
    //         .dispatch_protocol_message(serde_json::to_string(&message).unwrap())
    //         .is_ok());

    //     // Send a bad message.
    //     message.method.name.insert(0, '\0');
    //     assert!(inspector
    //         .dispatch_protocol_message(serde_json::to_string(&message).unwrap())
    //         .is_err());
    // }
}