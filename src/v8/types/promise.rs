/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_FreePromise, v8_PromiseGetResult, v8_PromiseGetState,
    v8_PromiseState_v8_PromiseState_Fulfilled, v8_PromiseState_v8_PromiseState_Pending,
    v8_PromiseState_v8_PromiseState_Rejected, v8_PromiseThen, v8_PromiseToValue, v8_local_promise,
};

use crate::v8::context_scope::ContextScope;
use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::native_function::LocalNativeFunction;
use crate::v8::types::LocalValueGeneric;

pub struct LocalPromise<'isolate_scope, 'isolate> {
    pub(crate) inner_promise: *mut v8_local_promise,
    pub(crate) isolate_scope: &'isolate_scope IsolateScope<'isolate>,
}

#[derive(Debug, PartialEq)]
pub enum PromiseState {
    Fulfilled,
    Rejected,
    Pending,
    Unknown,
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
                self.inner_promise,
                ctx.inner_ctx_ref,
                resolve.inner_func,
                reject.inner_func,
            );
        };
    }

    /// Return the state on the promise object
    /// # Panics
    #[must_use]
    pub fn state(&self) -> PromiseState {
        let inner_state = unsafe { v8_PromiseGetState(self.inner_promise) };
        if inner_state == v8_PromiseState_v8_PromiseState_Fulfilled {
            PromiseState::Fulfilled
        } else if inner_state == v8_PromiseState_v8_PromiseState_Rejected {
            PromiseState::Rejected
        } else if inner_state == v8_PromiseState_v8_PromiseState_Pending {
            PromiseState::Pending
        } else {
            panic!("bad promise state");
        }
    }

    /// Return the result of the promise object.
    /// Only applicable if the promise object was resolved/rejected.
    #[must_use]
    pub fn get_result(&self) -> LocalValueGeneric<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_PromiseGetResult(self.inner_promise) };
        LocalValueGeneric {
            inner_val,
            isolate_scope: self.isolate_scope,
        }
    }

    /// Convert the promise object into a generic JS value
    #[must_use]
    pub fn to_value(&self) -> LocalValueGeneric<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_PromiseToValue(self.inner_promise) };
        LocalValueGeneric {
            inner_val,
            isolate_scope: self.isolate_scope,
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalPromise<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreePromise(self.inner_promise) }
    }
}
