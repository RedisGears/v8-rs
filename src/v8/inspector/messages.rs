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

use serde_aux::prelude::*;

#[derive(Debug, Clone, Deserialize)]
pub struct ScriptParsed {
    /// The ID of the script parsed within the current context.
    #[serde(rename = "scriptId")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: u64,
    /// The URL path to the script. If empty, the script was parsed not
    /// from any path but from a string instead.
    pub url: String,
    #[serde(rename = "startLine")]
    pub start_line: u64,
    #[serde(rename = "startColumn")]
    pub start_column: u64,
    #[serde(rename = "endLine")]
    pub end_line: u64,
    #[serde(rename = "endColumn")]
    pub end_column: u64,
    #[serde(rename = "executionContextId")]
    pub execution_context_id: u64,
    pub hash: String,
    #[serde(rename = "isLiveEdit")]
    pub is_live_edit: bool,
    /// A string containing the source map URL. Sometimes encoded in
    /// `base64`.
    #[serde(rename = "sourceMapURL")]
    pub source_map_url: String,
    /// [`true`] when the script parsed has a single URL path to its
    /// source.
    #[serde(rename = "hasSourceURL")]
    pub has_source_url: bool,
    /// [`true`] if the script parsed is a module.
    #[serde(rename = "isModule")]
    pub is_module: bool,
    /// The length of the script in bytes.
    pub length: u64,
    /// The script language.
    #[serde(rename = "scriptLanguage")]
    pub script_language: String,
    /// The embedder name.
    #[serde(rename = "embedderName")]
    pub embedder_name: String,
}

/// A method invocation message.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MethodCallInformation {
    /// The name of the method.
    #[serde(rename = "method")]
    pub name: String,
    /// The parameters to pass to the method.
    #[serde(rename = "params")]
    pub arguments: serde_json::Map<String, serde_json::Value>,
}

impl MethodCallInformation {
    /// Returns [`true`] if the method is about a script having been
    /// parsed.
    pub fn is_script_parsed(&self) -> bool {
        self.arguments.contains_key("Debugger.scriptParsed")
    }

    /// Returns the [`ScriptParsed`] object when a script is parsed
    /// by the inspector. The event is usually queued by the inspector
    /// to be sent to a client after the compilation of a script has
    /// been done.
    pub fn get_script_parsed(&self) -> Option<ScriptParsed> {
        serde_json::from_value(serde_json::Value::Object(self.arguments.clone())).ok()
    }
}

/// A message from the debugger front-end (from the client to the
/// server).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClientMessage {
    /// The ID of the message.
    pub id: u64,
    /// The method information.
    #[serde(flatten)]
    pub method: MethodCallInformation,
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
            method: MethodCallInformation {
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
            method: MethodCallInformation {
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
    Invoke(MethodCallInformation),
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

impl ServerMessage {
    /// Returns [`Self::Error`] if it is stored.
    pub fn get_error(&self) -> Option<&ErrorMessage> {
        match self {
            Self::Error { error } => Some(error),
            _ => None,
        }
    }

    /// Returns [`Self::Invoke`] if it is stored.
    pub fn get_invocation(&self) -> Option<&MethodCallInformation> {
        match self {
            Self::Invoke(invocation) => Some(invocation),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::v8::inspector::messages::ServerMessage;

    #[test]
    fn parse_script_parsed_message() {
        let s = r#"
        {"method":"Debugger.scriptParsed","params":{"scriptId":"3","url":"","startLine":0,"startColumn":0,"endLine":16,"endColumn":1542,"executionContextId":1,"hash":"0f8e927f0c26b758686526c597c164842bed5dc3","isLiveEdit":false,"sourceMapURL":"data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiY29kZS5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbImNvZGUudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsU0FBUyxJQUFJO0lBQ1QsT0FBTyxDQUFDLEdBQUcsQ0FBQyx3QkFBd0IsQ0FBQyxDQUFDO0lBQ3RDLE9BQU8sQ0FBQyxHQUFHLENBQUMsNERBQTRELENBQUMsQ0FBQztJQUUxRSxLQUFLLElBQUksQ0FBQyxHQUFHLENBQUMsRUFBRSxDQUFDLElBQUksQ0FBQyxFQUFFLEVBQUUsQ0FBQyxFQUFFO1FBQ3pCLE9BQU8sQ0FBQyxHQUFHLENBQUksQ0FBQyxXQUFNLENBQUMsV0FBTSxDQUFDLEdBQUcsQ0FBRyxDQUFDLENBQUM7S0FDekM7SUFFRCxJQUFNLENBQUMsR0FBRyxDQUFDLENBQUM7SUFDWixJQUFNLENBQUMsR0FBRyxDQUFDLENBQUM7SUFDWixJQUFNLENBQUMsR0FBRyxDQUFDLEdBQUcsQ0FBQyxDQUFDO0lBRWhCLE9BQU8sQ0FBQyxHQUFHLENBQUksQ0FBQyxXQUFNLENBQUMsV0FBTSxDQUFHLENBQUMsQ0FBQztBQUN0QyxDQUFDO0FBRUQsSUFBTSxLQUFLLEdBQUcsVUFBQyxJQUFZO0lBQ3ZCLE9BQU8sQ0FBQyxHQUFHLENBQUMsU0FBTyxJQUFNLENBQUMsQ0FBQztBQUMvQixDQUFDLENBQUM7QUFFRixJQUFJLEVBQUUsQ0FBQztBQUVQLEtBQUssQ0FBQyxLQUFLLENBQUMsQ0FBQyIsInNvdXJjZXNDb250ZW50IjpbImZ1bmN0aW9uIG1haW4oKSB7XG4gICAgY29uc29sZS5sb2coJ0hlbGxvLCBDaHJvbWVEZXZUb29scyEnKTtcbiAgICBjb25zb2xlLmxvZyhgSSBoZWFyZCB5b3UncmUgYW4gYW1hemluZyB0b29sISBJJ20gaGVyZSB0byBwbGF5IHdpdGggeW91IWApO1xuXG4gICAgZm9yIChsZXQgaSA9IDE7IGkgPD0gNTsgKytpKSB7XG4gICAgICAgIGNvbnNvbGUubG9nKGAke2l9ICogJHtpfSA9ICR7aSAqIGl9YCk7XG4gICAgfVxuXG4gICAgY29uc3QgYSA9IDc7XG4gICAgY29uc3QgYiA9IDU7XG4gICAgY29uc3QgYyA9IGEgKiBiO1xuXG4gICAgY29uc29sZS5sb2coYCR7YX0gKiAke2J9ID0gJHtjfWApO1xufVxuXG5jb25zdCBzYXlIaSA9IChuYW1lOiBzdHJpbmcpID0+IHtcbiAgICBjb25zb2xlLmxvZyhgSGksICR7bmFtZX1gKTtcbn07XG5cbm1haW4oKTtcblxuc2F5SGkoJ0ZvbycpOyJdfQ==","hasSourceURL":false,"isModule":false,"length":1957,"scriptLanguage":"JavaScript","embedderName":""}}
        "#;
        let message: ServerMessage = serde_json::from_str(s).unwrap();
        let message = message
            .get_invocation()
            .unwrap()
            .get_script_parsed()
            .unwrap();
        assert_eq!(message.id, 3);
    }
}
