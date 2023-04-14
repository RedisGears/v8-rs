/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_ExternalDataGet, v8_ExternalDataToValue, v8_local_external_data,
};

use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::LocalValueGeneric;

/// JS object
pub struct LocalExternalData<'isolate_scope, 'isolate> {
    pub(crate) inner_ext: *mut v8_local_external_data,
    pub(crate) isolate_scope: &'isolate_scope IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> LocalExternalData<'isolate_scope, 'isolate> {
    /// Convert the object into a generic JS value
    #[must_use]
    pub fn to_value(&self) -> LocalValueGeneric<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ExternalDataToValue(self.inner_ext) };
        LocalValueGeneric {
            inner_val,
            isolate_scope: self.isolate_scope,
        }
    }

    pub fn get_data<T>(&self) -> &'isolate_scope T {
        unsafe { &*(v8_ExternalDataGet(self.inner_ext) as *const T) }
    }

    pub fn get_data_mut<T>(&mut self) -> &mut T {
        unsafe { &mut *(v8_ExternalDataGet(self.inner_ext) as *mut T) }
    }
}
