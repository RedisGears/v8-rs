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
use std::ptr::NonNull;

#[cfg(feature = "debug-server")]
pub mod messages;
#[cfg(feature = "debug-server")]
pub mod server;

use super::v8_context_scope::V8ContextScope;

/// The debugging inspector, carefully wrapping the
/// [`v8_inspector::Inspector`](https://chromium.googlesource.com/v8/v8/+/refs/heads/main/src/inspector)
/// API. An inspector is tied to the [V8Isolate] it was created for.
///
/// # Example
///
/// ```rust
/// use v8_rs::v8::*;
/// use v8_rs::v8::inspector::Inspector;
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
/// let _inspector = Inspector::new(&ctx_scope);
/// ```
#[derive(Debug)]
pub struct Inspector {
    raw: NonNull<crate::v8_c_raw::bindings::v8_inspector_c_wrapper>,
}

impl Inspector {
    /// Creates a new inspector for the provided isolate. The created
    /// inspector object has no callbacks set.
    pub fn new(context_scope: &V8ContextScope<'_, '_>) -> Self {
        let raw_context = context_scope.get_inner();

        let raw = unsafe {
            NonNull::new_unchecked(crate::v8_c_raw::bindings::v8_InspectorCreate(
                raw_context,
                None,
                std::ptr::null_mut(),
                None,
                None,
                std::ptr::null_mut(),
                None,
            ))
        };
        Self { raw }
    }

    /// Creates a new [Inspector] with callbacks.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[allow(unused)]
    pub(crate) fn new_with_callbacks<
        OnResponse: OnResponseCallback,
        OnWait: OnWaitFrontendMessageOnPauseCallback,
    >(
        context_scope: &V8ContextScope<'_, '_>,
        on_response_callback: Option<OnResponse>,
        on_wait_frontend_message_on_pause_callback: Option<OnWait>,
    ) -> Self {
        let raw_context = context_scope.get_inner();

        let (on_response_c, deallocate_on_response, on_response_callback_ptr) =
            match on_response_callback {
                Some(r) => (
                    Some(on_response::<OnResponse>),
                    Some(deallocate_box),
                    Box::into_raw(Box::new(r)),
                ),
                None => (None, None, std::ptr::null_mut()),
            };
        let (on_wait_c, deallocate_on_wait, on_wait_callback_ptr) =
            match on_wait_frontend_message_on_pause_callback {
                Some(w) => (
                    Some(on_wait_frontend_message_on_pause::<OnWait>),
                    Some(deallocate_box),
                    Box::into_raw(Box::new(w)),
                ),
                None => (None, None, std::ptr::null_mut()),
            };

        let raw = unsafe {
            NonNull::new_unchecked(crate::v8_c_raw::bindings::v8_InspectorCreate(
                raw_context,
                on_response_c.map(|r| r as _),
                on_response_callback_ptr as _,
                deallocate_on_response.map(|d| d as _),
                on_wait_c.map(|w| w as _),
                on_wait_callback_ptr as _,
                deallocate_on_wait.map(|d| d as _),
            ))
        };
        Self { raw }
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

    /// Sets the callback which is used by the debugger to send messages
    /// to the remote client.
    fn set_on_response_callback<T: OnResponseCallback>(&self, on_response_callback: T) {
        let on_response_callback = Box::new(on_response_callback);
        let on_response_callback = Box::into_raw(on_response_callback);

        unsafe {
            crate::v8_c_raw::bindings::v8_InspectorSetOnResponseCallback(
                self.raw.as_ptr(),
                Some(on_response::<T>),
                on_response_callback as _,
                Some(deallocate_box),
            );
        }
    }

    /// Sets the callback when the debugger needs to wait for the
    /// remote client's message.
    fn set_on_wait_frontend_message_on_pause_callback<T: OnWaitFrontendMessageOnPauseCallback>(
        &self,
        on_wait_frontend_message_on_pause_callback: T,
    ) {
        let on_wait_frontend_message_on_pause_callback =
            Box::new(on_wait_frontend_message_on_pause_callback);
        let on_wait_frontend_message_on_pause_callback =
            Box::into_raw(on_wait_frontend_message_on_pause_callback);

        unsafe {
            crate::v8_c_raw::bindings::v8_InspectorSetOnWaitFrontendMessageOnPauseCallback(
                self.raw.as_ptr(),
                Some(on_wait_frontend_message_on_pause::<T>),
                on_wait_frontend_message_on_pause_callback as _,
                Some(deallocate_box),
            );
        }
    }
}

impl Drop for Inspector {
    fn drop(&mut self) {
        unsafe {
            crate::v8_c_raw::bindings::v8_FreeInspector(self.raw.as_ptr());
        }
    }
}

// TODO remove and rewrite so that we don't use it.
/// Currently, we rely on the thread-safety of V8 which is said to not
/// exist.
unsafe impl Sync for Inspector {}
unsafe impl Send for Inspector {}

/// The callback which is invoked when the V8 Inspector needs to reply
/// to the client.
pub(crate) trait OnResponseCallback: FnMut(String) {}

impl<T: FnMut(String)> OnResponseCallback for T {}

/// The callback which is invoked when the V8 Inspector requires more
/// data from the front-end (the client) and, therefore, this callback
/// must attempt to read more data and dispatch it to the inspector.
///
/// The callback should return `1` when it is possible to operate (read,
/// write, send, receive messages) and `0` when not, to indicate the
/// impossibility of the further action, in which case, the inspector
/// will stop.
pub(crate) trait OnWaitFrontendMessageOnPauseCallback:
    FnMut(*mut crate::v8_c_raw::bindings::v8_inspector_c_wrapper) -> std::os::raw::c_int
{
}

impl<T: FnMut(*mut crate::v8_c_raw::bindings::v8_inspector_c_wrapper) -> std::os::raw::c_int>
    OnWaitFrontendMessageOnPauseCallback for T
{
}

/// The callback function used to send the messages back to the client
/// of the inspector (to the one who is debugging).
extern "C" fn on_response<T: OnResponseCallback>(
    string: *const ::std::os::raw::c_char,
    rust_callback: *mut ::std::os::raw::c_void,
) {
    let string = unsafe { std::ffi::CStr::from_ptr(string) }.to_string_lossy();
    log::trace!("Outgoing message: {string}");
    let rust_callback: &mut T = unsafe { &mut *(rust_callback.cast::<T>()) };
    rust_callback(string.to_string())
}

extern "C" fn on_wait_frontend_message_on_pause<T: OnWaitFrontendMessageOnPauseCallback>(
    raw: *mut crate::v8_c_raw::bindings::v8_inspector_c_wrapper,
    rust_callback: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    log::trace!("on_wait_frontend_message_on_pause");

    let rust_callback: &mut T = unsafe { &mut *(rust_callback.cast::<T>()) };
    rust_callback(raw)
}

#[allow(clippy::from_raw_with_void_ptr)]
extern "C" fn deallocate_box(raw_box: *mut ::std::os::raw::c_void) {
    unsafe { drop(Box::from_raw(raw_box as *mut _)) };
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
        let inspector = Inspector::new(&ctx_scope);

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
    //     let inspector = Inspector::new(&ctx_scope);

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
