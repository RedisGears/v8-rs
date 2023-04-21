/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! Contains the JavaScript promise facilities.

use crate::v8_c_raw::bindings::{
    v8_FreePromise, v8_PromiseGetResult, v8_PromiseGetState,
    v8_PromiseState_v8_PromiseState_Fulfilled, v8_PromiseState_v8_PromiseState_Pending,
    v8_PromiseState_v8_PromiseState_Rejected, v8_PromiseThen, v8_PromiseToValue, v8_local_promise,
};

use crate::v8::context_scope::ContextScope;
use crate::v8::types::native_function::LocalNativeFunction;
use crate::v8::types::ScopedValue;

use super::any::LocalValueAny;

/// A promise is an object representing an eventual completion
/// (successful or not) of an asynchronous operation and its resulting
/// value.
pub struct LocalPromise<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_local_promise>,
);

/// The states a [LocalPromise] can be in.
#[derive(Debug, PartialEq)]
pub enum PromiseState {
    /// The operation has completed successfully.
    Fulfilled,
    /// The operation has failed.
    Rejected,
    /// The initial state of a promise - pending execution.
    Pending,
}

impl<'isolate_scope, 'isolate> LocalPromise<'isolate_scope, 'isolate> {
    /// Set resolve and reject callbacks
    pub fn then(
        &self,
        ctx: &ContextScope,
        resolve: &LocalNativeFunction,
        reject: &LocalNativeFunction,
    ) {
        unsafe {
            v8_PromiseThen(
                self.0.inner_val,
                ctx.inner_ctx_ref,
                resolve.0.inner_val,
                reject.0.inner_val,
            );
        };
    }

    /// Return the state on the promise object.
    #[allow(non_upper_case_globals)]
    pub fn state(&self) -> Option<PromiseState> {
        let inner_state = unsafe { v8_PromiseGetState(self.0.inner_val) };
        Some(match inner_state {
            v8_PromiseState_v8_PromiseState_Fulfilled => PromiseState::Fulfilled,
            v8_PromiseState_v8_PromiseState_Rejected => PromiseState::Rejected,
            v8_PromiseState_v8_PromiseState_Pending => PromiseState::Pending,
            _ => return None,
        })
    }

    /// Return the result of the promise object.
    /// Only applicable if the promise object was resolved/rejected.
    pub fn get_result(&self) -> LocalValueAny<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_PromiseGetResult(self.0.inner_val) };
        LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope: self.0.isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalPromise<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreePromise(self.0.inner_val) }
    }
}

impl<'isolate_scope, 'isolate> From<LocalPromise<'isolate_scope, 'isolate>>
    for LocalValueAny<'isolate_scope, 'isolate>
{
    fn from(value: LocalPromise<'isolate_scope, 'isolate>) -> Self {
        let inner_val = unsafe { v8_PromiseToValue(value.0.inner_val) };
        Self(ScopedValue {
            inner_val,
            isolate_scope: value.0.isolate_scope,
        })
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueAny<'isolate_scope, 'isolate>>
    for LocalPromise<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(value: LocalValueAny<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if value.is_promise() {
            Ok(unsafe { value.as_promise() })
        } else {
            Err("The value is not a promise.")
        }
    }
}
