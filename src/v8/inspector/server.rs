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
//! use inspector::server::DebuggerSessionBuilder;
//! use std::sync::{Arc, Mutex};
//!
//! // Initialise the V8 engine:
//! v8_init(1);
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
//! // Enter the created execution context:
//! let ctx_scope = ctx.enter(&i_scope);
//!
//! // Create an inspector.
//! let mut inspector = ctx_scope.new_inspector();
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
//!             ws.write_message(message).expect("Couldn't send the message");
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
//!         let _ = ws.read_message();
//!         drop(stage_1.lock().expect("Couldn't lock the stage 1"));
//!         ws.close(None).expect("Couldn't close the WebSocket");
//!     })
//! };
//!
//! // Create the debugging server on the default host.
//! let debugger_session = DebuggerSessionBuilder::new(&mut inspector, address)
//!     .unwrap()
//!     .build()
//!     .unwrap();
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
//! assert!(debugger_session.process_messages().is_ok());
//!
//! fake_client.join();
//!
//! // Get the result:
//! let res_utf8 = res.to_utf8().unwrap();
//! assert_eq!(res_utf8.as_str(), "2");
//! ```
use std::fmt::Write;
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Duration;

use tungstenite::{Error, Message, WebSocket};

use crate::v8::inspector::messages::ClientMessage;

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

    /// Starts listening for and a new single websocket connection.
    /// Once the connection is accepted, it is returned to the user.
    pub fn accept_next_websocket_connection(&self) -> Result<WebSocketServer, std::io::Error> {
        tungstenite::accept(self.server.accept()?.0)
            .map(WebSocketServer::from)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Returns the ways to connect to the server to establish a new
    /// debugger session.
    pub fn get_connection_hints(&self) -> Option<DebuggerSessionConnectionHints> {
        let websocket_address = self.get_listening_address().ok()?;
        let vscode_configuration = format!(
            r#"
        {{
            "version": "0.2.0",
            "configurations": [
                {{
                    "name": "Attach to RedisGears V8 through WebSocket at {websocket_address}",
                    "type": "node",
                    "request": "attach",
                    "cwd": "${{workspaceFolder}}",
                    "websocketAddress": "ws://{websocket_address}",
                }}
            ]
        }}
        "#
        );
        let chromium_link = format!(
            "devtools://devtools/bundled/inspector.html?experiments=true&v8only=true&ws={websocket_address}"
        );
        Some(DebuggerSessionConnectionHints {
            websocket_address,
            vscode_configuration,
            chromium_link,
        })
    }
}

/// A WebSocket server.
#[repr(transparent)]
#[derive(Debug)]
pub struct WebSocketServer(WebSocket<TcpStream>);
impl WebSocketServer {
    /// The default read timeout duration.
    const DEFAULT_READ_TIMEOUT_DURATION: Duration = Duration::from_millis(100);

    /// Waits for a message available to read, and once there is one,
    /// reads it and returns as a text.
    pub fn read_next_message(&mut self) -> Result<String, std::io::Error> {
        log::trace!("Reading the next message.");
        match self.0.read_message() {
            Ok(message) => message
                .into_text()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)),
            Err(Error::Io(e)) => Err(e),
            Err(Error::ConnectionClosed | Error::AlreadyClosed) => Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                "The WebSocket connection has been closed.",
            )),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
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

struct InspectorCallbacks {
    on_response: Box<OnResponseCallback>,
    on_wait_frontend_message_on_pause: Box<OnWaitFrontendMessageOnPauseCallback>,
}

/// The means of connection to the V8 debugger.
#[derive(Debug, Clone)]
pub struct DebuggerSessionConnectionHints {
    websocket_address: std::net::SocketAddr,
    vscode_configuration: String,
    chromium_link: String,
}

impl std::fmt::Display for DebuggerSessionConnectionHints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let address = &self.websocket_address;
        let vscode_configuration = &self.vscode_configuration;
        let chromium_link = &self.chromium_link;
        f.write_fmt(format_args!("The V8 remote debugging server is waiting for a connection via WebSocket on: {address}.\nHint: you may browse this link: {chromium_link} or use this launch configuration for Visual Studio Code (<https://code.visualstudio.com/>):{vscode_configuration}"))
    }
}

/// The debugger session builder.
#[derive(Debug, Default)]
pub struct DebuggerSessionBuilder<'inspector, 'isolate> {
    tcp_server: Option<TcpServer>,
    inspector: Option<&'inspector mut Inspector<'isolate>>,
}
impl<'inspector, 'isolate> DebuggerSessionBuilder<'inspector, 'isolate> {
    /// Creates a new debugger session builder.
    pub fn new<T: std::net::ToSocketAddrs>(
        inspector: &'inspector mut Inspector<'isolate>,
        address: T,
    ) -> Result<Self, std::io::Error> {
        Ok(Self {
            tcp_server: Some(TcpServer::new(address)?),
            inspector: Some(inspector),
        })
    }

    /// Sets a [TcpServer].
    pub fn tcp_server(&mut self, tcp_server: TcpServer) -> &mut Self {
        self.tcp_server = Some(tcp_server);
        self
    }

    /// Sets an [Inspector].
    pub fn inspector(&mut self, inspector: &'inspector mut Inspector<'isolate>) -> &mut Self {
        self.inspector = Some(inspector);
        self
    }

    /// Returns the ways to connect to the server to establish a new
    /// debugger session.
    pub fn get_connection_hints(&self) -> Option<DebuggerSessionConnectionHints> {
        self.tcp_server.as_ref()?.get_connection_hints()
    }

    /// Consumes [self] and attempts to build a [DebuggerSession].
    pub fn build(self) -> Result<DebuggerSession<'inspector, 'isolate>, std::io::Error> {
        let inspector = self.inspector.expect("The V8 Inspector wasn't set");
        let server = self.tcp_server.expect("The TCP server wasn't set");
        DebuggerSession::new(inspector, server)
    }
}

/// A single debugger session.
#[derive(Debug)]
pub struct DebuggerSession<'inspector, 'isolate> {
    web_socket: Rc<Mutex<WebSocketServer>>,
    inspector: &'inspector mut Inspector<'isolate>,
    connection_hints: DebuggerSessionConnectionHints,
}

impl<'inspector, 'isolate> DebuggerSession<'inspector, 'isolate> {
    fn create_inspector_callbacks(web_socket: Rc<Mutex<WebSocketServer>>) -> InspectorCallbacks {
        let websocket = web_socket.clone();

        let on_response = move |s: String| {
            log::trace!("Responding with string {s}.");
            match websocket.lock() {
                Ok(mut websocket) => match websocket.0.write_message(Message::Text(s)) {
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

    /// Creates a new builder.
    pub fn builder() -> DebuggerSessionBuilder<'inspector, 'isolate> {
        DebuggerSessionBuilder::default()
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
    pub fn new(
        inspector: &'inspector mut Inspector<'isolate>,
        server: TcpServer,
    ) -> Result<Self, std::io::Error> {
        let connection_hints = server
            .get_connection_hints()
            .expect("Couldn't get the connection hints");

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
            connection_hints,
        };

        // The re-locking read loop to wait until the remote debugger
        // is ready to start.
        loop {
            let message_string: String = session.read_next_message()?;
            let message: ClientMessage = match serde_json::from_str(&message_string) {
                Ok(message) => message,
                Err(_) => continue,
            };
            log::trace!("Parsed out the incoming message: {message:?}");
            session.inspector.dispatch_protocol_message(&message_string);

            if message.is_client_ready() {
                session
                    .inspector
                    .schedule_pause_on_next_statement("Debugger started.");
                session.inspector.wait_frontend_message_on_pause();

                return Ok(session);
            }
        }
    }

    /// Returns the ways to connect to the server to establish a new
    /// debugger session.
    pub fn get_connection_hints(&self) -> &DebuggerSessionConnectionHints {
        &self.connection_hints
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
    /// See [super::Inspector::schedule_pause_on_next_statement].
    pub fn schedule_pause_on_next_statement(&self) {
        self.inspector
            .schedule_pause_on_next_statement("User breakpoint.");
    }
}

impl<'inspector, 'isolate> Drop for DebuggerSession<'inspector, 'isolate> {
    fn drop(&mut self) {
        self.inspector.reset_on_response_callback();
        self.inspector
            .reset_on_wait_frontend_message_on_pause_callback();
    }
}

#[cfg(test)]
mod tests {
    use crate::v8::isolate::V8Isolate;

    use super::{ClientMessage, DebuggerSessionBuilder};
    use std::sync::{Arc, Mutex};

    /*
    This is to test the crash when setting a breakpoint.

    Chromium inspector:

    32029:M 26 May 2023 10:56:37.928 . <redisgears_2> 'v8_rs::v8::inspector::server' /home/fx/workspace/v8-rs/src/v8/inspector/server.rs:281: [OnWait] Read the message: "{\"id\":20,\"method\":\"Debugger.setBreakpointByUrl\",\"params\":{\"lineNumber\":2,\"scriptHash\":\"e18847f3eb3ba96b6de3f10a3370067120fe0f5dc162f58b08e39df7e5f5308f\",\"columnNumber\":68,\"condition\":\"\"}}"
    */
    #[test]
    fn can_set_breakpoint() {
        // Initialise the V8 engine:
        crate::test_utils::initialize();

        // Create a new isolate:
        let isolate = V8Isolate::new();

        // Enter the isolate created:
        let i_scope = isolate.enter();

        // Create the code string object:
        let code_str = i_scope.new_string("1+1");

        // Create a JS execution context for code invocation:""
        let ctx = i_scope.new_context(None);

        // Enter the created execution context:
        let ctx_scope = ctx.enter(&i_scope);

        // Create an inspector.
        let mut inspector = ctx_scope.new_inspector();

        let stage_1 = Arc::new(Mutex::new(()));

        let lock_1 = stage_1.lock().unwrap();

        // The remote debugging server port for the [WebSocketServer].
        const PORT_V4: u16 = 9005;
        // The remote debugging server ip address for the [WebSocketServer].
        const IP_V4: std::net::Ipv4Addr = std::net::Ipv4Addr::LOCALHOST;
        // The full remote debugging server host name for the [WebSocketServer].
        const LOCAL_HOST: std::net::SocketAddrV4 = std::net::SocketAddrV4::new(IP_V4, PORT_V4);

        let address = LOCAL_HOST.to_string();
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
                    ws.write_message(message)
                        .expect("Couldn't send the message");
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
                let _ = ws.read_message();
                client.send_breakpoint(&mut ws, 0, 0, "
                file:///home/fx/workspace/RedisGears/redisgears_core/src/lib.rs($|?)|/home/fx/workspace/RedisGears/redisgears_core/src/lib.rs($|?)");
                drop(stage_1.lock().expect("Couldn't lock the stage 1"));
                ws.close(None).expect("Couldn't close the WebSocket");
            })
        };
        // Create the debugging server on the default host.
        let debugger_session = DebuggerSessionBuilder::new(&mut inspector, address)
            .unwrap()
            .build()
            .unwrap();
        // At this point, the server is running and has accepted a remote
        // client. Once the client connects, the debugger pauses on the very
        // first (next) instruction, so we can safely attempt to run the
        // script, as it won't actually run but will wait for remote client
        // to act.
        // Compile the code:
        let script = ctx_scope.compile(&code_str).unwrap();
        // Allow the fake client to stop (for this test not to hang).
        drop(lock_1);
        // Run the compiled code:
        let res = script.run(&ctx_scope).unwrap();
        // To let the remote debugger operate, we need to be able to send
        // and receive data to and from it. This is achieved by starting the
        // main loop of the debugger session:
        // debugger_session.process_messages().expect("Debugger error");
        debugger_session
            .read_and_process_next_message()
            .expect("Debugger error");
        fake_client
            .join()
            .expect("Couldn't join the fake client thread.");
        // Get the result:
        let res_utf8 = res.to_utf8().unwrap();
        assert_eq!(res_utf8.as_str(), "2");
    }
}
