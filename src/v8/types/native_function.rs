/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_FreeNativeFunction, v8_NativeFunctionToValue, v8_local_native_function,
};

use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::LocalValueGeneric;

/// Native function object
pub struct LocalNativeFunction<'isolate_scope, 'isolate> {
    pub(crate) inner_func: *mut v8_local_native_function,
    pub(crate) isolate_scope: &'isolate_scope IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> LocalNativeFunction<'isolate_scope, 'isolate> {
    /// Convert the native function into a JS generic value
    #[must_use]
    pub fn to_value(&self) -> LocalValueGeneric<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_NativeFunctionToValue(self.inner_func) };
        LocalValueGeneric {
            inner_val,
            isolate_scope: self.isolate_scope,
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalNativeFunction<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeNativeFunction(self.inner_func) }
    }
}
