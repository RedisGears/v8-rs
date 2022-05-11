use crate::v8_c_raw::bindings::{v8_Dispose, v8_Initialize};

use std::ptr;

pub mod handler_scope;
pub mod isolate;
pub mod isolate_scope;
pub mod try_catch;
pub mod v8_context;
pub mod v8_context_scope;
pub mod v8_module;
pub mod v8_native_function;
pub mod v8_native_function_template;
pub mod v8_object;
pub mod v8_object_template;
pub mod v8_promise;
pub mod v8_resolver;
pub mod v8_script;
pub mod v8_string;
pub mod v8_utf8;
pub mod v8_value;

/// Initialize the v8, must be called before any other v8 API.
pub fn v8_init() {
    unsafe { v8_Initialize(ptr::null_mut()) }
}

/// Destroy v8, after called it is not allowed to use any v8 API anymore.
pub fn v8_destroy() {
    unsafe { v8_Dispose() }
}
