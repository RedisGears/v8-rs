use crate::v8_c_raw::bindings::{
    v8_Initialize,
    v8_Despose,
};

use std::ptr;

pub mod isolate;
pub mod isolate_scope;
pub mod handler_scope;
pub mod v8_string;
pub mod v8_object_template;
pub mod v8_native_function_template;
pub mod v8_native_function;
pub mod v8_context;
pub mod v8_context_scope;
pub mod v8_script;
pub mod v8_value;
pub mod v8_utf8;
pub mod try_catch;
pub mod v8_promise;
pub mod v8_resolver;

pub fn v8_init() {
    unsafe {v8_Initialize(ptr::null_mut())}
}

pub fn v8_destroy() {
    unsafe {v8_Despose()}
}