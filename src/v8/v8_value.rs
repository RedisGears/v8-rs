/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_FreePersistedValue, v8_FreeValue, v8_FunctionCall, v8_GetBigInt, v8_GetBool, v8_GetNumber,
    v8_PersistValue, v8_PersistedValueToLocal, v8_ToUtf8, v8_ValueAsArray, v8_ValueAsArrayBuffer,
    v8_ValueAsExternalData, v8_ValueAsObject, v8_ValueAsPromise, v8_ValueAsResolver, v8_ValueAsSet,
    v8_ValueAsString, v8_ValueIsArray, v8_ValueIsArrayBuffer, v8_ValueIsAsyncFunction,
    v8_ValueIsBigInt, v8_ValueIsBool, v8_ValueIsExternalData, v8_ValueIsFunction, v8_ValueIsNull,
    v8_ValueIsNumber, v8_ValueIsObject, v8_ValueIsPromise, v8_ValueIsSet, v8_ValueIsString,
    v8_ValueIsStringObject, v8_ValueIsUndefined, v8_ValueToValue, v8_local_value,
    v8_persisted_value,
};

use std::ptr;

use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::v8_array::V8LocalArray;
use crate::v8::v8_array_buffer::V8LocalArrayBuffer;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_external_data::V8LocalExternalData;
use crate::v8::v8_native_function_template::V8LocalNativeFunctionArgsIter;
use crate::v8::v8_object::V8LocalObject;
use crate::v8::v8_promise::V8LocalPromise;
use crate::v8::v8_resolver::V8LocalResolver;
use crate::v8::v8_set::V8LocalSet;
use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_utf8::V8LocalUtf8;
use crate::v8::OptionalTryFrom;

/// JS generic local value
pub struct V8LocalValue<'isolate_scope, 'isolate> {
    pub(crate) inner_val: *mut v8_local_value,
    pub(crate) isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
}

/// This stuct is a wrapper for `V8LocalValue` that also have access
/// to the current `V8ContextScope`.
/// It is used by `NativeFunctionArgument` derive macro to be able
/// to get arguments from JS object.
pub struct V8CtxValue<'isolate_scope, 'isolate, 'value, 'ctx_scope> {
    pub(crate) val: &'value V8LocalValue<'isolate_scope, 'isolate>,
    pub(crate) ctx_scope: &'ctx_scope V8ContextScope<'isolate_scope, 'isolate>,
}

/// JS generic persisted value
#[derive(Debug, Clone)]
pub struct V8PersistValue {
    pub(crate) inner_val: *mut v8_persisted_value,
    forget: bool,
}

impl<'isolate_scope, 'isolate, 'value, 'ctx_scope>
    V8CtxValue<'isolate_scope, 'isolate, 'value, 'ctx_scope>
{
    pub fn new(
        val: &'value V8LocalValue<'isolate_scope, 'isolate>,
        ctx_scope: &'ctx_scope V8ContextScope<'isolate_scope, 'isolate>,
    ) -> V8CtxValue<'isolate_scope, 'isolate, 'value, 'ctx_scope> {
        V8CtxValue { val, ctx_scope }
    }

    pub fn get_ctx_scope(&self) -> &'ctx_scope V8ContextScope<'isolate_scope, 'isolate> {
        self.ctx_scope
    }

    pub fn get_value(&self) -> &'value V8LocalValue<'isolate_scope, 'isolate> {
        self.val
    }
}

impl<'isolate_scope, 'isolate> V8LocalValue<'isolate_scope, 'isolate> {
    /// Return string representation of the value or None on failure
    #[must_use]
    pub fn to_utf8(&self) -> Option<V8LocalUtf8<'isolate_scope, 'isolate>> {
        let inner_val =
            unsafe { v8_ToUtf8(self.isolate_scope.isolate.inner_isolate, self.inner_val) };
        if inner_val.is_null() {
            None
        } else {
            Some(V8LocalUtf8 {
                inner_val,
                _isolate_scope: self.isolate_scope,
            })
        }
    }

    /// Return true if the value is string and false otherwise.
    #[must_use]
    pub fn is_string(&self) -> bool {
        (unsafe { v8_ValueIsString(self.inner_val) } != 0)
    }

    /// Convert the object into a string, applicable only if the value is string.
    #[must_use]
    pub fn as_string(&self) -> V8LocalString<'isolate_scope, 'isolate> {
        let inner_str = unsafe { v8_ValueAsString(self.inner_val) };
        V8LocalString {
            inner_string: inner_str,
            isolate_scope: self.isolate_scope,
        }
    }

    /// Return true if the value is string object and false otherwise.
    #[must_use]
    pub fn is_string_object(&self) -> bool {
        (unsafe { v8_ValueIsStringObject(self.inner_val) } != 0)
    }

    /// Return true if the value is string and false otherwise.
    #[must_use]
    pub fn is_array(&self) -> bool {
        (unsafe { v8_ValueIsArray(self.inner_val) } != 0)
    }

    /// Convert the object into a string, applicable only if the value is string.
    #[must_use]
    pub fn as_array(&self) -> V8LocalArray<'isolate_scope, 'isolate> {
        let inner_array = unsafe { v8_ValueAsArray(self.inner_val) };
        V8LocalArray {
            inner_array,
            isolate_scope: self.isolate_scope,
        }
    }

    /// Return true if the value is string and false otherwise.
    #[must_use]
    pub fn is_array_buffer(&self) -> bool {
        (unsafe { v8_ValueIsArrayBuffer(self.inner_val) } != 0)
    }

    /// Convert the object into a string, applicable only if the value is string.
    #[must_use]
    pub fn as_array_buffer(&self) -> V8LocalArrayBuffer<'isolate_scope, 'isolate> {
        let inner_array_buffer = unsafe { v8_ValueAsArrayBuffer(self.inner_val) };
        V8LocalArrayBuffer {
            inner_array_buffer,
            isolate_scope: self.isolate_scope,
        }
    }

    /// Return true if the value is null and false otherwise.
    #[must_use]
    pub fn is_null(&self) -> bool {
        (unsafe { v8_ValueIsNull(self.inner_val) } != 0)
    }

    /// Return true if the value is null and false otherwise.
    pub fn is_undefined(&self) -> bool {
        (unsafe { v8_ValueIsUndefined(self.inner_val) } != 0)
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
        (unsafe { v8_GetBool(self.inner_val) } != 0)
    }

    /// Return true if the value is promise and false otherwise.
    #[must_use]
    pub fn is_promise(&self) -> bool {
        (unsafe { v8_ValueIsPromise(self.inner_val) } != 0)
    }

    /// Convert the object into a promise, applicable only if the object is promise.
    #[must_use]
    pub fn as_promise(&self) -> V8LocalPromise<'isolate_scope, 'isolate> {
        let inner_promise = unsafe { v8_ValueAsPromise(self.inner_val) };
        V8LocalPromise {
            inner_promise,
            isolate_scope: self.isolate_scope,
        }
    }

    /// Convert the object into a resolver, applicable only if the object is resolver.
    #[must_use]
    pub fn as_resolver(&self) -> V8LocalResolver<'isolate_scope, 'isolate> {
        let inner_resolver = unsafe { v8_ValueAsResolver(self.inner_val) };
        V8LocalResolver {
            inner_resolver,
            isolate_scope: self.isolate_scope,
        }
    }

    /// Return true if the value is object and false otherwise.
    #[must_use]
    pub fn is_object(&self) -> bool {
        (unsafe { v8_ValueIsObject(self.inner_val) } != 0)
    }

    /// Convert the object into a promise, applicable only if the object is promise.
    #[must_use]
    pub fn as_object(&self) -> V8LocalObject<'isolate_scope, 'isolate> {
        let inner_obj = unsafe { v8_ValueAsObject(self.inner_val) };
        V8LocalObject {
            inner_obj,
            isolate_scope: self.isolate_scope,
        }
    }

    /// Convert the value into a another JS value. notice that both will point to the same
    /// JS value. the differece is that they are different local pointers which means that
    /// the new given value can be used for `try_from` line functions.
    #[must_use]
    pub fn as_value(&self) -> V8LocalValue<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ValueToValue(self.inner_val) };
        V8LocalValue {
            inner_val,
            isolate_scope: self.isolate_scope,
        }
    }

    #[must_use]
    pub fn is_external(&self) -> bool {
        (unsafe { v8_ValueIsExternalData(self.inner_val) } != 0)
    }

    #[must_use]
    pub fn as_external_data(&self) -> V8LocalExternalData<'isolate_scope, 'isolate> {
        let inner_obj = unsafe { v8_ValueAsExternalData(self.inner_val) };
        V8LocalExternalData {
            inner_ext: inner_obj,
            isolate_scope: self.isolate_scope,
        }
    }

    /// Return true if the value is set and false otherwise.
    #[must_use]
    pub fn is_set(&self) -> bool {
        (unsafe { v8_ValueIsSet(self.inner_val) } != 0)
    }

    /// Convert the object into a promise, applicable only if the object is promise.
    #[must_use]
    pub fn as_set(&self) -> V8LocalSet<'isolate_scope, 'isolate> {
        let inner_set = unsafe { v8_ValueAsSet(self.inner_val) };
        V8LocalSet {
            inner_set,
            isolate_scope: self.isolate_scope,
        }
    }

    /// Persist the local object so it can be saved beyond the current handlers scope.
    #[must_use]
    pub fn persist(&self) -> V8PersistValue {
        let inner_val =
            unsafe { v8_PersistValue(self.isolate_scope.isolate.inner_isolate, self.inner_val) };
        V8PersistValue {
            inner_val,
            forget: false,
        }
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
            Some(Self {
                inner_val: res,
                isolate_scope: self.isolate_scope,
            })
        }
    }
}

impl V8PersistValue {
    /// Convert the persisted value back to local value.
    #[must_use]
    pub fn as_local<'isolate, 'isolate_scope>(
        &self,
        isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
    ) -> V8LocalValue<'isolate_scope, 'isolate> {
        assert!(!self.inner_val.is_null());
        let inner_val = unsafe {
            v8_PersistedValueToLocal(isolate_scope.isolate.inner_isolate, self.inner_val)
        };
        V8LocalValue {
            inner_val,
            isolate_scope,
        }
    }

    pub fn forget(&mut self) {
        assert!(!self.inner_val.is_null());
        self.forget = true;
    }

    pub fn take_local<'isolate, 'isolate_scope>(
        &mut self,
        isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
    ) -> V8LocalValue<'isolate_scope, 'isolate> {
        let val = self.as_local(isolate_scope);
        unsafe { v8_FreePersistedValue(self.inner_val) }
        self.forget();
        self.inner_val = ptr::null_mut();
        val
    }
}

unsafe impl Sync for V8PersistValue {}
unsafe impl Send for V8PersistValue {}

impl<'isolate_scope, 'isolate> Drop for V8LocalValue<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        if !self.inner_val.is_null() {
            unsafe { v8_FreeValue(self.inner_val) }
        }
    }
}

impl Drop for V8PersistValue {
    fn drop(&mut self) {
        if self.forget {
            return;
        }
        unsafe { v8_FreePersistedValue(self.inner_val) }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<&V8LocalValue<'isolate_scope, 'isolate>> for i64 {
    type Error = &'static str;

    fn try_from(val: &V8LocalValue<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_long() {
            return Err("Value is not long");
        }

        Ok(val.get_long())
    }
}

impl<'isolate_scope, 'isolate> TryFrom<&V8LocalValue<'isolate_scope, 'isolate>> for f64 {
    type Error = &'static str;

    fn try_from(val: &V8LocalValue<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_number() {
            return Err("Value is not number");
        }

        Ok(val.get_number())
    }
}

impl<'isolate_scope, 'isolate> TryFrom<&V8LocalValue<'isolate_scope, 'isolate>> for String {
    type Error = &'static str;

    fn try_from(val: &V8LocalValue<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_string() && !val.is_string_object() {
            return Err("Value is not string");
        }

        let v8_utf8 = match val.to_utf8() {
            Some(val) => val,
            None => return Err("Failed converting to utf8"),
        };
        Ok(v8_utf8.as_str().to_string())
    }
}

impl<'isolate_scope, 'isolate> TryFrom<&V8LocalValue<'isolate_scope, 'isolate>> for bool {
    type Error = &'static str;

    fn try_from(val: &V8LocalValue<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_boolean() {
            return Err("Value is not a boolean");
        }

        Ok(val.get_boolean())
    }
}

// impl<'isolate_scope, 'isolate> TryFrom<V8LocalValue<'isolate_scope, 'isolate>> for V8LocalValue<'isolate_scope, 'isolate>
// {
//     type Error = &'static str;

//     fn try_from(val: V8LocalValue<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
//         Ok(val)
//     }
// }

macro_rules! from_iter_impl {
    ( $x:ty ) => {
        impl<'isolate_scope, 'isolate> TryFrom<V8LocalValue<'isolate_scope, 'isolate>> for $x {
            type Error = &'static str;

            fn try_from(val: V8LocalValue<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
                (&val).try_into()
            }
        }

        impl<'isolate_scope, 'isolate, 'ctx_scope, 'a>
            TryFrom<&mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'ctx_scope, 'a>>
            for $x
        {
            type Error = &'static str;
            fn try_from(
                val: &mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'ctx_scope, 'a>,
            ) -> Result<Self, Self::Error> {
                match val.next() {
                    Some(val) => val.try_into(),
                    None => Err("Wrong number of arguments given".into()),
                }
            }
        }

        impl<'isolate_scope, 'isolate, 'value, 'ctx_scope>
            TryFrom<V8CtxValue<'isolate_scope, 'isolate, 'value, 'ctx_scope>> for $x
        {
            type Error = &'static str;
            fn try_from(
                val: V8CtxValue<'isolate_scope, 'isolate, 'value, 'ctx_scope>,
            ) -> Result<Self, Self::Error> {
                val.val.try_into()
            }
        }
    };
}

from_iter_impl!(i64);
from_iter_impl!(f64);
from_iter_impl!(String);
from_iter_impl!(bool);
from_iter_impl!(V8LocalArray<'isolate_scope, 'isolate>);
from_iter_impl!(V8LocalArrayBuffer<'isolate_scope, 'isolate>);
from_iter_impl!(V8LocalObject<'isolate_scope, 'isolate>);
from_iter_impl!(V8LocalSet<'isolate_scope, 'isolate>);
from_iter_impl!(V8LocalUtf8<'isolate_scope, 'isolate>);

impl<'isolate_scope, 'isolate, 'ctx_scope, 'a>
    TryFrom<&mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'ctx_scope, 'a>>
    for V8LocalValue<'isolate_scope, 'isolate>
{
    type Error = &'static str;
    fn try_from(
        val: &mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'ctx_scope, 'a>,
    ) -> Result<Self, Self::Error> {
        val.next().ok_or("Wrong number of arguments given")
    }
}

impl<'isolate_scope, 'isolate, 'value, 'ctx_scope>
    TryFrom<V8CtxValue<'isolate_scope, 'isolate, 'value, 'ctx_scope>>
    for V8LocalValue<'isolate_scope, 'isolate>
{
    type Error = &'static str;
    fn try_from(
        val: V8CtxValue<'isolate_scope, 'isolate, 'value, 'ctx_scope>,
    ) -> Result<Self, Self::Error> {
        Ok(val.get_value().as_value())
    }
}

impl<'isolate_scope, 'isolate, 'ctx_scope, 'a, T>
    OptionalTryFrom<&mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'ctx_scope, 'a>>
    for T
where
    T: TryFrom<V8LocalValue<'isolate_scope, 'isolate>, Error = &'static str>,
{
    type Error = &'static str;
    fn optional_try_from(
        val: &mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'ctx_scope, 'a>,
    ) -> Result<Option<Self>, Self::Error> {
        let val = match val.next() {
            Some(v) => v,
            None => return Ok(None),
        };
        val.try_into().map(|v| Some(v))
    }
}

impl<'isolate_scope, 'isolate, 'ctx_scope, 'a>
    OptionalTryFrom<&mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'ctx_scope, 'a>>
    for V8LocalValue<'isolate_scope, 'isolate>
{
    type Error = &'static str;
    fn optional_try_from(
        val: &mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'ctx_scope, 'a>,
    ) -> Result<Option<Self>, Self::Error> {
        Ok(val.next())
    }
}

impl<'isolate_scope, 'isolate, 'ctx_scope, 'a, T>
    TryFrom<&mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'ctx_scope, 'a>> for Vec<T>
where
    T: TryFrom<V8LocalValue<'isolate_scope, 'isolate>, Error = &'static str>,
{
    type Error = &'static str;
    fn try_from(
        val: &mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'ctx_scope, 'a>,
    ) -> Result<Self, Self::Error> {
        let mut res = Self::new();
        for v in val {
            match v.try_into() {
                Ok(v) => res.push(v),
                Err(e) => return Err(e),
            }
        }
        Ok(res)
    }
}

impl<'isolate_scope, 'isolate, 'ctx_scope, 'a>
    TryFrom<&mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'ctx_scope, 'a>>
    for Vec<V8LocalValue<'isolate_scope, 'isolate>>
{
    type Error = &'static str;
    fn try_from(
        val: &mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'ctx_scope, 'a>,
    ) -> Result<Self, Self::Error> {
        Ok(val.collect())
    }
}
