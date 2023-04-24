/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! Contains the thread-safe facilities.
//!
//! From [https://v8.github.io/api/head/classv8_1_1Unlocker.html#aa6789fe804cc059d9554c7ae7958c440]:
//!
//! > Multiple threads in V8 are allowed, but only one thread at a time
//! > is allowed to use any given V8 isolate, see the comments in the
//! > Isolate class. The definition of 'using a V8 isolate' includes
//! > accessing handles or holding onto object pointers obtained from V8
//! > handles while in the particular V8 isolate. It is up to the user
//! > of V8 to ensure, perhaps with locking, that this constraint is not
//! > violated. In addition to any other synchronization mechanism that
//! > may be used, the v8::Locker and v8::Unlocker classes must be used
//! > to signal thread switches to V8.
//! >
//! > v8::Locker is a scoped lock object. While it's active, i.e.
//! > between its construction and destruction, the current thread is
//! > allowed to use the locked isolate. V8 guarantees that an isolate
//! > can be locked by at most one thread at any time. In other words,
//! > the scope of a v8::Locker is a critical section.

use crate::v8::isolate_scope::IsolateScope;
use crate::v8_c_raw::bindings::{v8_FreeUnlocker, v8_NewUnlocker, v8_unlocker};

use super::ScopedValue;

/// Releases the threads locked by the `Locker`, for example a
/// long-running callback from V8, to enable other threads to work.
#[derive(Debug, Clone)]
pub struct Unlocker<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_unlocker>,
);

impl<'isolate_scope, 'isolate> Unlocker<'isolate_scope, 'isolate> {
    /// Creates a new [Unlocker] within the provided [IsolateScope].
    pub fn new(isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        let inner_val = unsafe { v8_NewUnlocker(isolate_scope.isolate.inner_isolate) };
        Self(ScopedValue {
            inner_val,
            isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> Drop for Unlocker<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeUnlocker(self.0.inner_val) };
    }
}
