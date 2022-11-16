/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{v8_Dispose, v8_Initialize, v8_Version};

use std::ffi::CStr;
use std::ptr;

pub mod isolate;
pub mod isolate_scope;
pub mod try_catch;
pub mod v8_array;
pub mod v8_array_buffer;
pub mod v8_context;
pub mod v8_context_scope;
pub mod v8_external_data;
pub mod v8_module;
pub mod v8_native_function;
pub mod v8_native_function_template;
pub mod v8_object;
pub mod v8_object_template;
pub mod v8_promise;
pub mod v8_resolver;
pub mod v8_script;
pub mod v8_set;
pub mod v8_string;
pub mod v8_unlocker;
pub mod v8_utf8;
pub mod v8_value;

pub(crate) static mut FATAL_ERROR_CALLBACK: Option<Box<dyn Fn(&str, &str)>> = None;
pub(crate) static mut OOM_ERROR_CALLBACK: Option<Box<dyn Fn(&str, bool)>> = None;

/// Initialize the v8, must be called before any other v8 API.
pub fn v8_init() {
    unsafe { v8_Initialize(ptr::null_mut()) }
}

pub fn v8_init_with_error_handlers(
    fatal_error_hanlder: Box<dyn Fn(&str, &str)>,
    oom_error_handler: Box<dyn Fn(&str, bool)>,
) {
    v8_init();
    unsafe {
        FATAL_ERROR_CALLBACK = Some(fatal_error_hanlder);
        OOM_ERROR_CALLBACK = Some(oom_error_handler);
    }
}

/// Destroy v8, after called it is not allowed to use any v8 API anymore.
pub fn v8_destroy() {
    unsafe { v8_Dispose() }
}

pub fn v8_version() -> &'static str {
    let s = unsafe { CStr::from_ptr(v8_Version()) };
    s.to_str().unwrap()
}
