/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{v8_Dispose, v8_Initialize, v8_Version};

use std::ffi::CStr;
use std::ptr;

pub mod inspector;
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

pub(crate) type FatalErrorCallback = dyn Fn(&str, &str);
pub(crate) type OutOfMemoryErrorCallback = dyn Fn(&str, bool);
pub(crate) static mut FATAL_ERROR_CALLBACK: Option<Box<FatalErrorCallback>> = None;
pub(crate) static mut OOM_ERROR_CALLBACK: Option<Box<OutOfMemoryErrorCallback>> = None;

pub trait OptionalTryFrom<T>: Sized {
    type Error;

    fn optional_try_from(value: T) -> Result<Option<Self>, Self::Error>;
}

/// Initialize the v8, must be called before any other v8 API.
pub fn v8_init(thread_pool_size: i32) {
    unsafe { v8_Initialize(ptr::null_mut(), thread_pool_size) }
}

/// Initialise the V8 engine with custom fatal error and OOM handlers
/// as well as with the custom thread pool size.
pub fn v8_init_with_error_handlers(
    fatal_error_handler: Box<FatalErrorCallback>,
    oom_error_handler: Box<OutOfMemoryErrorCallback>,
    thread_pool_size: i32,
) {
    v8_init(thread_pool_size);
    unsafe {
        FATAL_ERROR_CALLBACK = Some(fatal_error_handler);
        OOM_ERROR_CALLBACK = Some(oom_error_handler);
    }
}

/// Destroys v8, after calling it is not allowed to use any v8 API anymore.
pub fn v8_destroy() {
    unsafe { v8_Dispose() }
}

/// Returns the version of V8 as as string.
pub fn v8_version() -> &'static str {
    let s = unsafe { CStr::from_ptr(v8_Version()) };
    s.to_str().unwrap()
}
