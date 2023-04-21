/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_ExternalDataGet, v8_ExternalDataToValue, v8_NewExternalData, v8_local_external_data,
};

use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::ScopedValue;

use super::any::LocalValueAny;

extern "C" fn free_external_data<T>(arg1: *mut ::std::os::raw::c_void) {
    unsafe { Box::from_raw(arg1 as *mut T) };
}

/// TODO a proper comment.
#[derive(Debug, Clone)]
pub struct LocalExternalData<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_local_external_data>,
);

impl<'isolate_scope, 'isolate> LocalExternalData<'isolate_scope, 'isolate> {
    /// Creates a new local external data object within the passed [IsolateScope].
    pub fn new<T>(data: T, isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        let data = Box::into_raw(Box::new(data));
        let inner_val = unsafe {
            v8_NewExternalData(
                isolate_scope.isolate.inner_isolate,
                data as *mut _,
                Some(free_external_data::<T>),
            )
        };
        Self(ScopedValue {
            inner_val,
            isolate_scope,
        })
    }

    pub fn get_data<T>(&self) -> &'isolate_scope T {
        unsafe { &*(v8_ExternalDataGet(self.0.inner_val) as *const T) }
    }

    pub fn get_data_mut<T>(&mut self) -> &mut T {
        unsafe { &mut *(v8_ExternalDataGet(self.0.inner_val) as *mut T) }
    }
}

impl<'isolate_scope, 'isolate> From<LocalExternalData<'isolate_scope, 'isolate>>
    for LocalValueAny<'isolate_scope, 'isolate>
{
    fn from(value: LocalExternalData<'isolate_scope, 'isolate>) -> Self {
        let inner_val = unsafe { v8_ExternalDataToValue(value.0.inner_val) };
        LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope: value.0.isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueAny<'isolate_scope, 'isolate>>
    for LocalExternalData<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: LocalValueAny<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_external_data() {
            return Err("Value is not a string");
        }

        Ok(unsafe { val.as_external_data() })
    }
}
