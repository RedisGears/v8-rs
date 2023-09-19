/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! This module provides with the debug server structures. With these
//! structures it is possibly to easily start a debug server and start
//! processing remote debuggers' messages over the network. The module
//! implements messaging via the `WebSocket` protocol, as it is mainly
//! used together with Chrome V8 engine for debugging, however, the
//! implementation is platform-agnostic and one can use any sort of
//! transport for the messages.
//!
//! # Clients
//!
//! To connect to a remote debugging WebSocket server, one can use:
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
//! # Example
//!
//! To debug, we must properly initialise the V8 and have a context
//! scope ([super::V8ContextScope]), on which we can create an
//! inspector.
//!
//! An inspector is just a hook into the `V8Platform` spying on the
//! actions inside the [crate::v8::v8_context::V8Context] and which
//! can pause, inspect and continue the execution.
//!
//! ```rust
//! use v8_rs::v8::*;
//! use inspector::messages::ClientMessage;
//! use inspector::server::{DebuggerSession, TcpServer};
//! use std::sync::{Arc, Mutex};
//! use v8_rs::v8::inspector::Inspector;
//!
//! // Initialise the V8 engine:
//! v8_init_platform(1, Some("--expose-gc")).unwrap();
//! v8_init().unwrap();
//!
//! // Create a new isolate:
//! let isolate = isolate::V8Isolate::new();
//!
//! // Enter the isolate created:
//! let i_scope = isolate.enter();
//!
//! // Create the code string object:
//! let code_str = i_scope.new_string("1+1");
//!
//! // Create a JS execution context for code invocation:""
//! let ctx = i_scope.new_context(None);
//!
//! // Enter the created execution context for debugging:
//! let ctx_scope = ctx.enter(&i_scope);
//!
//! // Create an inspector.
//! let inspector = Arc::new(Inspector::new(&ctx_scope));
//!
//! let mut stage_1 = Arc::new(Mutex::new(()));
//! let mut stage_2 = Arc::new(Mutex::new(()));
//!
//! let lock_1 = stage_1.lock().unwrap();
//! let lock_2 = stage_2.lock().unwrap();
//!
//! // The remote debugging server port for the [WebSocketServer].
//! const PORT_V4: u16 = 9005;
//! // The remote debugging server ip address for the [WebSocketServer].
//! const IP_V4: std::net::Ipv4Addr = std::net::Ipv4Addr::LOCALHOST;
//! // The full remote debugging server host name for the [WebSocketServer].
//! const LOCAL_HOST: std::net::SocketAddrV4 =
//!     std::net::SocketAddrV4::new(IP_V4, PORT_V4);
//!
//! let address = LOCAL_HOST.to_string();
//! let address = &address;
//!
//! let fake_client = {
//!     use std::net::TcpStream;
//!     use tungstenite::protocol::{WebSocket, Message};
//!     use tungstenite::stream::MaybeTlsStream;
//!
//!     let address = address.clone();
//!     let stage_1 = stage_1.clone();
//!
//!     #[derive(Copy, Clone, Default, Debug)]
//!     struct Client {
//!         last_message_id: u64,
//!     }
//!     impl Client {
//!         fn send_ready(
//!             &mut self,
//!             ws: &mut WebSocket<MaybeTlsStream<TcpStream>>
//!         ) {
//!             let message = ClientMessage::new_client_ready(self.last_message_id);
//!             let message = Message::Text(serde_json::to_string(&message).unwrap());
//!             ws.send(message).expect("Couldn't send the message");
//!         }
//!     }
//!
//!     std::thread::spawn(move || {
//!         let mut ws: WebSocket<MaybeTlsStream<TcpStream>>;
//!         loop {
//!             match tungstenite::connect(format!("ws://{address}")) {
//!                 Ok(s) => { ws = s.0; break; },
//!                 _ => continue,
//!             }
//!         };
//!         let mut client = Client::default();
//!         client.send_ready(&mut ws);
//!         let _ = ws.read();
//!         drop(stage_1.lock().expect("Couldn't lock the stage 1"));
//!         ws.close(None).expect("Couldn't close the WebSocket");
//!     })
//! };
//!
//!
//! // Let's create a server and start listening for the connections
//! // on the address provided, but not accepting those yet.
//! let server = TcpServer::new(address).expect("Couldn't create a tcp server");
//!
//! // Now let's wait for the user to connect.
//! let web_socket = server
//!     .accept_next_websocket_connection()
//!     .expect("Couldn't accept the next web socket connection");
//!
//! // Now that we have accepted a connection, we can start our
//! // debugging session.
//! let debugger_session = DebuggerSession::new(web_socket, inspector, &i_scope)
//!     .expect("Couldn't start a debugging session");
//!
//! // At this point, the server is running and has accepted a remote
//! // client. Once the client connects, the debugger pauses on the very
//! // first (next) instruction, so we can safely attempt to run the
//! // script, as it won't actually run but will wait for remote client
//! // to act.
//!
//! // Compile the code:
//! let script = ctx_scope.compile(&code_str).unwrap();
//!
//! // Allow the fake client to stop (for this test not to hang).
//! drop(lock_1);
//!
//! // Run the compiled code:
//! let res = script.run(&ctx_scope).unwrap();
//!
//! // To let the remote debugger operate, we need to be able to send
//! // and receive data to and from it. This is achieved by starting the
//! // main loop of the debugger session:
//! assert!(debugger_session.process_messages(&i_scope).is_ok());
//!
//! fake_client.join();
//!
//! // Get the result:
//! let res_utf8 = res.to_utf8().unwrap();
//! assert_eq!(res_utf8.as_str(), "2");
//! ```
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tungstenite::{Error, Message, WebSocket};

use crate::v8::inspector::messages::ClientMessage;
use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8_c_raw::bindings::v8_inspector_c_wrapper;

use super::{Inspector, OnResponseCallback, OnWaitFrontendMessageOnPauseCallback};

/// The debugging server which waits for a connection of a remote
/// debugger, receives messages from there and sends the replies back.
#[derive(Debug)]
pub struct TcpServer {
    /// The server that accepts remote debugging connections.
    server: TcpListener,
}

impl TcpServer {
    /// Creates a new [TcpServer] object with a tcp listener to the specified
    /// address.
    pub fn new<T: std::net::ToSocketAddrs>(address: T) -> Result<Self, std::io::Error> {
        let server = TcpListener::bind(address)?;
        Ok(Self { server })
    }

    /// Returns the currently listening address.
    pub fn get_listening_address(&self) -> Result<std::net::SocketAddr, std::io::Error> {
        self.server.local_addr()
    }

    /// Starts listening for a new single websocket connection.
    /// The socket attempts to accept a connection in a non-blocking
    /// mode, meaning it would return [`std::io::ErrorKind::WouldBlock`]
    /// in case there is no user connection.
    ///
    /// Once the connection is accepted, it is returned to the user.
    pub fn try_accept_next_websocket_connection(
        self,
    ) -> Result<WebSocketServer, (Self, std::io::Error)> {
        if let Err(e) = self.server.set_nonblocking(true) {
            return Err((self, e));
        }

        let connection = match self.server.accept() {
            Ok(connection) => connection,
            Err(e) => return Err((self, e)),
        };

        if let Err(e) = self.server.set_nonblocking(false) {
            return Err((self, e));
        }

        tungstenite::accept(connection.0)
            .map(WebSocketServer::from)
            .map_err(|e| (self, std::io::Error::new(std::io::ErrorKind::Other, e)))
    }

    /// Starts listening for a new single websocket connection.
    /// Once the connection is accepted, it is returned to the user.
    pub fn accept_next_websocket_connection(self) -> Result<WebSocketServer, std::io::Error> {
        let connection = self.server.accept()?;
        tungstenite::accept(connection.0)
            .map(WebSocketServer::from)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Returns the ways to connect to the server to establish a new
    /// debugger session.
    pub fn get_connection_hints(&self) -> Option<DebuggerSessionConnectionHints> {
        self.get_listening_address().ok().map(|a| a.into())
    }
}

/// A WebSocket server.
#[derive(Debug)]
pub struct WebSocketServer(WebSocket<TcpStream>);
impl WebSocketServer {
    /// The default read timeout duration.
    const DEFAULT_READ_TIMEOUT_DURATION: Duration = Duration::from_millis(100);

    /// Returns the ways to connect to the server to establish a new
    /// debugger session.
    pub fn get_connection_hints(&self) -> Result<DebuggerSessionConnectionHints, std::io::Error> {
        Ok(DebuggerSessionConnectionHints::from(
            self.0.get_ref().local_addr()?,
        ))
    }

    /// Returns [`true`] if there is data available to read.
    pub fn has_data_to_read(&mut self) -> Result<bool, std::io::Error> {
        let mut bytes = [0];
        if self.0.can_read() && self.0.get_ref().peer_addr().is_ok() {
            self.0.get_ref().peek(&mut bytes).map(|b| b == 1)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "The connection is aborted.",
            ))
        }
    }

    fn process_message(
        &mut self,
        message: tungstenite::Result<Message>,
    ) -> Result<Option<String>, std::io::Error> {
        let closed_error = || {
            std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                "The WebSocket connection has been closed.",
            )
        };

        let convert_error = |e| -> std::io::Error {
            match e {
                Error::Io(e) => e,
                Error::ConnectionClosed | Error::AlreadyClosed => closed_error(),
                e => std::io::Error::new(std::io::ErrorKind::Other, e),
            }
        };

        match message {
            Ok(message) => match message {
                Message::Close(_) => Err(closed_error()),
                Message::Ping(payload) => {
                    log::trace!("Sending Pong.");

                    self.0
                        .send(Message::Pong(payload))
                        .map(|_| None)
                        .map_err(convert_error)
                }
                message => message.into_text().map(Some).map_err(convert_error),
            },
            Err(e) => Err(convert_error(e)),
        }
    }

    /// Waits for a message available to read, and once there is one,
    /// reads it and returns as a text.
    pub fn read_next_message(&mut self) -> Result<String, std::io::Error> {
        log::trace!("Reading the next message (blocking).");

        let message = self.0.read();
        self.process_message(message).map(|m| m.unwrap_or_default())
    }

    /// Attempts to read the next message.
    pub fn try_read_next_message(&mut self) -> Result<Option<String>, std::io::Error> {
        if !self.has_data_to_read()? {
            return Ok(None);
        }

        log::trace!("Reading the next message (non-blocking).");

        let message = self.0.read();
        self.process_message(message)
    }

    /// Sets a read timeout. Setting [`None`] removes the timeout.
    pub fn set_read_timeout(&self, duration: Option<std::time::Duration>) -> std::io::Result<()> {
        self.0.get_ref().set_read_timeout(duration)
    }
}

impl From<WebSocket<TcpStream>> for WebSocketServer {
    fn from(value: WebSocket<TcpStream>) -> Self {
        value
            .get_ref()
            .set_read_timeout(Some(Self::DEFAULT_READ_TIMEOUT_DURATION))
            .expect("Couldn't set the read timeout.");

        Self(value)
    }
}

struct InspectorCallbacks<R: OnResponseCallback, W: OnWaitFrontendMessageOnPauseCallback> {
    on_response: R,
    on_wait_frontend_message_on_pause: W,
}

/// The means of connection to the V8 debugger.
#[derive(Debug, Clone)]
pub struct DebuggerSessionConnectionHints {
    address: std::net::SocketAddr,
    vscode_configuration: String,
    chromium_link: String,
}

impl DebuggerSessionConnectionHints {
    /// Returns a [std::net::SocketAddr] address on which the server is
    /// going to be listening.
    pub fn get_address(&self) -> std::net::SocketAddr {
        self.address
    }

    /// Returns a hint on how to connect to the remote debugger server
    /// for remote debugging using Visual Studio Code.
    pub fn get_visual_studio_code_configuration(&self) -> &str {
        &self.vscode_configuration
    }

    /// Returns a hint on how to connect to the remote debugger server
    /// for remote debugging using Chromium-based browsers.
    pub fn get_chromium_link(&self) -> &str {
        &self.chromium_link
    }
}

impl std::fmt::Display for DebuggerSessionConnectionHints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let address = &self.address;
        let vscode_configuration = &self.vscode_configuration;
        let chromium_link = &self.chromium_link;
        f.write_fmt(format_args!("The V8 remote debugging server is waiting for a connection via WebSocket on: {address}.\nHint: you may browse this link: {chromium_link} or use this launch configuration for Visual Studio Code (<https://code.visualstudio.com/>):{vscode_configuration}"))
    }
}

impl<T: Into<std::net::SocketAddr>> From<T> for DebuggerSessionConnectionHints {
    fn from(address: T) -> Self {
        let address = address.into();
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
        let chromium_link = format!(
            "devtools://devtools/bundled/inspector.html?experiments=true&v8only=true&ws={address}"
        );
        DebuggerSessionConnectionHints {
            address,
            vscode_configuration,
            chromium_link,
        }
    }
}

/// A single debugger session.
#[derive(Debug)]
pub struct DebuggerSession {
    web_socket: Rc<Mutex<WebSocketServer>>,
    inspector: Arc<Inspector>,
    connection_hints: DebuggerSessionConnectionHints,
}

impl DebuggerSession {
    fn create_inspector_callbacks(
        web_socket: Rc<Mutex<WebSocketServer>>,
    ) -> InspectorCallbacks<impl Fn(String), impl Fn(*mut v8_inspector_c_wrapper) -> i32> {
        let websocket = web_socket.clone();

        let on_response = move |s: String| {
            if let Ok(message) = serde_json::from_str::<ClientMessage>(&s) {
                if let Some(parsed_script) = message.method.get_script_parsed() {
                    log::trace!("Parsed script: {parsed_script:?}");
                }
            }
            log::trace!("Responding with string {s}.");
            match websocket.lock() {
                Ok(mut websocket) => match websocket.0.send(Message::Text(s)) {
                    Ok(_) => log::trace!("Responded with string successfully."),
                    Err(e) => log::error!("Couldn't send a websocket message to the client: {e}"),
                },
                Err(e) => log::error!("Couldn't lock the socket: {e}"),
            }
        };

        let on_wait_frontend_message_on_pause = move |raw: *mut crate::v8_c_raw::bindings::v8_inspector_c_wrapper| -> std::os::raw::c_int {
            // When this is returned, the callback will be called again,
            // if no error occured.
            const CONTINUE_WAITING: std::os::raw::c_int = 0;
            // Returning this would result in stopping the wait.
            const STOP_WAITING: std::os::raw::c_int = 1;

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
                            return CONTINUE_WAITING;
                        }
                    },
                    Err(e) => {
                        log::error!("The WebSocketServer mutex is poisoned: {e:?}");
                        return CONTINUE_WAITING;
                    },
                }
            }

            unsafe {
                crate::v8_c_raw::bindings::v8_InspectorDispatchProtocolMessage(
                    raw,
                    string.as_ptr(),
                )
            }

            STOP_WAITING
        };

        InspectorCallbacks {
            on_response,
            on_wait_frontend_message_on_pause,
        }
    }

    /// Creates a new debugger to be used with the provided [Inspector].
    /// Starts a web socket debugging session using [WebSocketServer].
    ///
    /// Once the connection is accepted and the inspector starts, the
    /// function returns.
    ///
    /// After the function returns, to start the debugging main loop,
    /// one needs to call the [`Self::process_messages`].
    /// method.
    ///
    /// # Notes
    ///
    /// This method requires a [`V8IsolateScope`].
    pub fn new(
        web_socket: WebSocketServer,
        inspector: Arc<Inspector>,
        isolate_scope: &V8IsolateScope<'_>,
    ) -> Result<Self, std::io::Error> {
        let connection_hints = web_socket.get_connection_hints()?;
        let web_socket = Rc::new(Mutex::new(web_socket));
        let callbacks = Self::create_inspector_callbacks(web_socket.clone());
        inspector.set_on_response_callback(callbacks.on_response);
        inspector.set_on_wait_frontend_message_on_pause_callback(
            callbacks.on_wait_frontend_message_on_pause,
        );

        let session = Self {
            web_socket,
            inspector,
            connection_hints,
        };

        let inspector_guard = session.inspector.guard(isolate_scope)?;

        // The re-locking read loop to wait until the remote debugger
        // is ready to start.
        loop {
            let message_string: String = session.read_next_message()?;
            let message: ClientMessage = match serde_json::from_str(&message_string) {
                Ok(message) => message,
                Err(_) => continue,
            };

            log::trace!("Parsed out the incoming message: {message:?}");

            inspector_guard.dispatch_protocol_message(&message_string)?;

            if message.is_client_ready() {
                return Ok(session);
            }
        }
    }

    /// Returns the ways to connect to the server to establish a new
    /// debugger session.
    pub fn get_connection_hints(&self) -> &DebuggerSessionConnectionHints {
        &self.connection_hints
    }

    /// Sets the read timeout for the web socket server.
    pub fn set_read_timeout(
        &self,
        duration: Option<std::time::Duration>,
    ) -> Result<(), std::io::Error> {
        self.web_socket
            .lock()
            .expect("Couldn't lock the WebSocketServer mutex")
            .set_read_timeout(duration)
    }

    /// Reads the next message without parsing it.
    pub fn read_next_message(&self) -> Result<String, std::io::Error> {
        loop {
            if let Ok(mut websocket) = self.web_socket.lock() {
                match websocket.read_next_message() {
                    Ok(s) => {
                        if let Ok(None) = websocket.0.get_ref().read_timeout() {
                            websocket
                                .set_read_timeout(Some(
                                    WebSocketServer::DEFAULT_READ_TIMEOUT_DURATION,
                                ))
                                .expect("Failed to temporarily reset the read timeout");
                        }
                        return Ok(s);
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::TimedOut {
                            continue;
                        } else if e.kind() == std::io::ErrorKind::WouldBlock {
                            websocket
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

    /// Attempts to read the next message, if it is available. Returns
    /// immediately if there is nothing to read, without waiting for
    /// the message to be received.
    pub fn try_read_next_message(&self) -> Result<Option<String>, std::io::Error> {
        if let Ok(mut websocket) = self.web_socket.lock() {
            websocket.try_read_next_message()
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "The mutex is poisoned.",
            ))
        }
    }

    /// Waits for a message to read, reads (without parsing), proccesses
    /// it and then it.
    pub fn read_and_process_next_message(
        &self,
        isolate_scope: &V8IsolateScope<'_>,
    ) -> Result<String, std::io::Error> {
        let message = self.read_next_message()?;
        log::trace!("Got incoming websocket message: {message}");
        self.inspector
            .guard(isolate_scope)?
            .dispatch_protocol_message(&message)?;
        Ok(message)
    }

    /// Attempts to read a message from the client. If there are no
    /// messages available to read at this time, [`None`] is returned.
    pub fn try_read_and_process_next_message(
        &self,
        isolate_scope: &V8IsolateScope<'_>,
    ) -> Result<Option<String>, std::io::Error> {
        let message = self.try_read_next_message()?;

        if let Some(ref message) = message {
            log::trace!(
                "Got incoming websocket message: {message}, len={}",
                message.len()
            );
            self.inspector
                .guard(isolate_scope)?
                .dispatch_protocol_message(message)?;
        }
        Ok(message)
    }

    /// Reads and processes all the next messages in a loop, until
    /// the connection is dropped by the client or until an error state
    /// is reached.
    pub fn process_messages(
        &self,
        isolate_scope: &V8IsolateScope<'_>,
    ) -> Result<(), std::io::Error> {
        log::trace!("Inspector main loop started.");
        loop {
            if let Err(e) = self.read_and_process_next_message(isolate_scope) {
                if e.kind() == std::io::ErrorKind::ConnectionAborted {
                    log::trace!("Inspector main loop successfully stopped.");
                    return Ok(());
                } else {
                    return Err(e);
                }
            }
        }
    }

    /// Reads and processes all the next messages queued, until
    /// the read timeout is reached, or the connection is dropped by the
    /// client or until any other error state is reached.
    ///
    /// Returns [`true`] if the client has disconnected and the remote
    /// debugging is thus no longer possible.
    ///
    /// # Notes
    ///
    /// This method requires a [`V8IsolateScope`].
    pub fn process_messages_with_timeout(
        &self,
        duration: std::time::Duration,
        isolate_scope: &V8IsolateScope<'_>,
    ) -> Result<bool, std::io::Error> {
        self.set_read_timeout(Some(duration))?;

        if let Err(e) = self.try_read_and_process_next_message(isolate_scope) {
            if e.kind() == std::io::ErrorKind::ConnectionAborted {
                Ok(true)
            } else if e.kind() == std::io::ErrorKind::WouldBlock
                || e.kind() == std::io::ErrorKind::TimedOut
                || e.kind() == std::io::ErrorKind::Interrupted
            {
                return Ok(false);
            } else {
                log::trace!("Other message error: {e:?}");
                return Err(e);
            }
        } else {
            Ok(false)
        }
    }

    /// Schedules a pause (sets a breakpoint) for the next statement.
    /// See [`super::Inspector::schedule_pause_on_next_statement`].
    pub fn schedule_pause_on_next_statement(
        &self,
        isolate_scope: &V8IsolateScope<'_>,
    ) -> Result<(), std::io::Error> {
        self.inspector
            .guard(isolate_scope)?
            .schedule_pause_on_next_statement("User breakpoint.")
    }

    /// Stops the debugging session if it has been established.
    pub fn stop(&self) {
        if let Ok(mut ws) = self.web_socket.lock() {
            if ws.0.can_write() {
                if let Err(e) = ws.0.send(Message::Close(None)) {
                    log::trace!("Couldn't stop the debugging session: {e}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::v8::{
        inspector::{
            server::{DebuggerSession, TcpServer},
            Inspector,
        },
        isolate::V8Isolate,
    };

    use super::ClientMessage;
    use std::sync::{atomic::AtomicU16, Arc, Mutex};

    static PORT_GENERATOR: AtomicU16 = AtomicU16::new(9006u16);

    /// This is to test the crash when setting a breakpoint.
    /// It:
    /// 1. Starts the V8 engine.
    /// 2. Creates all the facilities required for a simple script.
    /// 3. Starts a websocket server on the host.
    /// 4. Accepts a connection and starts the debugging.
    /// 5. Uses a fake client to send the breakpoint setting message.
    /// 6. Ensures the message is read by the server without a problem.
    #[test]
    fn can_set_breakpoint() {
        // Initialise the V8 engine:
        crate::test_utils::initialize();

        // Create a new isolate:
        let isolate = V8Isolate::new();

        // Enter the isolate created:
        let isolate_scope = isolate.enter();

        // Create the code string object:
        let code_str = isolate_scope.new_string("1+1");

        // Create a JS execution context for code invocation:""
        let context = isolate_scope.new_context(None);

        // Enter the created execution context for debugging:
        let context_scope = context.enter(&isolate_scope);

        let inspector = Arc::new(Inspector::new(&context_scope));

        let stage_1 = Arc::new(Mutex::new(()));

        let lock_1 = stage_1.lock().unwrap();

        // The remote debugging server port for the [WebSocketServer].
        let port = PORT_GENERATOR.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        // The remote debugging server ip address for the [WebSocketServer].
        const IP_V4: std::net::Ipv4Addr = std::net::Ipv4Addr::LOCALHOST;
        // The full remote debugging server host name for the [WebSocketServer].
        let host: std::net::SocketAddrV4 = std::net::SocketAddrV4::new(IP_V4, port);

        let address = host.to_string();
        let address = &address;

        let fake_client = {
            use std::net::TcpStream;
            use tungstenite::protocol::{Message, WebSocket};
            use tungstenite::stream::MaybeTlsStream;
            let address = address.clone();

            let stage_1 = stage_1.clone();
            #[derive(Copy, Clone, Default, Debug)]
            struct Client {
                last_message_id: u64,
            }

            impl Client {
                fn send_message(
                    &mut self,
                    ws: &mut WebSocket<MaybeTlsStream<TcpStream>>,
                    message: ClientMessage,
                ) {
                    let message = Message::Text(serde_json::to_string(&message).unwrap());
                    ws.send(message).expect("Couldn't send the message");
                    self.last_message_id += 1;
                }

                fn send_ready(&mut self, ws: &mut WebSocket<MaybeTlsStream<TcpStream>>) {
                    let message = ClientMessage::new_client_ready(self.last_message_id);
                    self.send_message(ws, message)
                }

                fn send_breakpoint(
                    &mut self,
                    ws: &mut WebSocket<MaybeTlsStream<TcpStream>>,
                    column: u64,
                    line: u64,
                    url: &str,
                ) {
                    let message =
                        ClientMessage::new_breakpoint(self.last_message_id, column, line, url);
                    self.send_message(ws, message)
                }
            }

            std::thread::spawn(move || {
                let mut ws: WebSocket<MaybeTlsStream<TcpStream>>;
                loop {
                    match tungstenite::connect(format!("ws://{address}")) {
                        Ok(s) => {
                            ws = s.0;
                            break;
                        }
                        _ => continue,
                    }
                }
                let mut client = Client::default();
                client.send_ready(&mut ws);
                let _ = ws.read();
                let project_path = env!("CARGO_MANIFEST_DIR");
                client.send_breakpoint(
                    &mut ws,
                    0,
                    0,
                    &format!(
                        "
                file://{project_path}/src/lib.rs($|?)|{project_path}/src/lib.rs($|?)"
                    ),
                );
                drop(stage_1.lock().expect("Couldn't lock the stage 1"));
                ws.close(None).expect("Couldn't close the WebSocket");
            })
        };

        // Let's create a server and start listening for the connections
        // on the address provided, but not accepting those yet.
        let server = TcpServer::new(address).expect("Couldn't create a tcp server");

        // Now let's wait for the user to connect.
        let web_socket = server
            .accept_next_websocket_connection()
            .expect("Couldn't accept the next web socket connection");

        // Now that we have accepted a connection, we can start our
        // debugging session.
        let debugger_session = DebuggerSession::new(web_socket, inspector, &isolate_scope)
            .expect("Couldn't start a debugging session");

        // At this point, the server is running and has accepted a remote
        // client. Once the client connects, the debugger pauses on the very
        // first (next) instruction, so we can safely attempt to run the
        // script, as it won't actually run but will wait for remote client
        // to act.
        // Compile the code:
        let script = context_scope.compile(&code_str).unwrap();
        // Allow the fake client to stop (for this test not to hang).
        drop(lock_1);
        // Run the compiled code:
        let res = script.run(&context_scope).unwrap();
        // To let the remote debugger operate, we need to be able to send
        // and receive data to and from it. This is achieved by starting the
        // main loop of the debugger session:
        // debugger_session.process_messages().expect("Debugger error");
        debugger_session
            .read_and_process_next_message(&isolate_scope)
            .expect("Debugger error");
        fake_client
            .join()
            .expect("Couldn't join the fake client thread.");
        // Get the result:
        let res_utf8 = res.to_utf8().unwrap();
        assert_eq!(res_utf8.as_str(), "2");
    }

    /// Tests that there is no timeout waiting for the connection, if
    /// the client connection is attempted.
    /// doesn't happen within the provided time limit. time limit.
    #[test]
    fn connection_accept_doesnt_timeout() {
        // The remote debugging server port for the [WebSocketServer].
        let port = PORT_GENERATOR.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        // The remote debugging server ip address for the [WebSocketServer].
        const IP_V4: std::net::Ipv4Addr = std::net::Ipv4Addr::LOCALHOST;
        // The full remote debugging server host name for the [WebSocketServer].
        let host: std::net::SocketAddrV4 = std::net::SocketAddrV4::new(IP_V4, port);

        let address = host.to_string();
        let address = &address;

        // Let's create a server and start listening for the connections
        // on the address provided, but not accepting those yet.
        let mut server = TcpServer::new(address).expect("Couldn't create a tcp server");

        let time_limit = std::time::Duration::from_millis(5000);
        let mut current_waiting_time = std::time::Duration::ZERO;

        let address = address.clone();

        let wait = Arc::new(Mutex::new(()));

        let wait_client = wait.clone();
        // The client thread, attempting to connect.
        let client_thread = std::thread::spawn(move || {
            let _web_socket = 'connect: loop {
                match tungstenite::connect(format!("ws://{address}")) {
                    Ok(ws) => break 'connect ws,
                    Err(_) => continue,
                }
            };

            let _lock = wait_client.lock().unwrap();
        });

        // Now let's wait for the user to connect.
        {
            let _lock = wait.lock();
            let _web_socket = 'accept_loop: loop {
                let start_accepting_time = std::time::Instant::now();

                match server.try_accept_next_websocket_connection() {
                    Ok(connection) => break 'accept_loop connection,
                    Err((s, e)) => {
                        if let Some(raw_error) = e.raw_os_error() {
                            // EWOULDBLOCK / EAGAIN
                            assert_eq!(raw_error, 11, "{e:#?}");
                        }
                        assert_eq!(e.kind(), std::io::ErrorKind::WouldBlock);
                        server = s;
                        current_waiting_time += start_accepting_time.elapsed();

                        if current_waiting_time >= time_limit {
                            unreachable!("The connection is accepted.")
                        }
                    }
                }
            };
        }

        client_thread.join().expect("Thread joined")
    }

    /// Tests that there is a timeout waiting for the connection, if it
    /// doesn't happen within the provided
    #[test]
    fn connection_accept_timesout() {
        // The remote debugging server port for the [WebSocketServer].
        let port = PORT_GENERATOR.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        // The remote debugging server ip address for the [WebSocketServer].
        const IP_V4: std::net::Ipv4Addr = std::net::Ipv4Addr::LOCALHOST;
        // The full remote debugging server host name for the [WebSocketServer].
        let host: std::net::SocketAddrV4 = std::net::SocketAddrV4::new(IP_V4, port);

        let address = host.to_string();
        let address = &address;

        // Let's create a server and start listening for the connections
        // on the address provided, but not accepting those yet.
        let mut server = TcpServer::new(address).expect("Couldn't create a tcp server");

        let time_limit = std::time::Duration::from_millis(1000);
        let mut current_waiting_time = std::time::Duration::ZERO;

        // Now let's wait for the user to connect.
        let _web_socket = 'accept_loop: loop {
            let start_accepting_time = std::time::Instant::now();

            match server.try_accept_next_websocket_connection() {
                Ok(connection) => break 'accept_loop connection,
                Err((s, e)) => {
                    assert_eq!(e.kind(), std::io::ErrorKind::WouldBlock);
                    server = s;
                    current_waiting_time += start_accepting_time.elapsed();

                    if current_waiting_time >= time_limit {
                        return;
                    }
                }
            }
        };

        unreachable!("The connection is never accepted.");
    }
}
