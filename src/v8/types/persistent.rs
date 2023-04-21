/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! This is an abstraction over a persistent (non-scoped) V8 value.
//! As this is a fully abstract class in V8, the objects it can hold
//! can be of any javascript type. Hence it is generic.

use crate::{
    v8::isolate_scope::IsolateScope,
    v8_c_raw::bindings::{v8_FreePersistedValue, v8_PersistedValueToLocal, v8_persisted_value},
};

use super::{any::LocalValueAny, ScopedValue};

/// JS generic persisted value
pub struct PersistValue {
    pub(crate) inner_val: *mut v8_persisted_value,
    forget: bool,
}

impl PersistValue {
    /// Converts the persisted value back to local value.
    ///
    /// # Panics
    ///
    /// Panics when the inner value is a null pointer.
    #[must_use]
    pub fn as_local<'isolate, 'isolate_scope>(
        &self,
        isolate_scope: &'isolate_scope IsolateScope<'isolate>,
    ) -> LocalValueAny<'isolate_scope, 'isolate> {
        assert!(!self.inner_val.is_null());
        let inner_val = unsafe {
            v8_PersistedValueToLocal(isolate_scope.isolate.inner_isolate, self.inner_val)
        };
        LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope,
        })
    }

    /// Disables the [Drop] implementation for this object, marking it
    /// as a non-requiring to be dropped.
    ///
    /// # Panics
    ///
    /// Panics when the inner value is a null pointer.
    pub fn forget(&mut self) {
        assert!(!self.inner_val.is_null());
        self.forget = true;
    }

    /// Consumes the current persistent value and attempts to convert it
    /// into a local value. The object isn't dropped.
    ///
    /// # Panics
    ///
    /// Panics when the inner value is a null pointer, see [Self::as_local].
    pub fn take_local<'isolate, 'isolate_scope>(
        mut self,
        isolate_scope: &'isolate_scope IsolateScope<'isolate>,
    ) -> LocalValueAny<'isolate_scope, 'isolate> {
        let val = self.as_local(isolate_scope);
        unsafe { v8_FreePersistedValue(self.inner_val) }
        self.forget();
        self.inner_val = std::ptr::null_mut();
        val
    }
}

impl From<*mut v8_persisted_value> for PersistValue {
    fn from(value: *mut v8_persisted_value) -> Self {
        Self {
            inner_val: value,
            forget: false,
        }
    }
}

unsafe impl Sync for PersistValue {}
unsafe impl Send for PersistValue {}

impl Drop for PersistValue {
    fn drop(&mut self) {
        if self.forget {
            return;
        }
        unsafe { v8_FreePersistedValue(self.inner_val) }
    }
}
