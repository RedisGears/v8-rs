/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! This is an abstraction over a scoped V8 value type.
//! As this is a fully abstract class in V8, the objects it can hold
//! can be of any javascript type. Hence it is generic.

use std::ptr::NonNull;

use crate::{
    v8::context_scope::ContextScope,
    v8_c_raw::bindings::{
        v8_FreeValue, v8_FunctionCall, v8_GetBigInt, v8_GetBool, v8_GetNumber, v8_PersistValue,
        v8_ValueAsArray, v8_ValueAsArrayBuffer, v8_ValueAsExternalData, v8_ValueAsObject,
        v8_ValueAsPromise, v8_ValueAsResolver, v8_ValueAsSet, v8_ValueAsString, v8_ValueIsArray,
        v8_ValueIsArrayBuffer, v8_ValueIsAsyncFunction, v8_ValueIsBigInt, v8_ValueIsBool,
        v8_ValueIsExternalData, v8_ValueIsFunction, v8_ValueIsNull, v8_ValueIsNumber,
        v8_ValueIsObject, v8_ValueIsPromise, v8_ValueIsSet, v8_ValueIsString,
        v8_ValueIsStringObject, v8_local_value,
    },
};

use super::{
    array::LocalArray, array_buffer::LocalArrayBuffer, external_data::LocalExternalData,
    object::LocalObject, persistent::PersistValue, promise::LocalPromise,
    resolver::LocalPromiseResolver, set::LocalSet, string::LocalString, utf8::LocalUtf8,
    ScopedValue,
};

/// A type the objects of [LocalValueAny] can hold.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Type {
    /// The `BigInt` type in JavaScript.
    /// BigInt values represent numeric values which are too large to be
    /// represented by the number
    BigInteger,
    /// The `Number` type in JavaScript.
    /// All primitive numbers in JavaScript are of this (`Number`) type,
    /// even "Integer" literals like `5` is a floating-point value of
    /// type `Number`.
    ///
    /// # Example
    ///
    /// ```javascript
    /// 255; // two-hundred and fifty-five
    /// 255.0; // same number
    /// 255 === 255.0; // true
    /// 255 === 0xff; // true (hexadecimal notation)
    /// 255 === 0b11111111; // true (binary notation)
    /// 255 === 0.255e+3; // true (decimal exponential notation)
    /// ```
    ///
    /// See more at
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number>.
    Number,
    /// A boolean JavaScript object (not primitive!).
    ///
    /// # Example
    ///
    /// ```javascript
    /// const x = new Boolean(false);
    /// if (x) {
    ///   // this code is executed
    /// }
    /// // However:
    /// const x = false;
    /// if (x) {
    ///   // this code is not executed
    /// }
    /// ```
    Boolean,
    /// A JavaScript's `null` value.
    Null,
    /// The Set object lets you store unique values of any type,
    /// whether primitive values or object references.
    ///
    /// # Example
    ///
    /// ```javascript
    /// const mySet1 = new Set();
    ///
    /// mySet1.add(1); // Set(1) { 1 }
    /// ```
    Set,
    /// A foreign, non-JavaScript object which can be worked with in
    /// JavaScript. It doesn't have methods, but can be passed and
    /// received. Usually, for such objects an API is provided to work
    /// with those. An example may be a C++ object, for which a pointer
    /// is provided to JavaScript as [Type::ExternalData].
    ExternalData,
    /// A JavaScript object. See more at
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object>.
    Object,
    /// Resolver is an object which can resolve or reject promises.
    /// See more at
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/resolve>.
    Resolver,
    /// The Promise object represents the eventual completion
    /// (or failure) of an asynchronous operation and its resulting
    /// value.
    ///
    /// # Example
    ///
    /// ```javascript
    /// new Promise((resolveOuter) => {
    ///   resolveOuter(
    ///     new Promise((resolveInner) => {
    ///       setTimeout(resolveInner, 1000);
    ///     }),
    ///   );
    /// });
    /// ```
    Promise,
    /// A JavaScript function. Every function is actually an object of
    /// type `Function`.
    ///
    /// # [Example](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function)
    ///
    /// ```javascript
    /// // Create a global property with `var`
    /// var x = 10;
    ///
    /// function createFunction1() {
    ///     const x = 20;
    ///     return new Function("return x;"); // this `x` refers to global `x`
    /// }
    ///
    /// function createFunction2() {
    ///     const x = 20;
    ///     function f() {
    ///         return x; // this `x` refers to the local `x` above
    ///     }
    ///     return f;
    /// }
    ///
    /// const f1 = createFunction1();
    /// console.log(f1()); // 10
    /// const f2 = createFunction2();
    /// console.log(f2()); // 20
    /// ```
    Function,
    /// An asynchrous function. Every async function in JavaScript is
    /// actually an AsyncFunction object.
    ///
    AsyncFunction,
    /// A javascript array, which can be created like this:
    /// ```javascript
    /// const fruits = [];
    /// fruits.push("banana", "apple", "peach");
    /// ```
    Array,
    /// The ArrayBuffer is used to represent a generic raw binary data
    /// buffer. Example:
    /// ```javascript
    /// const buffer = new ArrayBuffer(8);
    /// const view = new Int32Array(buffer);
    /// ```
    ArrayBuffer,
    /// A literal string in JavaScript. For example:
    /// ```javascript
    /// const string1 = "A string primitive";
    /// const string2 = 'Also a string primitive';
    /// const string3 = `Yet another string primitive`;
    /// ```
    String,
    /// A string object. A string object is a proper JavaScript object
    /// which can be created the following way:
    /// ```javascript
    /// const stringObject = new String("A String object");
    /// ```
    StringObject,
    /// A UTF-8 encoded string.
    Utf8,
}

/// A local value for which there is no type information available.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct LocalValueAny<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_local_value>,
);

impl<'isolate_scope, 'isolate> LocalValueAny<'isolate_scope, 'isolate> {
    /// Returns the type of the value hold if it the valid, [`None`]
    /// otherwise.
    pub fn get_type(&self) -> Option<Type> {
        Some(if self.is_array() {
            Type::Array
        } else if self.is_array_buffer() {
            Type::ArrayBuffer
        } else if self.is_async_function() {
            Type::AsyncFunction
        } else if self.is_boolean() {
            Type::Boolean
        } else if self.is_external_data() {
            Type::ExternalData
        } else if self.is_function() {
            Type::Function
        } else if self.is_long() {
            Type::BigInteger
        } else if self.is_null() {
            Type::Null
        } else if self.is_number() {
            Type::Number
        } else if self.is_object() {
            Type::Object
        } else if self.is_promise() {
            Type::Promise
        } else if self.is_set() {
            Type::Set
        } else if self.is_string() {
            Type::String
        } else if self.is_string_object() {
            Type::StringObject
        } else {
            return None;
        })
    }

    /// Return string representation of the value or None on failure
    #[deprecated = "Use [LocalUtf8::try_from] instead."]
    pub fn into_utf8(self) -> Option<LocalUtf8<'isolate_scope, 'isolate>> {
        LocalUtf8::try_from(self).ok()
    }

    /// Return true if the value is string and false otherwise.
    pub fn is_string(&self) -> bool {
        (unsafe { v8_ValueIsString(self.0.inner_val) } != 0)
    }

    /// Convert the object into a string, applicable only if the value is string.
    ///
    /// # Safety
    ///
    /// The function doesn't perform checks for the value being actually
    /// of the target type. And doesn't panic if this is not the case.
    /// If a fallible conversion is preferred, use [`TryFrom`].
    ///
    /// In case the target type is not checked before this function is
    /// invoked and the value is not of this target type, the results
    /// are unknown.
    pub unsafe fn as_string(&self) -> LocalString<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ValueAsString(self.0.inner_val) };
        LocalString(ScopedValue {
            inner_val,
            isolate_scope: self.0.isolate_scope,
        })
    }

    /// Return true if the value is string object and false otherwise.
    pub fn is_string_object(&self) -> bool {
        (unsafe { v8_ValueIsStringObject(self.0.inner_val) } != 0)
    }

    /// Return true if the value is string and false otherwise.
    pub fn is_array(&self) -> bool {
        (unsafe { v8_ValueIsArray(self.0.inner_val) } != 0)
    }

    /// Convert the object into a string, applicable only if the value is string.
    ///
    /// # Safety
    ///
    /// The function doesn't perform checks for the value being actually
    /// of the target type. And doesn't panic if this is not the case.
    /// If a fallible conversion is preferred, use [`TryFrom`].
    ///
    /// In case the target type is not checked before this function is
    /// invoked and the value is not of this target type, the results
    /// are unknown.
    pub unsafe fn as_array(&self) -> LocalArray<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ValueAsArray(self.0.inner_val) };
        LocalArray(ScopedValue {
            inner_val,
            isolate_scope: self.0.isolate_scope,
        })
    }

    /// Return true if the value is string and false otherwise.
    pub fn is_array_buffer(&self) -> bool {
        (unsafe { v8_ValueIsArrayBuffer(self.0.inner_val) } != 0)
    }

    /// Convert the object into a string, applicable only if the value is string.
    ///
    /// # Safety
    ///
    /// The function doesn't perform checks for the value being actually
    /// of the target type. And doesn't panic if this is not the case.
    /// If a fallible conversion is preferred, use [`TryFrom`].
    ///
    /// In case the target type is not checked before this function is
    /// invoked and the value is not of this target type, the results
    /// are unknown.
    pub unsafe fn as_array_buffer(&self) -> LocalArrayBuffer<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ValueAsArrayBuffer(self.0.inner_val) };
        LocalArrayBuffer(ScopedValue {
            inner_val,
            isolate_scope: self.0.isolate_scope,
        })
    }

    /// Return true if the value is null and false otherwise.
    pub fn is_null(&self) -> bool {
        (unsafe { v8_ValueIsNull(self.0.inner_val) } != 0)
    }

    /// Return true if the value is function and false otherwise.
    pub fn is_function(&self) -> bool {
        (unsafe { v8_ValueIsFunction(self.0.inner_val) } != 0)
    }

    /// Return true if the value is async function and false otherwise.
    pub fn is_async_function(&self) -> bool {
        (unsafe { v8_ValueIsAsyncFunction(self.0.inner_val) } != 0)
    }

    /// Return true if the value is number and false otherwise.
    pub fn is_number(&self) -> bool {
        (unsafe { v8_ValueIsNumber(self.0.inner_val) } != 0)
    }

    /// Returns an [f64] value.
    ///
    /// # Safety
    ///
    /// The function doesn't perform checks for the value being actually
    /// of the target type. And doesn't panic if this is not the case.
    /// If a fallible conversion is preferred, use [`TryFrom`].
    ///
    /// In case the target type is not checked before this function is
    /// invoked and the value is not of this target type, the results
    /// are unknown.
    pub unsafe fn get_number(&self) -> f64 {
        unsafe { v8_GetNumber(self.0.inner_val) }
    }

    /// Return true if the value is number and false otherwise.
    pub fn is_long(&self) -> bool {
        (unsafe { v8_ValueIsBigInt(self.0.inner_val) } != 0)
    }

    /// Returns an [i64] value.
    ///
    /// # Safety
    ///
    /// The function doesn't perform checks for the value being actually
    /// of the target type. And doesn't panic if this is not the case.
    /// If a fallible conversion is preferred, use [`TryFrom`].
    ///
    /// In case the target type is not checked before this function is
    /// invoked and the value is not of this target type, the results
    /// are unknown.
    pub unsafe fn get_long(&self) -> i64 {
        unsafe { v8_GetBigInt(self.0.inner_val) }
    }

    /// Return true if the value is boolean and false otherwise.
    pub fn is_boolean(&self) -> bool {
        (unsafe { v8_ValueIsBool(self.0.inner_val) } != 0)
    }

    /// Returns a [bool] value.
    ///
    /// # Safety
    ///
    /// The function doesn't perform checks for the value being actually
    /// of the target type. And doesn't panic if this is not the case.
    /// If a fallible conversion is preferred, use [`TryFrom`].
    ///
    /// In case the target type is not checked before this function is
    /// invoked and the value is not of this target type, the results
    /// are unknown.
    pub unsafe fn get_boolean(&self) -> bool {
        (unsafe { v8_GetBool(self.0.inner_val) } != 0)
    }

    /// Return true if the value is promise and false otherwise.
    pub fn is_promise(&self) -> bool {
        (unsafe { v8_ValueIsPromise(self.0.inner_val) } != 0)
    }

    /// Returns a [LocalPromise] value.
    ///
    /// # Safety
    ///
    /// The function doesn't perform checks for the value being actually
    /// of the target type. And doesn't panic if this is not the case.
    /// If a fallible conversion is preferred, use [`TryFrom`].
    ///
    /// In case the target type is not checked before this function is
    /// invoked and the value is not of this target type, the results
    /// are unknown.
    pub unsafe fn as_promise(&self) -> LocalPromise<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ValueAsPromise(self.0.inner_val) };
        LocalPromise(ScopedValue {
            inner_val,
            isolate_scope: self.0.isolate_scope,
        })
    }

    /// Returns a [LocalResolver] value.
    ///
    /// # Safety
    ///
    /// The function doesn't perform checks for the value being actually
    /// of the target type. And doesn't panic if this is not the case.
    /// If a fallible conversion is preferred, use [`TryFrom`].
    ///
    /// In case the target type is not checked before this function is
    /// invoked and the value is not of this target type, the results
    /// are unknown.
    pub unsafe fn as_resolver(&self) -> LocalPromiseResolver<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ValueAsResolver(self.0.inner_val) };
        LocalPromiseResolver(ScopedValue {
            inner_val,
            isolate_scope: self.0.isolate_scope,
        })
    }

    /// Return true if the value is object and false otherwise.
    pub fn is_object(&self) -> bool {
        (unsafe { v8_ValueIsObject(self.0.inner_val) } != 0)
    }

    /// Returns a [LocalObject] value.
    ///
    /// # Safety
    ///
    /// The function doesn't perform checks for the value being actually
    /// of the target type. And doesn't panic if this is not the case.
    /// If a fallible conversion is preferred, use [`TryFrom`].
    ///
    /// In case the target type is not checked before this function is
    /// invoked and the value is not of this target type, the results
    /// are unknown.
    pub unsafe fn as_object(&self) -> LocalObject<'isolate_scope, 'isolate> {
        let inner_obj = unsafe { v8_ValueAsObject(self.0.inner_val) };
        LocalObject(ScopedValue {
            inner_val: inner_obj,
            isolate_scope: self.0.isolate_scope,
        })
    }

    /// Returns `true` if the value stored is of type `External`. If it
    /// is so, this object can be converted into the [LocalExternalData].
    pub fn is_external_data(&self) -> bool {
        (unsafe { v8_ValueIsExternalData(self.0.inner_val) } != 0)
    }

    /// Returns a [LocalExternalData] value.
    ///
    /// # Safety
    ///
    /// The function doesn't perform checks for the value being actually
    /// of the target type. And doesn't panic if this is not the case.
    /// If a fallible conversion is preferred, use [`TryFrom`].
    ///
    /// In case the target type is not checked before this function is
    /// invoked and the value is not of this target type, the results
    /// are unknown.
    pub unsafe fn as_external_data(&self) -> LocalExternalData<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ValueAsExternalData(self.0.inner_val) };
        LocalExternalData(ScopedValue {
            inner_val,
            isolate_scope: self.0.isolate_scope,
        })
    }

    /// Return true if the value is set and false otherwise.
    pub fn is_set(&self) -> bool {
        (unsafe { v8_ValueIsSet(self.0.inner_val) } != 0)
    }

    /// Returns a [LocalSet] value.
    ///
    /// # Safety
    ///
    /// The function doesn't perform checks for the value being actually
    /// of the target type. And doesn't panic if this is not the case.
    /// If a fallible conversion is preferred, use [`TryFrom`].
    ///
    /// In case the target type is not checked before this function is
    /// invoked and the value is not of this target type, the results
    /// are unknown.
    pub unsafe fn as_set(&self) -> LocalSet<'isolate_scope, 'isolate> {
        let inner_val = unsafe { v8_ValueAsSet(self.0.inner_val) };
        LocalSet(ScopedValue {
            inner_val,
            isolate_scope: self.0.isolate_scope,
        })
    }

    /// Persist the local object so it can be saved beyond the current handlers scope.
    /// TODO move to `From` impl.
    pub fn persist(&self) -> PersistValue {
        let inner_val = unsafe {
            v8_PersistValue(self.0.isolate_scope.isolate.inner_isolate, self.0.inner_val)
        };
        PersistValue::from(inner_val)
    }

    /// Run the value, applicable only if the value is a function or async function.
    pub fn call(&self, ctx: &ContextScope, args: Option<&[&Self]>) -> Option<Self> {
        NonNull::new(match args {
            Some(args) => {
                let args = args
                    .iter()
                    .map(|v| v.0.inner_val)
                    .collect::<Vec<*mut v8_local_value>>();
                let ptr = args.as_ptr();
                unsafe { v8_FunctionCall(ctx.inner_ctx_ref, self.0.inner_val, args.len(), ptr) }
            }
            None => unsafe {
                v8_FunctionCall(ctx.inner_ctx_ref, self.0.inner_val, 0, std::ptr::null())
            },
        })
        .map(|ptr| {
            Self(ScopedValue {
                inner_val: ptr.as_ptr(),
                isolate_scope: self.0.isolate_scope,
            })
        })
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalValueAny<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        if !self.0.inner_val.is_null() {
            unsafe { v8_FreeValue(self.0.inner_val) }
        }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueAny<'isolate_scope, 'isolate>> for String {
    type Error = &'static str;

    fn try_from(val: LocalValueAny<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        LocalUtf8::try_from(val).map(|ls| ls.as_str().to_owned())
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueAny<'isolate_scope, 'isolate>> for bool {
    type Error = &'static str;

    fn try_from(val: LocalValueAny<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_boolean() {
            return Err("Value is not a boolean");
        }

        Ok(unsafe { val.get_boolean() })
    }
}
