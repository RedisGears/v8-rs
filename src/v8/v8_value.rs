use crate::v8_c_raw::bindings::{
    v8_FreePersistedValue, v8_FreeValue, v8_FunctionCall, v8_GetBigInt, v8_GetBool, v8_GetNumber,
    v8_PersistValue, v8_PersistedValueToLocal, v8_ToUtf8, v8_ValueAsArray, v8_ValueAsObject,
    v8_ValueAsPromise, v8_ValueAsResolver, v8_ValueAsString, v8_ValueIsArray,
    v8_ValueIsAsyncFunction, v8_ValueIsBigInt, v8_ValueIsBool, v8_ValueIsFunction,
    v8_ValueIsNumber, v8_ValueIsObject, v8_ValueIsPromise, v8_ValueIsString, v8_local_value,
    v8_persisted_value,
};

use std::ptr;

use crate::v8::isolate::V8Isolate;
use crate::v8::v8_array::V8LocalArray;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_object::V8LocalObject;
use crate::v8::v8_promise::V8LocalPromise;
use crate::v8::v8_resolver::V8LocalResolver;
use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_utf8::V8LocalUtf8;

/// JS generic local value
pub struct V8LocalValue {
    pub(crate) inner_val: *mut v8_local_value,
}

/// JS generic persisted value
pub struct V8PersistValue {
    pub(crate) inner_val: *mut v8_persisted_value,
}

impl V8LocalValue {
    /// Return string representation of the value or None on failure
    #[must_use]
    pub fn to_utf8(&self, isolate: &V8Isolate) -> Option<V8LocalUtf8> {
        let inner_val = unsafe { v8_ToUtf8(isolate.inner_isolate, self.inner_val) };
        if inner_val.is_null() {
            None
        } else {
            Some(V8LocalUtf8 { inner_val })
        }
    }

    /// Return true if the value is string and false otherwise.
    #[must_use]
    pub fn is_string(&self) -> bool {
        (unsafe { v8_ValueIsString(self.inner_val) } != 0)
    }

    /// Convert the object into a string, applicable only if the value is string.
    #[must_use]
    pub fn as_string(&self) -> V8LocalString {
        let inner_str = unsafe { v8_ValueAsString(self.inner_val) };
        V8LocalString {
            inner_string: inner_str,
        }
    }

    /// Return true if the value is string and false otherwise.
    #[must_use]
    pub fn is_array(&self) -> bool {
        (unsafe { v8_ValueIsArray(self.inner_val) } != 0)
    }

    /// Convert the object into a string, applicable only if the value is string.
    #[must_use]
    pub fn as_array(&self) -> V8LocalArray {
        let inner_array = unsafe { v8_ValueAsArray(self.inner_val) };
        V8LocalArray { inner_array }
    }

    /// Return true if the value is function and false otherwise.
    #[must_use]
    pub fn is_function(&self) -> bool {
        (unsafe { v8_ValueIsFunction(self.inner_val) } != 0)
    }

    /// Return true if the value is async function and false otherwise.
    #[must_use]
    pub fn is_async_function(&self) -> bool {
        (unsafe { v8_ValueIsAsyncFunction(self.inner_val) } != 0)
    }

    /// Return true if the value is number and false otherwise.
    #[must_use]
    pub fn is_number(&self) -> bool {
        (unsafe { v8_ValueIsNumber(self.inner_val) } != 0)
    }

    pub fn get_number(&self) -> f64 {
        unsafe { v8_GetNumber(self.inner_val) }
    }

    /// Return true if the value is number and false otherwise.
    #[must_use]
    pub fn is_long(&self) -> bool {
        (unsafe { v8_ValueIsBigInt(self.inner_val) } != 0)
    }

    pub fn get_long(&self) -> i64 {
        unsafe { v8_GetBigInt(self.inner_val) }
    }

    /// Return true if the value is boolean and false otherwise.
    #[must_use]
    pub fn is_boolean(&self) -> bool {
        (unsafe { v8_ValueIsBool(self.inner_val) } != 0)
    }

    pub fn get_boolean(&self) -> bool {
        if unsafe { v8_GetBool(self.inner_val) } == 0 {
            false
        } else {
            true
        }
    }

    /// Return true if the value is promise and false otherwise.
    #[must_use]
    pub fn is_promise(&self) -> bool {
        (unsafe { v8_ValueIsPromise(self.inner_val) } != 0)
    }

    /// Convert the object into a promise, applicable only if the object is promise.
    #[must_use]
    pub fn as_promise(&self) -> V8LocalPromise {
        let inner_promise = unsafe { v8_ValueAsPromise(self.inner_val) };
        V8LocalPromise { inner_promise }
    }

    /// Convert the object into a resolver, applicable only if the object is resolver.
    #[must_use]
    pub fn as_resolver(&self) -> V8LocalResolver {
        let inner_resolver = unsafe { v8_ValueAsResolver(self.inner_val) };
        V8LocalResolver { inner_resolver }
    }

    /// Return true if the value is object and false otherwise.
    #[must_use]
    pub fn is_object(&self) -> bool {
        (unsafe { v8_ValueIsObject(self.inner_val) } != 0)
    }

    /// Convert the object into a promise, applicable only if the object is promise.
    #[must_use]
    pub fn as_object(&self) -> V8LocalObject {
        let inner_obj = unsafe { v8_ValueAsObject(self.inner_val) };
        V8LocalObject { inner_obj }
    }

    /// Persist the local object so it can be saved beyond the current handlers scope.
    #[must_use]
    pub fn persist(&self, isolate: &V8Isolate) -> V8PersistValue {
        let inner_val = unsafe { v8_PersistValue(isolate.inner_isolate, self.inner_val) };
        V8PersistValue { inner_val }
    }

    /// Run the value, applicable only if the value is a function or async function.
    #[must_use]
    pub fn call(&self, ctx: &V8ContextScope, args: Option<&[&Self]>) -> Option<Self> {
        let res = match args {
            Some(args) => {
                let args = args
                    .iter()
                    .map(|v| v.inner_val)
                    .collect::<Vec<*mut v8_local_value>>();
                let ptr = args.as_ptr();
                unsafe { v8_FunctionCall(ctx.inner_ctx_ref, self.inner_val, args.len(), ptr) }
            }
            None => unsafe { v8_FunctionCall(ctx.inner_ctx_ref, self.inner_val, 0, ptr::null()) },
        };

        if res.is_null() {
            None
        } else {
            Some(Self { inner_val: res })
        }
    }
}

impl V8PersistValue {
    /// Convert the persisted value back to local value.
    #[must_use]
    pub fn as_local(&self, isolate: &V8Isolate) -> V8LocalValue {
        let inner_val = unsafe { v8_PersistedValueToLocal(isolate.inner_isolate, self.inner_val) };
        V8LocalValue { inner_val }
    }
}

unsafe impl Sync for V8PersistValue {}
unsafe impl Send for V8PersistValue {}

impl Drop for V8LocalValue {
    fn drop(&mut self) {
        if !self.inner_val.is_null() {
            unsafe { v8_FreeValue(self.inner_val) }
        }
    }
}

impl Drop for V8PersistValue {
    fn drop(&mut self) {
        unsafe { v8_FreePersistedValue(self.inner_val) }
    }
}
