/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_FreeNativeFunction, v8_NativeFunctionToValue, v8_local_native_function,
};

use crate::v8::types::ScopedValue;

use super::any::LocalValueAny;
use super::Value;

/// Native function object
pub struct LocalNativeFunction<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_local_native_function>,
);

impl<'isolate_scope, 'isolate> Drop for LocalNativeFunction<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeNativeFunction(self.0.inner_val) }
    }
}

impl<'isolate_scope, 'isolate> From<LocalNativeFunction<'isolate_scope, 'isolate>>
    for LocalValueAny<'isolate_scope, 'isolate>
{
    fn from(
        value: LocalNativeFunction<'isolate_scope, 'isolate>,
    ) -> LocalValueAny<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_NativeFunctionToValue(value.0.inner_val) };
        LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope: value.0.isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> From<LocalNativeFunction<'isolate_scope, 'isolate>>
    for Value<'isolate_scope, 'isolate>
{
    fn from(
        value: LocalNativeFunction<'isolate_scope, 'isolate>,
    ) -> Value<'isolate_scope, 'isolate> {
        LocalValueAny::from(value).into()
    }
}
