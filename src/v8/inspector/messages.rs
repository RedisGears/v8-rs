/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! The messages the V8 Inspector, represented by [super::Inspector],
//! sends and receives.
//!
//! See [ClientMessage], [MethodInvocation], [ServerMessage],
//! and [ErrorMessage].
use serde::{Deserialize, Serialize};

/// A method invocation message.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MethodInvocation {
    /// The name of the method.
    #[serde(rename = "method")]
    pub name: String,
    /// The parameters to pass to the method.
    #[serde(rename = "params")]
    pub arguments: serde_json::Map<String, serde_json::Value>,
}

/// A message from the debugger front-end (from the client to the
/// server).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClientMessage {
    /// The ID of the message.
    pub id: u64,
    /// The method information.
    #[serde(flatten)]
    pub method: MethodInvocation,
}

impl ClientMessage {
    /// The V8 method which is invoked when a client has successfully
    /// connected to the [super::Inspector] server and waits for the
    /// debugging session to start.
    const DEBUGGER_SHOULD_START_METHOD_NAME: &str = "Runtime.runIfWaitingForDebugger";

    /// Creates a new client message which says that the remote debugger
    /// (the client) is ready to proceed.
    pub fn new_client_ready(id: u64) -> Self {
        Self {
            id,
            method: MethodInvocation {
                name: Self::DEBUGGER_SHOULD_START_METHOD_NAME.to_owned(),
                ..Default::default()
            },
        }
    }

    /// Creates a new client message which instructs the [Inspector] to
    /// set a breakpoint.
    pub fn new_breakpoint(id: u64, column: u64, line: u64, url: &str) -> Self {
        const SET_BREAKPOINT_METHOD_NAME: &str = "Debugger.setBreakpointByUrl";
        // Example: { id: 1015, method: MethodInvocation { name: "Debugger.setBreakpointByUrl",
        // arguments: {"columnNumber": Number(0), "lineNumber": Number(0),
        // "urlRegex": String("file:\\/\\/\\/home\\/fx\\/workspace\\/RedisGears\\/redisgears_core\\/src\\/lib\\.rs($|\\?)|\\/home\\/fx\\/workspace\\/RedisGears\\/redisgears_core\\/src\\/lib\\.rs($|\\?)")} } }

        let mut arguments = serde_json::Map::new();
        arguments.insert("columnNumber".to_owned(), serde_json::json!(column));
        arguments.insert("lineNumber".to_owned(), serde_json::json!(line));
        arguments.insert("urlRegex".to_owned(), serde_json::json!(url));

        Self {
            id,
            method: MethodInvocation {
                name: SET_BREAKPOINT_METHOD_NAME.to_owned(),
                arguments,
            },
        }
    }

    /// Returns `true` if the message says that the remote debugger
    /// (the client) is ready to proceed.
    pub fn is_client_ready(&self) -> bool {
        self.method.name == Self::DEBUGGER_SHOULD_START_METHOD_NAME
    }
}

/// An error message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    /// The error code.
    pub code: i32,
    /// The error message.
    pub message: String,
}

/// A message from the server to the client (from the back-end to the
/// front-end).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServerMessage {
    /// In case the error occurs on the [super::Inspector] side, a
    /// message of this variant is sent to the client.
    Error {
        /// The object containing the [super::Inspector] error message.
        error: ErrorMessage,
    },
    /// The [super::Inspector] sends such sort of messages when it wants
    /// to execute a remote method.
    Invoke(MethodInvocation),
    /// Such kind of messages are sent from the [super::Inspector] to
    /// the remote client as a result of previous message, identifiable
    /// by the `id` member.
    Result {
        /// The ID of the previous message in chain to which this is
        /// the answer.
        id: u64,
        /// The result of the message processing.
        result: serde_json::Map<String, serde_json::Value>,
    },
}
