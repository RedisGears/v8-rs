use crate::v8_c_raw::bindings::{
    v8_FreePromise, v8_PromiseGetResult, v8_PromiseGetState,
    v8_PromiseState_v8_PromiseState_Fulfilled, v8_PromiseState_v8_PromiseState_Pending,
    v8_PromiseState_v8_PromiseState_Rejected, v8_PromiseThen, v8_PromiseToValue, v8_local_promise,
};

use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_native_function::V8LocalNativeFunction;
use crate::v8::v8_value::V8LocalValue;

pub struct V8LocalPromise {
    pub(crate) inner_promise: *mut v8_local_promise,
}

#[derive(Debug, PartialEq)]
pub enum V8PromiseState {
    Fulfilled,
    Rejected,
    Pending,
    Unknown,
}

impl V8LocalPromise {
    /// Set resolve and reject callbacks
    pub fn then(
        &self,
        ctx: &V8ContextScope,
        resolve: &V8LocalNativeFunction,
        reject: &V8LocalNativeFunction,
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
    pub fn state(&self) -> V8PromiseState {
        let inner_state = unsafe { v8_PromiseGetState(self.inner_promise) };
        if inner_state == v8_PromiseState_v8_PromiseState_Fulfilled {
            V8PromiseState::Fulfilled
        } else if inner_state == v8_PromiseState_v8_PromiseState_Rejected {
            V8PromiseState::Rejected
        } else if inner_state == v8_PromiseState_v8_PromiseState_Pending {
            V8PromiseState::Pending
        } else {
            panic!("bad promise state");
        }
    }

    /// Return the result of the promise object.
    /// Only applicable if the promise object was resolved/rejected.
    #[must_use]
    pub fn get_result(&self) -> V8LocalValue {
        let inner_val = unsafe { v8_PromiseGetResult(self.inner_promise) };
        V8LocalValue { inner_val }
    }

    /// Convert the promise object into a generic JS value
    #[must_use]
    pub fn to_value(&self) -> V8LocalValue {
        let inner_val = unsafe { v8_PromiseToValue(self.inner_promise) };
        V8LocalValue { inner_val }
    }
}

impl Drop for V8LocalPromise {
    fn drop(&mut self) {
        unsafe { v8_FreePromise(self.inner_promise) }
    }
}
