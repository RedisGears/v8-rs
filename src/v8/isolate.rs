use crate::v8_c_raw::bindings::{
    v8_isolate,
    v8_NewIsolate,
    v8_FreeIsolate,
    v8_IsolateRaiseException,
    v8_NewString,
    v8_StringToValue,
    v8_NewTryCatch,
    v8_GetCurrentCtxRef,
    v8_IdleNotificationDeadline,
    v8_SetInterrupt,
};

use std::os::raw::{c_char, c_void};

use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::handler_scope::V8HandlersScope;
use crate::v8::v8_value::V8LocalValue;
use crate::v8::try_catch::V8TryCatch;
use crate::v8::v8_context_scope::V8ContextScope;

pub struct V8Isolate {
    pub (crate) inner_isolate: *mut v8_isolate,
}

pub (crate)extern "C" fn interrupt_callback<T:Fn(&V8Isolate)>(isolate: *mut v8_isolate, data: *mut ::std::os::raw::c_void) {
    let func = unsafe{&*(data as *mut T)};
    func(&V8Isolate{inner_isolate: isolate});
}

impl V8Isolate {
    pub fn new() -> V8Isolate {
        let inner_isolate = unsafe{v8_NewIsolate()};
        V8Isolate {
            inner_isolate: inner_isolate,
        }
    }

    pub fn enter(&self) -> V8IsolateScope {
        V8IsolateScope::new(self)
    }

    pub fn new_handlers_scope(&self) -> V8HandlersScope {
        V8HandlersScope::new(self)
    }

    pub fn raise_exception(&self, exception: V8LocalValue) {
        unsafe{v8_IsolateRaiseException(self.inner_isolate, exception.inner_val)};
    }

    pub fn raise_exception_str(&self, msg: &str) {
        let inner_string = unsafe{v8_NewString(self.inner_isolate, msg.as_ptr() as *const c_char, msg.len())};
        let inner_val = unsafe{v8_StringToValue(inner_string)};
        unsafe{v8_IsolateRaiseException(self.inner_isolate, inner_val)};
    }

    pub fn new_try_catch(&self) ->V8TryCatch {
        let inner_trycatch = unsafe{v8_NewTryCatch(self.inner_isolate)};
        V8TryCatch {
            inner_trycatch: inner_trycatch,
        }
    }

    pub fn get_curr_context_scope(&self) -> V8ContextScope {
        let inner_ctx_ref = unsafe{v8_GetCurrentCtxRef(self.inner_isolate)};
        V8ContextScope {
            inner_ctx_ref: inner_ctx_ref,
            exit_on_drop: false,
        }
    }

    pub fn idle_notification_deadline(&self) {
        unsafe{v8_IdleNotificationDeadline(self.inner_isolate, 1.0)};
    }

    pub fn set_interrupt<T:Fn(&V8Isolate)>(&self, callback: T) {
        unsafe{v8_SetInterrupt(self.inner_isolate, Some(interrupt_callback::<T>), Box::into_raw(Box::new(callback)) as *mut c_void)};
    }

    pub fn free_isolate(&self) {
        unsafe {v8_FreeIsolate(self.inner_isolate)}
    }
}
