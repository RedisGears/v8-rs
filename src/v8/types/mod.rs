/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{v8_NewBool, v8_NewNull, v8_ValueFromDouble, v8_ValueFromLong};

pub mod any;
pub mod array;
pub mod array_buffer;
pub mod external_data;
pub mod module;
pub mod native_function;
pub mod native_function_template;
pub mod object;
pub mod object_template;
pub mod persistent;
pub mod promise;
pub mod resolver;
pub mod script;
pub mod set;
pub mod string;
pub mod try_catch;
pub mod unlocker;
pub mod utf8;

use crate::v8::context_scope::ContextScope;
use crate::v8::isolate_scope::IsolateScope;
use crate::v8::OptionalTryFrom;
use array::LocalArray;
use array_buffer::LocalArrayBuffer;
use external_data::LocalExternalData;
use native_function_template::V8LocalNativeFunctionArgsIter;
use object::LocalObject;
use promise::LocalPromise;
use set::LocalSet;
use string::LocalString;
use utf8::LocalUtf8;

use self::any::{LocalValueAny, Type};
use self::native_function_template::{LocalNativeFunctionArgs, LocalNativeFunctionTemplate};
use self::object_template::LocalObjectTemplate;
use self::try_catch::TryCatch;
use self::unlocker::Unlocker;

/// A generic, isolate-scoped JavaScript value.
#[derive(Debug, Copy, Clone)]
pub struct ScopedValue<'isolate_scope, 'isolate, BindingType: std::fmt::Debug> {
    pub(crate) inner_val: *mut BindingType,
    pub(crate) isolate_scope: &'isolate_scope IsolateScope<'isolate>,
}

/// An isolate-scoped [f64] local value in JavaScript.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct LocalValueDouble<'isolate_scope, 'isolate>(
    pub(crate) LocalValueAny<'isolate_scope, 'isolate>,
);

impl<'isolate_scope, 'isolate> LocalValueDouble<'isolate_scope, 'isolate> {
    fn new(value: f64, isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        let inner_val = unsafe { v8_ValueFromDouble(isolate_scope.isolate.inner_isolate, value) };
        Self(LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope,
        }))
    }
}

impl<'isolate_scope, 'isolate> From<LocalValueDouble<'isolate_scope, 'isolate>> for f64 {
    fn from(value: LocalValueDouble<'isolate_scope, 'isolate>) -> Self {
        unsafe { value.0.get_number() }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueAny<'isolate_scope, 'isolate>> for f64 {
    type Error = &'static str;

    fn try_from(val: LocalValueAny<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        Ok(LocalValueDouble::try_from(val)?.into())
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueAny<'isolate_scope, 'isolate>>
    for LocalValueDouble<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: LocalValueAny<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_number() {
            return Err("Value is not a number");
        }

        Ok(Self(val))
    }
}

/// An isolate-scoped [i64] local value in JavaScript.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct LocalValueInteger<'isolate_scope, 'isolate>(
    pub(crate) LocalValueAny<'isolate_scope, 'isolate>,
);

impl<'isolate_scope, 'isolate> LocalValueInteger<'isolate_scope, 'isolate> {
    fn new(value: i64, isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        let inner_val = unsafe { v8_ValueFromLong(isolate_scope.isolate.inner_isolate, value) };
        Self(LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope,
        }))
    }
}

impl<'isolate_scope, 'isolate> From<LocalValueInteger<'isolate_scope, 'isolate>> for i64 {
    fn from(value: LocalValueInteger<'isolate_scope, 'isolate>) -> Self {
        unsafe { value.0.get_long() }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueAny<'isolate_scope, 'isolate>>
    for LocalValueInteger<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: LocalValueAny<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_long() {
            return Err("Value is not a long");
        }

        Ok(Self(val))
    }
}

/// An isolate-scoped [bool] local value in JavaScript.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct LocalValueBoolean<'isolate_scope, 'isolate>(
    pub(crate) LocalValueAny<'isolate_scope, 'isolate>,
);

impl<'isolate_scope, 'isolate> LocalValueBoolean<'isolate_scope, 'isolate> {
    fn new(value: bool, isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        let inner_val = unsafe { v8_NewBool(isolate_scope.isolate.inner_isolate, value as i32) };
        Self(LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope,
        }))
    }
}

impl<'isolate_scope, 'isolate> From<LocalValueBoolean<'isolate_scope, 'isolate>> for bool {
    fn from(value: LocalValueBoolean<'isolate_scope, 'isolate>) -> Self {
        unsafe { value.0.get_boolean() }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueAny<'isolate_scope, 'isolate>>
    for LocalValueBoolean<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: LocalValueAny<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_boolean() {
            return Err("Value is not a boolean");
        }

        Ok(Self(val))
    }
}

/// An isolate-scoped `null` local value in JavaScript.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct LocalValueNull<'isolate_scope, 'isolate>(
    pub(crate) LocalValueAny<'isolate_scope, 'isolate>,
);

impl<'isolate_scope, 'isolate> LocalValueNull<'isolate_scope, 'isolate> {
    fn new(isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        let inner_val = unsafe { v8_NewNull(isolate_scope.isolate.inner_isolate) };
        Self(LocalValueAny(ScopedValue {
            inner_val,
            isolate_scope,
        }))
    }
}

impl<'isolate_scope, 'isolate> From<&'isolate_scope IsolateScope<'isolate>>
    for LocalValueNull<'isolate_scope, 'isolate>
{
    fn from(value: &'isolate_scope IsolateScope<'isolate>) -> Self {
        Self::new(value)
    }
}

impl<'isolate_scope, 'isolate> TryFrom<LocalValueAny<'isolate_scope, 'isolate>>
    for LocalValueNull<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: LocalValueAny<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if !val.is_null() {
            return Err("Value is not a null");
        }

        Ok(Self(val))
    }
}

/// All the possible values a JavaScript may have with which it is
/// possible to work.
#[derive(Debug, Clone)]
pub enum Value<'isolate_scope, 'isolate> {
    /// See [LocalString].
    String(LocalString<'isolate_scope, 'isolate>),
    /// See [LocalValueBoolean].
    Boolean(LocalValueBoolean<'isolate_scope, 'isolate>),
    /// See [LocalValueInteger].
    Integer(LocalValueInteger<'isolate_scope, 'isolate>),
    /// See [LocalValueDouble].
    Double(LocalValueDouble<'isolate_scope, 'isolate>),
    /// See [LocalArrayBuffer].
    ArrayBuffer(LocalArrayBuffer<'isolate_scope, 'isolate>),
    /// See [LocalArray].
    Array(LocalArray<'isolate_scope, 'isolate>),
    /// See [LocalSet].
    Set(LocalSet<'isolate_scope, 'isolate>),
    /// See [LocalObject].
    Object(LocalObject<'isolate_scope, 'isolate>),
    /// See [LocalObjectTemplate].
    ObjectTemplate(LocalObjectTemplate<'isolate_scope, 'isolate>),
    /// See [TryCatch].
    TryCatch(TryCatch<'isolate_scope, 'isolate>),
    /// See [Unlocker].
    Unlocker(Unlocker<'isolate_scope, 'isolate>),
    /// See [LocalNativeFunctionTemplate].
    NativeFunctionTemplate(LocalNativeFunctionTemplate<'isolate_scope, 'isolate>),
    /// See [LocalValueNull].
    Null(LocalValueNull<'isolate_scope, 'isolate>),
    /// See [LocalExternalData].
    ExternalData(LocalExternalData<'isolate_scope, 'isolate>),
    /// See [LocalValueAny].
    Other(LocalValueAny<'isolate_scope, 'isolate>),
}

impl<'isolate_scope, 'isolate> Value<'isolate_scope, 'isolate> {
    /// Creates a new double local value from [f64] for the passed [IsolateScope].
    pub fn from_f64(value: f64, isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        Value::Double(LocalValueDouble::new(value, isolate_scope))
    }

    /// Creates a new integer local value from [i64] for the passed [IsolateScope].
    pub fn from_i64(value: i64, isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        Value::Integer(LocalValueInteger::new(value, isolate_scope))
    }

    /// Creates a new boolean local value from [bool] for the passed [IsolateScope].
    pub fn from_bool(value: bool, isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        Value::Boolean(LocalValueBoolean::new(value, isolate_scope))
    }

    /// Creates a new local object for the passed [IsolateScope].
    pub fn new_object(isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        Value::Object(LocalObject::new(isolate_scope))
    }

    /// Creates a new local object template for the passed [IsolateScope].
    pub fn new_object_template(isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        Value::ObjectTemplate(LocalObjectTemplate::new(isolate_scope))
    }

    /// Creates a new null local object for the passed [IsolateScope].
    pub fn new_null(isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        Value::Null(LocalValueNull::new(isolate_scope))
    }

    /// Creates a new local set for the passed [IsolateScope].
    pub fn new_set(isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        Value::Set(LocalSet::new(isolate_scope))
    }

    /// Creates a new local string for the passed [IsolateScope].
    pub fn new_string(s: &str, isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        Value::String(LocalString::new(s, isolate_scope))
    }

    /// Creates a new local try catch for the passed [IsolateScope].
    pub fn new_try_catch(isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        Value::TryCatch(TryCatch::new(isolate_scope))
    }

    /// Creates a new local array buffer for the passed [IsolateScope].
    pub fn new_array_buffer(
        bytes: &[u8],
        isolate_scope: &'isolate_scope IsolateScope<'isolate>,
    ) -> Self {
        Value::ArrayBuffer(LocalArrayBuffer::new(bytes, isolate_scope))
    }

    /// Creates a new local array for the passed [IsolateScope].
    pub fn new_array(
        values: &[&LocalValueAny],
        isolate_scope: &'isolate_scope IsolateScope<'isolate>,
    ) -> Self {
        Value::Array(LocalArray::new(values, isolate_scope))
    }

    /// Creates a new unlocker for the passed [IsolateScope].
    pub fn new_unlocker(isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        Value::Unlocker(Unlocker::new(isolate_scope))
    }

    /// Creates a new external data object for the passed [IsolateScope].
    pub fn new_external_data<T>(
        data: T,
        isolate_scope: &'isolate_scope IsolateScope<'isolate>,
    ) -> Self {
        Value::ExternalData(LocalExternalData::new(data, isolate_scope))
    }

    /// Creates a new native function object for the passed [IsolateScope].
    pub fn new_native_function_template<
        T: for<'d, 'c> Fn(
            &LocalNativeFunctionArgs<'d, 'c>,
            &'d IsolateScope<'c>,
            &ContextScope<'d, 'c>,
        ) -> Option<LocalValueAny<'d, 'c>>,
    >(
        function: T,
        isolate_scope: &'isolate_scope IsolateScope<'isolate>,
    ) -> Self {
        Value::NativeFunctionTemplate(LocalNativeFunctionTemplate::new(function, isolate_scope))
    }
}

impl<'isolate_scope, 'isolate> From<LocalValueAny<'isolate_scope, 'isolate>>
    for Value<'isolate_scope, 'isolate>
{
    fn from(value: LocalValueAny<'isolate_scope, 'isolate>) -> Self {
        // TODO rewrite better than this.
        match value.get_type() {
            Some(Type::Array) => {
                Self::Array(LocalArray::try_from(value).expect("Conversion error"))
            }
            Some(Type::ArrayBuffer) => {
                Self::ArrayBuffer(LocalArrayBuffer::try_from(value).expect("Conversion error"))
            }
            Some(Type::String) => {
                Self::String(LocalString::try_from(value).expect("Conversion error"))
            }
            Some(Type::Integer) => {
                Self::Integer(LocalValueInteger::try_from(value).expect("Conversion error"))
            }
            Some(Type::Double) => {
                Self::Double(LocalValueDouble::try_from(value).expect("Conversion error"))
            }
            Some(Type::Set) => Self::Set(LocalSet::try_from(value).expect("Conversion error")),
            Some(Type::Object) => {
                Self::Object(LocalObject::try_from(value).expect("Conversion error"))
            }
            Some(Type::Null) => {
                Self::Null(LocalValueNull::try_from(value).expect("Conversion error"))
            }
            Some(Type::ExternalData) => {
                Self::ExternalData(LocalExternalData::try_from(value).expect("Conversion error"))
            }
            _ => Self::Other(value),
        }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<Value<'isolate_scope, 'isolate>>
    for LocalString<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: Value<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        match val {
            Value::String(ls) => Ok(ls),
            Value::Other(any) => any.try_into(),
            _ => Err("Value is not a string."),
        }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<Value<'isolate_scope, 'isolate>>
    for LocalUtf8<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: Value<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        match val {
            Value::String(s) => LocalValueAny::from(s).try_into(),
            Value::Other(any) => any.try_into(),
            _ => Err("Value is not string"),
        }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<Value<'isolate_scope, 'isolate>>
    for LocalPromise<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: Value<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if let Value::Other(any) = val {
            if any.is_promise() {
                return Ok(unsafe { any.as_promise() });
            }
        }
        Err("Value is not a promise")
    }
}

impl<'isolate_scope, 'isolate> TryFrom<Value<'isolate_scope, 'isolate>>
    for LocalObjectTemplate<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: Value<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if let Value::ObjectTemplate(object_template) = val {
            Ok(object_template)
        } else {
            Err("Value is not a promise")
        }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<Value<'isolate_scope, 'isolate>>
    for TryCatch<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: Value<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if let Value::TryCatch(tc) = val {
            Ok(tc)
        } else {
            Err("Value is not a try catch")
        }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<Value<'isolate_scope, 'isolate>>
    for LocalNativeFunctionTemplate<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: Value<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        if let Value::NativeFunctionTemplate(t) = val {
            Ok(t)
        } else {
            Err("Value is not a local function template")
        }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<Value<'isolate_scope, 'isolate>> for bool {
    type Error = &'static str;

    fn try_from(val: Value<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        Ok(match val {
            Value::Boolean(b) => b.into(),
            Value::Other(any) => {
                if !any.is_boolean() {
                    return Err("Value is not a boolean.");
                }

                unsafe { any.get_boolean() }
            }
            _ => return Err("Value is not a long."),
        })
    }
}

impl<'isolate_scope, 'isolate> TryFrom<Value<'isolate_scope, 'isolate>> for i64 {
    type Error = &'static str;

    fn try_from(val: Value<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        Ok(match val {
            Value::Integer(i) => i.into(),
            Value::Other(any) => {
                if !any.is_long() {
                    return Err("Value is not a long.");
                }

                unsafe { any.get_long() }
            }
            _ => return Err("Value is not a long."),
        })
    }
}

impl<'isolate_scope, 'isolate> TryFrom<Value<'isolate_scope, 'isolate>> for f64 {
    type Error = &'static str;

    fn try_from(val: Value<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        Ok(match val {
            Value::Double(f) => f.into(),
            Value::Other(any) => {
                if !any.is_number() {
                    return Err("Value is not a number");
                }

                unsafe { any.get_number() }
            }
            _ => return Err("Value is not a number."),
        })
    }
}

impl<'isolate_scope, 'isolate> TryFrom<Value<'isolate_scope, 'isolate>> for String {
    type Error = &'static str;

    fn try_from(val: Value<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        match val {
            Value::String(ls) => Ok(LocalUtf8::from(ls).into()),
            // Value::Object(o) => String::try_from(o),
            Value::Other(any) => String::try_from(any),
            _ => Err("Value is not a string."),
        }
    }
}

impl<'isolate_scope, 'isolate> TryFrom<Value<'isolate_scope, 'isolate>>
    for LocalValueAny<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(val: Value<'isolate_scope, 'isolate>) -> Result<Self, Self::Error> {
        match val {
            Value::Array(array) => Ok(array.into()),
            Value::ArrayBuffer(array_buffer) => Ok(array_buffer.into()),
            Value::Boolean(boolean) => Ok(boolean.0),
            Value::Integer(integer) => Ok(integer.0),
            Value::Double(double) => Ok(double.0),
            Value::Null(null) => Ok(null.0),
            Value::String(string) => Ok(string.into()),
            Value::Set(set) => Ok(set.into()),
            Value::Object(object) => Ok(object.into()),
            // LocalValue::ObjectTemplate(object_template) => Ok(object_template.into()),
            // LocalValue::TryCatch(try_catch) => Ok(try_catch.into()),
            // LocalValue::Unlocker(unlocker) => Ok(unlocker.into()),
            // LocalValue::NativeFunctionTemplate(native_function_template) => {
            //     Ok(native_function_template.into())
            // }
            Value::ExternalData(external_data) => Ok(external_data.into()),
            Value::Other(any) => Ok(any),
            _ => Err("Couldn't convert to any"),
        }
    }
}

macro_rules! from_iter_impl {
    ( $x:ty ) => {
        impl<'isolate_scope, 'isolate, 'a>
            TryFrom<&mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a>> for $x
        {
            type Error = &'static str;

            fn try_from(
                val: &mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a>,
            ) -> Result<Self, Self::Error> {
                match val.next() {
                    Some(val) => std::convert::TryInto::<$x>::try_into(val),
                    None => Err("Wrong number of arguments given".into()),
                }
            }
        }
    };
}

from_iter_impl!(i64);
from_iter_impl!(f64);
from_iter_impl!(String);
from_iter_impl!(bool);
from_iter_impl!(LocalArray<'isolate_scope, 'isolate>);
from_iter_impl!(LocalArrayBuffer<'isolate_scope, 'isolate>);
from_iter_impl!(LocalObject<'isolate_scope, 'isolate>);
from_iter_impl!(LocalSet<'isolate_scope, 'isolate>);
from_iter_impl!(LocalUtf8<'isolate_scope, 'isolate>);

impl<'isolate_scope, 'isolate, 'a>
    TryFrom<&mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a>>
    for Value<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn try_from(
        val: &mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a>,
    ) -> Result<Self, Self::Error> {
        val.next().ok_or("Wrong number of arguments given")
    }
}

impl<'isolate_scope, 'isolate, 'a, T>
    OptionalTryFrom<&mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a>> for T
where
    T: TryFrom<LocalValueAny<'isolate_scope, 'isolate>, Error = &'static str>,
{
    type Error = &'static str;

    fn optional_try_from(
        val: &mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a>,
    ) -> Result<Option<Self>, Self::Error> {
        let val = match val.next() {
            Some(v) => v,
            None => return Ok(None),
        };
        let val = LocalValueAny::try_from(val)?;
        let val = val.try_into()?;
        Ok(Some(val))
    }
}

impl<'isolate_scope, 'isolate, 'a>
    OptionalTryFrom<&mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a>>
    for Value<'isolate_scope, 'isolate>
{
    type Error = &'static str;

    fn optional_try_from(
        val: &mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a>,
    ) -> Result<Option<Self>, Self::Error> {
        Ok(val.next())
    }
}

// impl<'isolate_scope, 'isolate, 'a, T>
//     TryFrom<&mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a>> for Vec<T>
// where
//     T: TryFrom<LocalValueAny<'isolate_scope, 'isolate>, Error = &'static str>,
// {
//     type Error = &'static str;

//     fn try_from(
//         val: &mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a>,
//     ) -> Result<Self, Self::Error> {
//         let mut res = Self::new();
//         for v in val {
//             let v = LocalValueAny::try_from(v)?.try_into()?;
//             res.push(v);
//         }
//         Ok(res)
//     }
// }

impl<'isolate_scope, 'isolate, 'a>
    TryFrom<&mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a>>
    for Vec<Value<'isolate_scope, 'isolate>>
{
    type Error = &'static str;

    fn try_from(
        val: &mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a>,
    ) -> Result<Self, Self::Error> {
        Ok(val.collect())
    }
}

impl<'isolate_scope, 'isolate, 'a, T>
    TryFrom<&mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a>> for Vec<T>
where
    T: TryFrom<Value<'isolate_scope, 'isolate>, Error = &'static str>,
{
    type Error = &'static str;

    fn try_from(
        val: &mut V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'a>,
    ) -> Result<Self, Self::Error> {
        Ok(val.map(|v| T::try_from(v).unwrap()).collect())
    }
}
