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
//! find the [WebSocketServer] useful. The [WebSocketServer] is a very
//! simple WebSocket server, which can be used with an [Inspector].
//!
//! To start a debugging server session, it is required to have a
//! [V8ContextScope], a [WebSocketServer] and an [Inspector]. All of
//! those are gently packed within an easy-to-use [DebuggerSession]
//! struct which abstracts all the work. It is a scoped object, meaning
//! that once an object of [DebuggerSession] leaves its scope, the
//! debugger server and the debugging session are stopped.
//!
//! # Clients
//!
//! To connect to a remote debugging WebSocker server, one can use:
//!
//! 1. A custom web-socket client, and some implementation of the
//! V8 Inspector protocol.
//! 2. The chrome-devtools, for example, by running the chromium
//! web-browser and navigating to
//! <devtools://devtools/bundled/inspector.html?ws=ip:port> where
//! the `ip`/`port` parameters are the corresponding ip address and port
//! of the [WebSocketServer] used.
//! 3. (Needs verification) Using the WebKit MiniBrowser
//! <https://trac.webkit.org/wiki/RemoteInspectorGTKandWPE>.
//! 4. Using the V8's [D8 console debugger](https://v8.dev/docs/d8).
//! 5. Using the [Visual Studio Code](https://code.visualstudio.com/),
//! for example, using a configuration like this:
//!     ```json
//!     {
//!         "version": "0.2.0",
//!         "configurations": [
//!             {
//!                 "name": "Attach to RedisGears V8 on port 9005",
//!                 "type": "node",
//!                 "request": "attach",
//!                 "cwd": "${workspaceFolder}",
//!                 "websocketAddress": "ws://127.0.0.1:9005",
//!             }
//!         ]
//!     }
//!     ```
//!
use std::marker::PhantomData;
use std::net::TcpListener;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Duration;

use serde::Deserialize;

use crate::v8::v8_context::V8Context;
use crate::v8::v8_context_scope::V8ContextScope;

/// The remote debugging server port for the [WebSocketServer].
const PORT_V4: u16 = 9005;
/// The remote debugging server ip address for the [WebSocketServer].
const IP_V4: std::net::Ipv4Addr = std::net::Ipv4Addr::LOCALHOST;
/// The full remote debugging server host name for the [WebSocketServer].
pub const LOCAL_HOST: std::net::SocketAddrV4 = std::net::SocketAddrV4::new(IP_V4, PORT_V4);
/// The V8 method which is invoked when a client has successfully
/// connected to the [Inspector] server and waits for the debugging
/// session to start.
const DEBUGGER_SHOULD_START_METHOD_NAME: &str = "Runtime.runIfWaitingForDebugger";
/// The default read timeout duration.
const DEFAULT_READ_TIMEOUT_DURATION: Duration = Duration::from_millis(100);

/// The debugging server which waits for a connection of a remote
/// debugger, receives messages from there and sends the replies back.
#[derive(Debug)]
struct TcpServer {
    /// The server that accepts remote debugging connections.
    server: TcpListener,
}

impl TcpServer {
    /// Creates a new [Server] object with a tcp listener to the specified
    /// address.
    pub fn new<T: std::net::ToSocketAddrs>(address: T) -> Result<Self, std::io::Error> {
        let server = TcpListener::bind(address)?;
        Ok(Self { server })
    }

    /// Returns the currently listening address.
    pub fn get_listening_address(&self) -> Result<std::net::SocketAddr, std::io::Error> {
        self.server.local_addr()
    }

    /// Starts listening for and a new single websocket connection.
    /// Once the connection is accepted, it is returned to the user.
    pub fn accept_next_websocket_connection(&self) -> Result<WebSocketServer, std::io::Error> {
        tungstenite::accept(self.server.accept()?.0)
            .map(WebSocketServer::from)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

/// A WebSocket server.
#[repr(transparent)]
#[derive(Debug)]
pub struct WebSocketServer(tungstenite::WebSocket<std::net::TcpStream>);
impl WebSocketServer {
    /// Waits for a message available to read, and once there is one,
    /// reads it and returns as a text.
    pub fn read_next_message(&mut self) -> Result<String, std::io::Error> {
        log::trace!("Reading the next message.");
        match self.0.read_message() {
            Ok(message) => message
                .into_text()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)),
            Err(tungstenite::Error::Io(e)) => Err(e),
            Err(tungstenite::Error::ConnectionClosed | tungstenite::Error::AlreadyClosed) => {
                Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "The WebSocket connection has been closed.",
                ))
            }
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }
}

impl From<tungstenite::WebSocket<std::net::TcpStream>> for WebSocketServer {
    fn from(value: tungstenite::WebSocket<std::net::TcpStream>) -> Self {
        value
            .get_ref()
            .set_read_timeout(Some(DEFAULT_READ_TIMEOUT_DURATION))
            .expect("Couldn't set the read timeout.");

        Self(value)
    }
}

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

    /// Resets the `onResponse` callback. See [Self::set_on_response_callback].
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
        let string = match std::ffi::CString::new(message.as_ref()) {
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

/// A method invocation message.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct MethodInvocation {
    /// The name of the method.
    #[serde(rename = "method")]
    name: String,
    /// The parameters to pass to the method.
    params: serde_json::Map<String, serde_json::Value>,
}

/// A message from the debugger front-end (from the client to the
/// server).
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct IncomingMessage {
    /// The ID of the message.
    id: u64,
    /// The method information.
    #[serde(flatten)]
    method: MethodInvocation,
}

/// An error message.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct ErrorMessage {
    /// The error code.
    code: i32,
    /// The error message.
    message: String,
}

/// A message from the server to the client (from the back-end to the
/// front-end).
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum OutgoingMessage {
    Error {
        error: ErrorMessage,
    },
    Invoke(MethodInvocation),
    Result {
        id: u64,
        result: serde_json::Map<String, serde_json::Value>,
    },
}

struct InspectorCallbacks {
    on_response: Box<OnResponseCallback>,
    on_wait_frontend_message_on_pause: Box<OnWaitFrontendMessageOnPauseCallback>,
}

/// A single debugger session.
#[derive(Debug)]
pub struct DebuggerSession<'inspector, 'context_scope, 'isolate_scope, 'isolate> {
    web_socket: Rc<Mutex<WebSocketServer>>,
    inspector: &'inspector mut Inspector<'context_scope, 'isolate_scope, 'isolate>,
}

impl<'inspector, 'context_scope, 'isolate_scope, 'isolate>
    DebuggerSession<'inspector, 'context_scope, 'isolate_scope, 'isolate>
{
    fn create_inspector_callbacks(web_socket: Rc<Mutex<WebSocketServer>>) -> InspectorCallbacks {
        let websocket = web_socket.clone();

        let on_response = move |s: String| {
            log::trace!("Responding with string {s}.");
            match websocket.lock() {
                Ok(mut websocket) => match websocket.0.write_message(tungstenite::Message::Text(s))
                {
                    Ok(_) => log::trace!("Responded with string successfully."),
                    Err(e) => log::error!("Couldn't send a websocket message to the client: {e}"),
                },
                Err(e) => log::error!("Couldn't lock the socket: {e}"),
            }
        };

        let on_wait_frontend_message_on_pause = move |raw: *mut crate::v8_c_raw::bindings::v8_inspector_c_wrapper| -> std::os::raw::c_int {
            let string;

            loop {
                match web_socket.lock() {
                    Ok(mut websocket) => match websocket.read_next_message() {
                        Ok(message) => {
                            log::trace!("[OnWait] Read the message: {message:?}");

                            string = match std::ffi::CString::new(message) {
                                Ok(string) => string,
                                _ => continue,
                            };
                            break;
                        }
                        Err(e) => if e.kind() == std::io::ErrorKind::ConnectionAborted {
                            return 0;
                        }
                    },
                    Err(e) => {
                        log::error!("The WebSocketServer mutex is poisoned: {e:?}");
                        return 0;
                    },
                }
            }

            unsafe {
                crate::v8_c_raw::bindings::v8_InspectorDispatchProtocolMessage(
                    raw,
                    string.as_ptr(),
                )
            }

            1
        };

        InspectorCallbacks {
            on_response: Box::new(on_response),
            on_wait_frontend_message_on_pause: Box::new(on_wait_frontend_message_on_pause),
        }
    }

    /// Creates a new debugger to be used with the provided [Inspector].
    /// Starts a web socket debugging session using [WebSocketServer].
    ///
    /// Once the connection is accepted and the inspector starts, the
    /// function returns.
    ///
    /// After the function returns, to start the debugging main loop,
    /// one needs to call the [Self::process_messages].
    /// method.
    pub fn new<T: std::net::ToSocketAddrs>(
        inspector: &'inspector mut Inspector<'context_scope, 'isolate_scope, 'isolate>,
        address: T,
    ) -> Result<Self, std::io::Error> {
        let server = TcpServer::new(address)?;
        let address = server.get_listening_address()?;
        let vscode_configuration = format!(
            r#"
        {{
            "version": "0.2.0",
            "configurations": [
                {{
                    "name": "Attach to RedisGears V8 through WebSocket at {address}",
                    "type": "node",
                    "request": "attach",
                    "cwd": "${{workspaceFolder}}",
                    "websocketAddress": "ws://{address}",
                }}
            ]
        }}
        "#
        );
        log::info!(
            "The V8 remote debugging server is waiting for a connection via WebSocket on: {address}.\nHint: you may browse this link: <devtools://devtools/bundled/inspector.html?experiments=true&v8only=true&ws={address}> or use this launch configuration for Visual Studio Code (<https://code.visualstudio.com/>):{vscode_configuration}",
        );

        let web_socket = Rc::new(Mutex::new(server.accept_next_websocket_connection()?));
        log::trace!("Accepted the next websocket.");

        let callbacks = Self::create_inspector_callbacks(web_socket.clone());

        inspector.set_on_response_callback(callbacks.on_response);
        inspector.set_on_wait_frontend_message_on_pause_callback(
            callbacks.on_wait_frontend_message_on_pause,
        );

        let session = Self {
            web_socket,
            inspector,
        };

        // The re-locking read loop to wait until the remote debugger
        // is ready to start.
        loop {
            let message_string: String = session.read_next_message()?;
            let message: IncomingMessage = match serde_json::from_str(&message_string) {
                Ok(message) => message,
                Err(_) => continue,
            };
            log::trace!("Parsed out the incoming message: {message:?}");
            session.inspector.dispatch_protocol_message(&message_string);

            if message.method.name == DEBUGGER_SHOULD_START_METHOD_NAME {
                session
                    .inspector
                    .schedule_pause_on_next_statement("Debugger started.");
                session.inspector.wait_frontend_message_on_pause();

                return Ok(session);
            }
        }
    }

    /// Reads the next message without parsing it.
    pub fn read_next_message(&self) -> Result<String, std::io::Error> {
        loop {
            if let Ok(mut websocket) = self.web_socket.lock() {
                match websocket.read_next_message() {
                    Ok(s) => {
                        if let Ok(None) = websocket.0.get_ref().read_timeout() {
                            websocket
                                .0
                                .get_ref()
                                .set_read_timeout(Some(DEFAULT_READ_TIMEOUT_DURATION))
                                .expect("Failed to temporarily reset the read timeout");
                        }
                        return Ok(s);
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::TimedOut {
                            continue;
                        } else if e.kind() == std::io::ErrorKind::WouldBlock {
                            websocket
                                .0
                                .get_ref()
                                .set_read_timeout(None)
                                .expect("Failed to temporarily reset the read timeout");
                            log::trace!("Reset the timeout due to WouldBlock.");
                            continue;
                        } else {
                            return Err(e);
                        }
                    }
                }
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "The mutex is poisoned.",
                ));
            }
        }
    }

    /// Reads (without parsing), proccesses and then returns the next
    /// message from the client.
    pub fn read_and_process_next_message(&self) -> Result<String, std::io::Error> {
        let message = self.read_next_message()?;
        log::trace!("Got incoming websocket message: {message}");
        self.inspector.dispatch_protocol_message(&message);
        Ok(message)
    }

    /// Reads and processes all the next messages in a loop, until
    /// the connection is dropped by the client or until an error state
    /// is reached.
    pub fn process_messages(&self) -> Result<(), std::io::Error> {
        log::trace!("Inspector main loop started.");
        loop {
            if let Err(e) = self.read_and_process_next_message() {
                if e.kind() == std::io::ErrorKind::ConnectionAborted {
                    log::trace!("Inspector main loop successfully stopped.");
                    return Ok(());
                } else {
                    return Err(e);
                }
            }
        }
    }

    /// Schedules a pause (sets a breakpoint) for the next statement.
    /// See [Inspector::schedule_pause_on_next_statement].
    pub fn schedule_pause_on_next_statement(&self) {
        self.inspector
            .schedule_pause_on_next_statement("User breakpoint.");
    }
}

impl<'inspector, 'context_scope, 'isolate_scope, 'isolate> Drop
    for DebuggerSession<'inspector, 'context_scope, 'isolate_scope, 'isolate>
{
    fn drop(&mut self) {
        self.inspector.reset_on_response_callback();
        self.inspector
            .reset_on_wait_frontend_message_on_pause_callback();
    }
}
