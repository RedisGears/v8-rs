/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! V8-rs is a crate containing bindings to the V8 C++ API.

#![warn(missing_docs)]

/// The module contains the rust-idiomatic data structures and functions.
pub mod v8;
mod v8_c_raw;

/// The [`vergen`]'s git hash (long) - the full SHA hash of the commit
/// the crate was build from.
pub const GIT_SHA: &str = env!("VERGEN_GIT_SHA");
/// The [`vergen`]'s git semantic version based on the last checkout tag,
/// the number of commits ahead of the tag and a hash.
pub const GIT_SEMVER: &str = env!("VERGEN_GIT_DESCRIBE");

/// A user-available data index. The users of the crate may use this
/// index to store their data in V8.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct UserIndex(usize);
impl From<usize> for UserIndex {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// A raw V8 data index without any offsets. Should be used with
/// caution and shouldn't be allowed to be used directly by the users
/// of the crate.
///
/// The first elements we store in V8 aren't actually the user data,
/// but our internal data instead. So the user shouldn't be allowed to
/// set or get the internal data, and for that purpose we should always
/// correct the index which should point to real data location. This
/// real location is represented by the [`RawIndex`]. Thus, the user
/// index [`UserIndex`] should always be converted into a [`RawIndex`]
/// in order to access the V8 data. This is also why the [`RawIndex`] is
/// only used internally within the crate, so as to not allow the user
/// to work with the internal data and ensure the compile-time safety.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct RawIndex(usize);

impl From<UserIndex> for RawIndex {
    fn from(value: UserIndex) -> Self {
        const INTERNAL_OFFSET: usize = 1;
        Self(value.0 + INTERNAL_OFFSET)
    }
}

#[cfg(test)]
mod json_path_tests {
    use crate as v8_rs;
    use crate::v8::types::any::LocalValueAny;
    use crate::v8::types::native_function::LocalNativeFunction;
    use crate::v8::types::native_function_template::LocalNativeFunctionTemplate;
    use crate::v8::types::object_template::LocalObjectTemplate;
    use crate::v8::types::promise::LocalPromise;
    use crate::v8::types::try_catch::TryCatch;
    use crate::v8::types::utf8::LocalUtf8;
    use crate::v8::types::Value;
    use crate::v8::{
        context_scope, isolate, isolate_scope, types, types::array, types::array_buffer,
        types::native_function_template, types::object, types::set, types::utf8, v8_init,
    };

    use v8_derive::new_native_function;

    static mut IS_INITIALIZED: bool = false;

    fn initialize() {
        unsafe {
            if !IS_INITIALIZED {
                v8_init(1);
                IS_INITIALIZED = true;
            }
        }
    }

    #[test]
    fn test_simple_init_destroy() {
        initialize();
    }

    #[test]
    fn test_simple_isolate_creation() {
        initialize();
        let isolate = isolate::Isolate::new();
        let _i_scope = isolate.enter();
    }

    #[test]
    fn test_simple_string_creation() {
        initialize();
        let isolate = isolate::Isolate::new();
        let isolate_scope = isolate.enter();
        let _s = isolate_scope.create_string("test");
    }

    #[test]
    fn test_simple_object_creation() {
        initialize();
        let isolate = isolate::Isolate::new();
        let isolate_scope = isolate.enter();
        let _o = isolate_scope.create_object_template();
    }

    #[test]
    fn test_simple_native_function_creation() {
        initialize();
        let isolate = isolate::Isolate::new();
        let isolate_scope = isolate.enter();
        let _o = isolate_scope.create_native_function_template(|_args, _isolate, _ctx_scope| {
            println!("test");
            None
        });
    }

    #[test]
    fn test_native_function_args() {
        initialize();
        let isolate = isolate::Isolate::new();
        let isolate_scope = isolate.enter();
        let native = isolate_scope
            .create_native_function_template(|args, _isolate_scope, _ctx_scope| {
                let v = args.get(0);
                let s: LocalUtf8 = v.try_into().unwrap();
                assert_eq!(s.as_str(), "2");
                None
            })
            .try_into()
            .unwrap();
        let native_funciton_name = isolate_scope.create_string("foo").try_into().unwrap();
        let mut globals: LocalObjectTemplate =
            isolate_scope.create_object_template().try_into().unwrap();
        globals.set_native_function(&native_funciton_name, &native);
        let code_str = isolate_scope.create_string("foo(2)").try_into().unwrap();
        let ctx = isolate_scope.create_context(Some(&globals));
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        script.run(&ctx_scope).unwrap();
    }

    #[test]
    fn test_native_function_call_js() {
        initialize();
        let isolate = isolate::Isolate::new();
        let isolate_scope = isolate.enter();

        let foo1 = isolate_scope
            .create_native_function_template(|args, _isolate, ctx_scope| {
                let v: LocalValueAny = args.get(0).try_into().unwrap();
                let _res = v.call(ctx_scope, None);
                None
            })
            .try_into()
            .unwrap();
        let foo1_name = isolate_scope.create_string("foo1").try_into().unwrap();

        let foo2 = isolate_scope
            .create_native_function_template(|args, _isolate_scope, _ctx_scope| {
                let v = args.get(0);
                let s: LocalUtf8 = v.try_into().unwrap();
                assert_eq!(s.as_str(), "2");
                None
            })
            .try_into()
            .unwrap();
        let foo2_name = isolate_scope.create_string("foo2").try_into().unwrap();

        let mut globals: LocalObjectTemplate =
            isolate_scope.create_object_template().try_into().unwrap();
        globals.set_native_function(&foo1_name, &foo1);
        globals.set_native_function(&foo2_name, &foo2);

        let code_str = isolate_scope
            .create_string("foo1(()=>{foo2(2)})")
            .try_into()
            .unwrap();
        let i_scope = isolate.enter();
        let ctx = i_scope.create_context(Some(&globals));
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        script.run(&ctx_scope).unwrap();
    }

    #[test]
    fn test_native_function_call_with_args() {
        initialize();
        let isolate = isolate::Isolate::new();
        let isolate_scope = isolate.enter();

        let foo1 = isolate_scope
            .create_native_function_template(|args, isolate_scope, ctx_scope| {
                let foo: LocalValueAny = isolate_scope.create_string("foo").try_into().unwrap();
                let v: LocalValueAny = args.get(0).try_into().unwrap();
                let _res = v.call(ctx_scope, Some(&[&foo.into()]));
                None
            })
            .try_into()
            .unwrap();
        let foo1_name = isolate_scope.create_string("foo1").try_into().unwrap();

        let foo2 = isolate_scope
            .create_native_function_template(|args, _isolate_scope, _ctx_scope| {
                let v = args.get(0);
                let s: LocalUtf8 = v.try_into().unwrap();
                assert_eq!(s.as_str(), "foo");
                None
            })
            .try_into()
            .unwrap();
        let foo2_name = isolate_scope.create_string("foo2").try_into().unwrap();

        let mut globals: LocalObjectTemplate =
            isolate_scope.create_object_template().try_into().unwrap();
        globals.set_native_function(&foo1_name, &foo1);
        globals.set_native_function(&foo2_name, &foo2);

        let code_str = isolate_scope
            .create_string("foo1((a)=>{foo2(a)})")
            .try_into()
            .unwrap();
        let ctx = isolate_scope.create_context(Some(&globals));
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        script.run(&ctx_scope).unwrap();
    }

    #[test]
    fn test_native_function_raise_exception() {
        initialize();
        let isolate = isolate::Isolate::new();
        let isolate_scope = isolate.enter();

        let native = isolate_scope
            .create_native_function_template(|_args, isolate, _ctx_scope| {
                isolate.raise_exception_str("this is an error");
                None
            })
            .try_into()
            .unwrap();
        let native_funciton_name = isolate_scope.create_string("foo").try_into().unwrap();
        let mut globals: LocalObjectTemplate =
            isolate_scope.create_object_template().try_into().unwrap();
        globals.set_native_function(&native_funciton_name, &native);
        let code_str = isolate_scope.create_string("foo(2)").try_into().unwrap();
        let ctx = isolate_scope.create_context(Some(&globals));
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        let trycatch: TryCatch = isolate_scope.create_try_catch().try_into().unwrap();
        assert!(script.run(&ctx_scope).is_none());
        let exception = trycatch.get_exception();
        let exception_msg = exception.into_utf8().unwrap();
        assert_eq!(exception_msg.as_str(), "this is an error");
    }

    #[test]
    fn test_native_function_raise_exception_error() {
        initialize();
        let isolate = isolate::Isolate::new();
        let isolate_scope = isolate.enter();
        let code_str = isolate_scope
            .create_string("function foo(){throw new Error('this is an error!');};foo();")
            .try_into()
            .unwrap();
        let ctx = isolate_scope.create_context(None);
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        let trycatch: TryCatch = isolate_scope.create_try_catch().try_into().unwrap();
        assert!(script.run(&ctx_scope).is_none());
        let exception = trycatch.get_exception();
        let exception_msg = exception.into_utf8().unwrap();
        assert_eq!(exception_msg.as_str(), "Error: this is an error!");
        let trace = trycatch.get_trace(&ctx_scope);
        let trace_str = trace.unwrap().into_utf8().unwrap();
        assert!(trace_str.as_str().contains("at foo"));
    }

    #[test]
    fn test_simple_code_run() {
        initialize();
        let isolate = isolate::Isolate::new();
        let isolate_scope = isolate.enter();
        let code_str = isolate_scope.create_string("1+1").try_into().unwrap();
        let ctx = isolate_scope.create_context(None);
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        let res = script.run(&ctx_scope).unwrap();
        let res_utf8: LocalUtf8 = res.try_into().unwrap();
        assert_eq!(res_utf8.as_str(), "2");
    }

    #[test]
    fn test_simple_module_run() {
        initialize();
        let isolate = isolate::Isolate::new();
        let isolate_scope = isolate.enter();

        let mut globals: LocalObjectTemplate =
            isolate_scope.create_object_template().try_into().unwrap();
        globals.add_native_function("log", |args, _isolate_scope, _ctx_scope| {
            assert_eq!(args.len(), 1);
            let v = args.get(0);
            let res_utf8: LocalUtf8 = v.try_into().unwrap();
            assert_eq!(res_utf8.as_str(), "foo");
            None
        });

        let code_name = isolate_scope
            .create_string("base_module")
            .try_into()
            .unwrap();
        let code_str = isolate_scope
            .create_string("import {msg} from \"foo\"; log(msg);")
            .try_into()
            .unwrap();
        let ctx = isolate_scope.create_context(Some(&globals));
        let ctx_scope = ctx.enter(&isolate_scope);

        let module = ctx_scope
            .compile_as_module(&code_name, &code_str, true)
            .unwrap();
        let module = module
            .initialize(
                &ctx_scope,
                |isolate_scope, ctx_scope, name_to_load, _identity_hash| {
                    let code_str = isolate_scope
                        .create_string("export let msg = \"foo\";")
                        .try_into()
                        .unwrap();
                    ctx_scope.compile_as_module(name_to_load, &code_str, true)
                },
            )
            .unwrap();
        let res = module.evaluate(&ctx_scope).unwrap();
        let res: LocalPromise = res.try_into().unwrap();
        assert_eq!(
            res.state().unwrap(),
            crate::v8::types::promise::PromiseState::Fulfilled
        );
    }

    #[test]
    fn test_async_function() {
        initialize();
        let isolate = isolate::Isolate::new();
        let isolate_scope = isolate.enter();
        let code_str = isolate_scope
            .create_string("async function f(){return 1}; f")
            .try_into()
            .unwrap();
        let ctx = isolate_scope.create_context(None);
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        let res = script.run(&ctx_scope).unwrap();
        let res: LocalValueAny = res.try_into().unwrap();
        assert!(res.is_async_function());
        let async_res = res.call(&ctx_scope, None).unwrap();
        let promise: LocalPromise = async_res.try_into().unwrap();
        assert_eq!(
            promise.state().unwrap(),
            crate::v8::types::promise::PromiseState::Fulfilled
        );
        let promise_res = promise.get_result();
        let res_utf8 = promise_res.into_utf8().unwrap();
        assert_eq!(res_utf8.as_str(), "1");
    }

    #[test]
    fn test_promise_resolver() {
        initialize();
        let isolate = isolate::Isolate::new();
        let isolate_scope = isolate.enter();
        let mut globals: LocalObjectTemplate =
            isolate_scope.create_object_template().try_into().unwrap();
        globals.add_native_function("foo", |_args, isolate_scope, ctx_scope| {
            let resolver = ctx_scope.create_resolver();
            resolver.resolve(
                &ctx_scope,
                &isolate_scope.create_string("foo").try_into().unwrap(),
            );
            let promise = resolver.get_promise();
            // let promise_val = promise.to_value();
            Some(promise.try_into().unwrap())
        });
        let code_str = isolate_scope.create_string("foo()").try_into().unwrap();
        let ctx = isolate_scope.create_context(Some(&globals));
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope
            .compile(&code_str)
            .expect("Couldn't compile script");
        let res = script.run(&ctx_scope).unwrap();
        let s: LocalUtf8 = res.clone().try_into().unwrap();
        println!("{}", s.as_str());
        let promise: LocalPromise = res.try_into().unwrap();
        assert_eq!(
            promise.state().unwrap(),
            crate::v8::types::promise::PromiseState::Fulfilled
        );
        let promise_res = promise.get_result();
        let res_utf8 = promise_res.into_utf8().unwrap();
        assert_eq!(res_utf8.as_str(), "foo");
    }

    #[test]
    fn test_compilation_error() {
        initialize();
        let isolate = isolate::Isolate::new();
        let isolate_scope = isolate.enter();
        let code_str = isolate_scope.create_string("foo(").try_into().unwrap();
        let ctx = isolate_scope.create_context(None);
        let ctx_scope = ctx.enter(&isolate_scope);
        let trycatch: TryCatch = isolate_scope.create_try_catch().try_into().unwrap();
        let script = ctx_scope.compile(&code_str);
        assert!(script.is_none());
        assert_eq!(
            trycatch.get_exception().into_utf8().unwrap().as_str(),
            "SyntaxError: Unexpected end of input"
        );
    }

    #[test]
    fn test_run_error() {
        initialize();
        let isolate = isolate::Isolate::new();
        let isolate_scope = isolate.enter();
        let code_str = isolate_scope.create_string("foo()").try_into().unwrap();
        let ctx = isolate_scope.create_context(None);
        let ctx_scope = ctx.enter(&isolate_scope);
        let trycatch: TryCatch = isolate_scope.create_try_catch().try_into().unwrap();
        let script = ctx_scope.compile(&code_str).unwrap();
        let res = script.run(&ctx_scope);
        assert!(res.is_none());
        assert_eq!(
            trycatch.get_exception().into_utf8().unwrap().as_str(),
            "ReferenceError: foo is not defined"
        );
    }

    fn define_function_and_call<
        F: for<'d, 'e> Fn(
            &types::native_function_template::LocalNativeFunctionArgs<'d, 'e>,
            &'d isolate_scope::IsolateScope<'e>,
            &context_scope::ContextScope<'d, 'e>,
        ) -> Option<LocalValueAny<'d, 'e>>,
    >(
        code: &str,
        func_name: &str,
        f: F,
    ) -> Result<(), String> {
        initialize();
        let isolate = isolate::Isolate::new();
        let isolate_scope = isolate.enter();
        let native: LocalNativeFunctionTemplate = isolate_scope
            .create_native_function_template(f)
            .try_into()
            .unwrap();
        let native_funciton_name = isolate_scope.create_string(func_name).try_into().unwrap();
        let mut globals: LocalObjectTemplate =
            isolate_scope.create_object_template().try_into().unwrap();
        globals.set_native_function(&native_funciton_name, &native);
        let code_str = isolate_scope.create_string(code).try_into().unwrap();
        let ctx = isolate_scope.create_context(Some(&globals));
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        let trycatch: TryCatch = isolate_scope.create_try_catch().try_into().unwrap();
        let res = match script.run(&ctx_scope) {
            Some(_res) => Ok(()),
            None => Err(trycatch
                .get_exception()
                .into_utf8()
                .unwrap()
                .as_str()
                .to_string()),
        };
        res
    }

    #[test]
    fn test_value_is_object() {
        define_function_and_call("foo({})", "foo", |args, _isolate, _ctx_scope| {
            if let Value::Object(_) = args.get(0) {
                assert!(true);
            } else {
                assert!(false, "The value should have been an object!");
            }
            None
        })
        .expect("Got error on function run");
    }

    #[test]
    fn test_value_is_function() {
        define_function_and_call("foo(()=>{})", "foo", |args, _isolate, _ctx_scope| {
            if let Value::Other(any) = args.get(0) {
                assert!(any.is_function());
            } else {
                assert!(false, "The value should have been an object!");
            }
            None
        })
        .expect("Got error on function run");
    }

    #[test]
    fn test_value_is_async_function() {
        define_function_and_call(
            "foo(async function(){})",
            "foo",
            |args, _isolate, _ctx_scope| {
                if let Value::Other(any) = args.get(0) {
                    assert!(any.is_async_function());
                } else {
                    assert!(false, "The value should have been an object!");
                }
                None
            },
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_value_is_string() {
        define_function_and_call("foo(\"foo\")", "foo", |args, _isolate, _ctx_scope| {
            if let Value::String(_) = args.get(0) {
                assert!(true);
            } else {
                assert!(false, "The value should have been a string!");
            }
            None
        })
        .expect("Got error on function run");
    }

    #[test]
    fn test_value_is_number() {
        define_function_and_call("foo(1)", "foo", |args, _isolate, _ctx_scope| {
            if let Value::Double(_) = args.get(0) {
                assert!(true);
            } else {
                assert!(false, "The value should have been a number!");
            }
            None
        })
        .expect("Got error on function run");
    }

    #[test]
    fn test_value_is_promise() {
        define_function_and_call(
            "foo(async function(){}())",
            "foo",
            |args, _isolate, _ctx_scope| {
                if let Value::Other(any) = args.get(0) {
                    assert!(any.is_promise());
                } else {
                    assert!(false, "The value should have been a number!");
                }
                None
            },
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_i64() {
        define_function_and_call(
            "test(1,2)",
            "test",
            new_native_function!(|_isolate, _ctx_scope, arg1: i64, arg2: i64| {
                assert_eq!(arg1, 1);
                assert_eq!(arg2, 2);
                Result::<Option<LocalValueAny>, String>::Ok(None)
            }),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_f64() {
        define_function_and_call(
            "test(1,2.2)",
            "test",
            new_native_function!(|_isolate, _ctx_scope, arg1: i64, arg2: f64| {
                assert_eq!(arg1, 1);
                assert_eq!(arg2, 2.2);
                Result::<Option<LocalValueAny>, String>::Ok(None)
            }),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_string() {
        define_function_and_call(
            "test(1,2.2,'test')",
            "test",
            new_native_function!(|_isolate, _ctx_scope, arg1: i64, arg2: f64, arg3: String| {
                assert_eq!(arg1, 1);
                assert_eq!(arg2, 2.2);
                assert_eq!(arg3, "test");
                Result::<Option<LocalValueAny>, String>::Ok(None)
            }),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_bool() {
        define_function_and_call(
            "test(1,2.2,true)",
            "test",
            new_native_function!(|_isolate, _ctx_scope, arg1: i64, arg2: f64, arg3: bool| {
                assert_eq!(arg1, 1);
                assert_eq!(arg2, 2.2);
                assert_eq!(arg3, true);
                Result::<Option<LocalValueAny>, String>::Ok(None)
            }),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_v8_local_utf8() {
        define_function_and_call(
            "test('test')",
            "test",
            new_native_function!(|_isolate, _ctx_scope, arg1: types::utf8::LocalUtf8| {
                assert_eq!(arg1.as_str(), "test");
                Result::<Option<LocalValueAny>, String>::Ok(None)
            }),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_v8_local_value() {
        define_function_and_call(
            "test('test')",
            "test",
            new_native_function!(|_isolate, _ctx_scope, arg1: Value| {
                let utf8 = LocalUtf8::try_from(arg1).unwrap();
                assert_eq!(utf8.as_ref(), "test");
                Result::<Option<LocalValueAny>, String>::Ok(None)
            }),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_v8_local_set() {
        define_function_and_call(
            "test(new Set())",
            "test",
            new_native_function!(|_isolate, _ctx_scope, _arg1: types::set::LocalSet| {
                Result::<Option<LocalValueAny>, String>::Ok(None)
            }),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_v8_local_array() {
        define_function_and_call(
            "test([1, 2])",
            "test",
            new_native_function!(|_isolate, _ctx_scope, arg1: types::array::LocalArray| {
                assert_eq!(arg1.len(), 2);
                Result::<Option<LocalValueAny>, String>::Ok(None)
            }),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_v8_local_array_buffer() {
        define_function_and_call(
            "test(new Uint8Array([255, 255, 255, 255]).buffer)",
            "test",
            new_native_function!(
                |_isolate, _ctx_scope, arg1: types::array_buffer::LocalArrayBuffer| {
                    assert_eq!(arg1.data(), &[255, 255, 255, 255]);
                    Result::<Option<LocalValueAny>, String>::Ok(None)
                }
            ),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_v8_local_object() {
        define_function_and_call(
            "test({'foo':'bar'})",
            "test",
            new_native_function!(
                |isolate_scope: &isolate_scope::IsolateScope,
                 ctx_scope,
                 arg1: types::object::LocalObject| {
                    let value = arg1
                        .get(
                            ctx_scope,
                            &isolate_scope.create_string("foo").try_into().unwrap(),
                        )
                        .unwrap();
                    let string = String::try_from(value).unwrap();
                    assert_eq!(&string, "bar");
                    Result::<Option<LocalValueAny>, String>::Ok(None)
                }
            ),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_wrong_args_count() {
        let err = define_function_and_call(
            "test(1)",
            "test",
            new_native_function!(|_isolate, _ctx_scope, arg1: i64, arg2: i64| {
                assert_eq!(arg1, 1);
                assert_eq!(arg2, 2);
                Result::<Option<LocalValueAny>, String>::Ok(None)
            }),
        )
        .expect_err("Did not get error when suppose to.");
        assert_eq!(
            err,
            "Can not convert value at position 1 into i64. Wrong number of arguments given."
        );
    }

    #[test]
    fn test_native_function_macro_wrong_arg_type() {
        let err = define_function_and_call(
            "test(1, 'foo')",
            "test",
            new_native_function!(|_isolate, _ctx_scope, arg1: i64, arg2: i64| {
                assert_eq!(arg1, 1);
                assert_eq!(arg2, 2);
                Result::<Option<LocalValueAny>, String>::Ok(None)
            }),
        )
        .expect_err("Did not get error when suppose to.");
        assert_eq!(
            err,
            "Can not convert value at position 1 into i64. Value is not long."
        );
    }

    #[test]
    fn test_native_function_macro_optional_arguments_not_exists() {
        let err = define_function_and_call(
            "test(1, 'foo')",
            "test",
            new_native_function!(|_isolate,
                                  _ctx_scope,
                                  arg1: i64,
                                  arg2: i64,
                                  arg3: Option<f64>| {
                assert_eq!(arg1, 1);
                assert_eq!(arg2, 2);
                assert_eq!(arg3, None);
                Result::<Option<LocalValueAny>, String>::Ok(None)
            }),
        )
        .expect_err("Did not get error when suppose to.");
        assert_eq!(
            err,
            "Can not convert value at position 1 into i64. Value is not long."
        );
    }

    #[test]
    fn test_native_function_macro_optional_arguments_exists() {
        let err = define_function_and_call(
            "test(1, 'foo', 2.2)",
            "test",
            new_native_function!(|_isolate,
                                  _ctx_scope,
                                  arg1: i64,
                                  arg2: i64,
                                  arg3: Option<f64>| {
                assert_eq!(arg1, 1);
                assert_eq!(arg2, 2);
                assert_eq!(arg3, Some(2.2));
                Result::<Option<LocalValueAny>, String>::Ok(None)
            }),
        )
        .expect_err("Did not get error when suppose to.");
        assert_eq!(
            err,
            "Can not convert value at position 1 into i64. Value is not long."
        );
    }

    #[test]
    fn test_native_function_macro_optional_arguments_object_not_exists() {
        let err = define_function_and_call(
            "test(1, 'foo', [1, 2])",
            "test",
            new_native_function!(
                |_isolate,
                 _ctx_scope,
                 arg1: i64,
                 arg2: i64,
                 arg3: Option<types::array::LocalArray>| {
                    assert_eq!(arg1, 1);
                    assert_eq!(arg2, 2);
                    assert!(arg3.is_none());
                    Result::<Option<LocalValueAny>, String>::Ok(None)
                }
            ),
        )
        .expect_err("Did not get error when suppose to.");
        assert_eq!(
            err,
            "Can not convert value at position 1 into i64. Value is not long."
        );
    }

    #[test]
    fn test_native_function_macro_optional_arguments_object() {
        let err = define_function_and_call(
            "test(1, 'foo', [1, 2])",
            "test",
            new_native_function!(
                |_isolate,
                 _ctx_scope,
                 arg1: i64,
                 arg2: i64,
                 arg3: Option<types::array::LocalArray>| {
                    assert_eq!(arg1, 1);
                    assert_eq!(arg2, 2);
                    if let Some(array) = arg3 {
                        assert_eq!(array.len(), 2);
                        assert!(true);
                    } else {
                        assert!(false, "Should have been an array.");
                    }
                    Result::<Option<LocalValueAny>, String>::Ok(None)
                }
            ),
        )
        .expect_err("Did not get error when suppose to.");
        assert_eq!(
            err,
            "Can not convert value at position 1 into i64. Value is not long."
        );
    }

    #[test]
    fn test_native_function_macro_optional_arguments_value() {
        let err =
            define_function_and_call(
                "test(1, 'foo', [1, 2])",
                "test",
                new_native_function!(
                    |_isolate, _ctx_scope, arg1: i64, arg2: i64, arg3: Option<types::Value>| {
                        assert_eq!(arg1, 1);
                        assert_eq!(arg2, 2);
                        if let Some(Value::Array(_)) = arg3 {
                            assert!(true);
                        } else {
                            assert!(false, "Should have been an array.");
                        }
                        Result::<Option<LocalValueAny>, String>::Ok(None)
                    }
                ),
            )
            .expect_err("Did not get error when suppose to.");
        assert_eq!(
            err,
            "Can not convert value at position 1 into i64. Value is not long."
        );
    }

    #[test]
    fn test_native_function_macro_consume_args() {
        define_function_and_call(
            "test(1, 'foo', [1, 2])",
            "test",
            new_native_function!(|_isolate, _ctx_scope, arg: Vec<types::Value>| {
                assert_eq!(arg.len(), 3);
                Result::<Option<LocalValueAny>, String>::Ok(None)
            }),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_consume_args_2() {
        define_function_and_call(
            "test(1, 'foo', [1, 2])",
            "test",
            new_native_function!(|_isolate, _ctx_scope, arg1: i64, arg2: Vec<types::Value>| {
                assert_eq!(arg1, 1);
                assert_eq!(arg2.len(), 2);
                Result::<Option<LocalValueAny>, String>::Ok(None)
            }),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_consume_args_error() {
        let err = define_function_and_call(
            "test(1, 'foo', [1, 2])",
            "test",
            new_native_function!(|_isolate, _ctx_scope, arg1: i64, arg2: Vec<i64>| {
                assert_eq!(arg1, 1);
                assert_eq!(arg2.len(), 2);
                Result::<Option<LocalValueAny>, String>::Ok(None)
            }),
        )
        .expect_err("Did not get error when suppose to.");
        assert_eq!(err, "Failed consuming arguments. Value is not long.");
    }
}
