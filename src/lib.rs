/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! V8-rs is a crate containing bindings to the V8 C++ API.

// #![deny(missing_docs)]

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
    use crate::v8::v8_array::V8LocalArray;
    use crate::v8::{
        isolate, isolate_scope, v8_array, v8_array_buffer, v8_context_scope, v8_init,
        v8_native_function_template, v8_object, v8_set, v8_utf8,
        v8_value::{self},
    };
    use crate::{self as v8_rs};

    use v8_derive::{new_native_function, NativeFunctionArgument};

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
        let isolate = isolate::V8Isolate::new();
        let _i_scope = isolate.enter();
    }

    #[test]
    fn test_simple_string_creation() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();
        let _s = isolate_scope.new_string("test");
    }

    #[test]
    fn test_simple_object_creation() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();
        let _o = isolate_scope.new_object_template();
    }

    #[test]
    fn test_simple_native_function_creation() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();
        let _o = isolate_scope.new_native_function_template(|_args, _isolate, _ctx_scope| {
            println!("test");
            None
        });
    }

    #[test]
    fn test_native_function_args() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();
        let native =
            isolate_scope.new_native_function_template(|args, _isolate_scope, _ctx_scope| {
                let v = args.get(0);
                let s = v.to_utf8().unwrap();
                assert_eq!(s.as_str(), "2");
                None
            });
        let native_funciton_name = isolate_scope.new_string("foo");
        let mut globals = isolate_scope.new_object_template();
        globals.set_native_function(&native_funciton_name, &native);
        let code_str = isolate_scope.new_string("foo(2)");
        let ctx = isolate_scope.new_context(Some(&globals));
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        script.run(&ctx_scope).unwrap();
    }

    #[test]
    fn test_native_function_call_js() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();

        let foo1 = isolate_scope.new_native_function_template(|args, _isolate, ctx_scope| {
            let v = args.get(0);
            let _res = v.call(ctx_scope, None);
            None
        });
        let foo1_name = isolate_scope.new_string("foo1");

        let foo2 =
            isolate_scope.new_native_function_template(|args, _isolate_scope, _ctx_scope| {
                let v = args.get(0);
                let s = v.to_utf8().unwrap();
                assert_eq!(s.as_str(), "2");
                None
            });
        let foo2_name = isolate_scope.new_string("foo2");

        let mut globals = isolate_scope.new_object_template();
        globals.set_native_function(&foo1_name, &foo1);
        globals.set_native_function(&foo2_name, &foo2);

        let code_str = isolate_scope.new_string("foo1(()=>{foo2(2)})");
        let i_scope = isolate.enter();
        let ctx = i_scope.new_context(Some(&globals));
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        script.run(&ctx_scope).unwrap();
    }

    #[test]
    fn test_native_function_call_with_args() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();

        let foo1 = isolate_scope.new_native_function_template(|args, isolate_scope, ctx_scope| {
            let foo = isolate_scope.new_string("foo");
            let v = args.get(0);
            let _res = v.call(ctx_scope, Some(&[&foo.to_value()]));
            None
        });
        let foo1_name = isolate_scope.new_string("foo1");

        let foo2 =
            isolate_scope.new_native_function_template(|args, _isolate_scope, _ctx_scope| {
                let v = args.get(0);
                let s = v.to_utf8().unwrap();
                assert_eq!(s.as_str(), "foo");
                None
            });
        let foo2_name = isolate_scope.new_string("foo2");

        let mut globals = isolate_scope.new_object_template();
        globals.set_native_function(&foo1_name, &foo1);
        globals.set_native_function(&foo2_name, &foo2);

        let code_str = isolate_scope.new_string("foo1((a)=>{foo2(a)})");
        let ctx = isolate_scope.new_context(Some(&globals));
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        script.run(&ctx_scope).unwrap();
    }

    #[test]
    fn test_native_function_raise_exception() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();

        let native = isolate_scope.new_native_function_template(|_args, isolate, _ctx_scope| {
            isolate.raise_exception_str("this is an error");
            None
        });
        let native_funciton_name = isolate_scope.new_string("foo");
        let mut globals = isolate_scope.new_object_template();
        globals.set_native_function(&native_funciton_name, &native);
        let code_str = isolate_scope.new_string("foo(2)");
        let ctx = isolate_scope.new_context(Some(&globals));
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        let trycatch = isolate_scope.new_try_catch();
        assert!(script.run(&ctx_scope).is_none());
        let exception = trycatch.get_exception();
        let exception_msg = exception.to_utf8().unwrap();
        assert_eq!(exception_msg.as_str(), "this is an error");
    }

    #[test]
    fn test_native_function_raise_exception_error() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();
        let code_str = isolate_scope
            .new_string("function foo(){throw new Error('this is an error!');};foo();");
        let ctx = isolate_scope.new_context(None);
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        let trycatch = isolate_scope.new_try_catch();
        assert!(script.run(&ctx_scope).is_none());
        let exception = trycatch.get_exception();
        let exception_msg = exception.to_utf8().unwrap();
        assert_eq!(exception_msg.as_str(), "Error: this is an error!");
        let trace = trycatch.get_trace(&ctx_scope);
        let trace_str = trace.unwrap().to_utf8().unwrap();
        assert!(trace_str.as_str().contains("at foo"));
    }

    #[test]
    fn test_set_api() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();
        let code_str = isolate_scope.new_string("new Set([1, 2, 3, 4]);");
        let ctx = isolate_scope.new_context(None);
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        let res = script.run(&ctx_scope).unwrap();

        assert!(res.is_set());
        let arr: V8LocalArray = res.as_set().into();
        assert_eq!(arr.len(), 4);
        assert_eq!(
            arr.iter(&ctx_scope)
                .map(|v| v.get_long())
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4],
        );
    }

    #[test]
    fn test_delete_object_property() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();
        let ctx = isolate_scope.new_context(None);
        let ctx_scope = ctx.enter(&isolate_scope);

        let property_name = isolate_scope.new_string("foo").to_value();
        let object = isolate_scope.new_object();
        object.set(&ctx_scope, &property_name, &isolate_scope.new_long(1));

        assert_eq!(
            &object
                .get_own_property_names(&ctx_scope)
                .iter(&ctx_scope)
                .map(|v| v.to_utf8().unwrap().as_str().to_owned())
                .collect::<Vec<String>>(),
            &["foo".to_owned()]
        );

        assert!(object.delete(&ctx_scope, &property_name));

        assert!(object
            .get_own_property_names(&ctx_scope)
            .iter(&ctx_scope)
            .map(|v| v.to_utf8().unwrap().as_str().to_owned())
            .collect::<Vec<String>>()
            .is_empty());
    }

    #[test]
    fn test_simple_code_run() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();
        let code_str = isolate_scope.new_string("1+1");
        let ctx = isolate_scope.new_context(None);
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        let res = script.run(&ctx_scope).unwrap();
        let res_utf8 = res.to_utf8().unwrap();
        assert_eq!(res_utf8.as_str(), "2");
    }

    #[test]
    fn test_simple_module_run() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();

        let mut globals = isolate_scope.new_object_template();
        globals.add_native_function("log", |args, _isolate_scope, _ctx_scope| {
            assert_eq!(args.len(), 1);
            let v = args.get(0);
            let res_utf8 = v.to_utf8().unwrap();
            assert_eq!(res_utf8.as_str(), "foo");
            None
        });

        let code_name = isolate_scope.new_string("base_module");
        let code_str = isolate_scope.new_string("import {msg} from \"foo\"; log(msg);");
        let ctx = isolate_scope.new_context(Some(&globals));
        let ctx_scope = ctx.enter(&isolate_scope);

        let module = ctx_scope
            .compile_as_module(&code_name, &code_str, true)
            .unwrap();
        module.initialize(
            &ctx_scope,
            |isolate_scope, ctx_scope, name_to_load, _identity_hash| {
                let code_str = isolate_scope.new_string("export let msg = \"foo\";");
                ctx_scope.compile_as_module(name_to_load, &code_str, true)
            },
        );
        let res = module.evaluate(&ctx_scope).unwrap();
        let res = res.as_promise();
        assert_eq!(
            res.state(),
            crate::v8::v8_promise::V8PromiseState::Fulfilled
        );
    }

    #[test]
    fn test_async_function() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();
        let code_str = isolate_scope.new_string("async function f(){return 1}; f");
        let ctx = isolate_scope.new_context(None);
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        let res = script.run(&ctx_scope).unwrap();
        assert!(res.is_async_function());
        let async_res = res.call(&ctx_scope, None).unwrap();
        assert!(async_res.is_promise());
        let promise = async_res.as_promise();
        assert_eq!(
            promise.state(),
            crate::v8::v8_promise::V8PromiseState::Fulfilled
        );
        let promise_res = promise.get_result();
        let res_utf8 = promise_res.to_utf8().unwrap();
        assert_eq!(res_utf8.as_str(), "1");
    }

    #[test]
    fn test_promise_resolver() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();
        let mut globals = isolate_scope.new_object_template();
        globals.add_native_function("foo", |_args, isolate_scope, ctx_scope| {
            let resolver = ctx_scope.new_resolver();
            resolver.resolve(&ctx_scope, &isolate_scope.new_string("foo").to_value());
            let promise = resolver.get_promise();
            let promise_val = promise.to_value();
            Some(promise_val)
        });
        let code_str = isolate_scope.new_string("foo()");
        let ctx = isolate_scope.new_context(Some(&globals));
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        let res = script.run(&ctx_scope).unwrap();
        println!("{}", res.to_utf8().unwrap().as_str());
        assert!(res.is_promise());
        let promise = res.as_promise();
        assert_eq!(
            promise.state(),
            crate::v8::v8_promise::V8PromiseState::Fulfilled
        );
        let promise_res = promise.get_result();
        let res_utf8 = promise_res.to_utf8().unwrap();
        assert_eq!(res_utf8.as_str(), "foo");
    }

    #[test]
    fn test_compilation_error() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();
        let code_str = isolate_scope.new_string("foo(");
        let ctx = isolate_scope.new_context(None);
        let ctx_scope = ctx.enter(&isolate_scope);
        let trycatch = isolate_scope.new_try_catch();
        let script = ctx_scope.compile(&code_str);
        assert!(script.is_none());
        assert_eq!(
            trycatch.get_exception().to_utf8().unwrap().as_str(),
            "SyntaxError: Unexpected end of input"
        );
    }

    #[test]
    fn test_run_error() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();
        let code_str = isolate_scope.new_string("foo()");
        let ctx = isolate_scope.new_context(None);
        let ctx_scope = ctx.enter(&isolate_scope);
        let trycatch = isolate_scope.new_try_catch();
        let script = ctx_scope.compile(&code_str).unwrap();
        let res = script.run(&ctx_scope);
        assert!(res.is_none());
        assert_eq!(
            trycatch.get_exception().to_utf8().unwrap().as_str(),
            "ReferenceError: foo is not defined"
        );
    }

    fn define_function_and_call<
        F: for<'d, 'e> Fn(
            &v8_native_function_template::V8LocalNativeFunctionArgs<'d, 'e>,
            &'d isolate_scope::V8IsolateScope<'e>,
            &v8_context_scope::V8ContextScope<'d, 'e>,
        ) -> Option<v8_value::V8LocalValue<'d, 'e>>,
    >(
        code: &str,
        func_name: &str,
        f: F,
    ) -> Result<(), String> {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();
        let native = isolate_scope.new_native_function_template(f);
        let native_funciton_name = isolate_scope.new_string(func_name);
        let mut globals = isolate_scope.new_object_template();
        globals.set_native_function(&native_funciton_name, &native);
        let code_str = isolate_scope.new_string(code);
        let ctx = isolate_scope.new_context(Some(&globals));
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        let trycatch = isolate_scope.new_try_catch();
        let res = match script.run(&ctx_scope) {
            Some(_res) => Ok(()),
            None => Err(trycatch
                .get_exception()
                .to_utf8()
                .unwrap()
                .as_str()
                .to_string()),
        };
        res
    }

    #[test]
    fn test_value_is_object() {
        define_function_and_call("foo({})", "foo", |args, _isolate, _ctx_scope| {
            assert!(args.get(0).is_object());
            None
        })
        .expect("Got error on function run");
    }

    #[test]
    fn test_value_is_function() {
        define_function_and_call("foo(()=>{})", "foo", |args, _isolate, _ctx_scope| {
            assert!(args.get(0).is_function());
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
                assert!(args.get(0).is_async_function());
                None
            },
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_value_is_string() {
        define_function_and_call("foo(\"foo\")", "foo", |args, _isolate, _ctx_scope| {
            assert!(args.get(0).is_string());
            None
        })
        .expect("Got error on function run");
    }

    #[test]
    fn test_value_is_number() {
        define_function_and_call("foo(1)", "foo", |args, _isolate, _ctx_scope| {
            assert!(args.get(0).is_number());
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
                assert!(args.get(0).is_promise());
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
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
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
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
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
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
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
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
            }),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_v8_local_utf8() {
        define_function_and_call(
            "test('test')",
            "test",
            new_native_function!(|_isolate, _ctx_scope, arg1: v8_utf8::V8LocalUtf8| {
                assert_eq!(arg1.as_str(), "test");
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
            }),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_v8_local_value() {
        define_function_and_call(
            "test('test')",
            "test",
            new_native_function!(|_isolate, _ctx_scope, arg1: v8_value::V8LocalValue| {
                assert_eq!(arg1.to_utf8().unwrap().as_str(), "test");
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
            }),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_v8_local_set() {
        define_function_and_call(
            "test(new Set())",
            "test",
            new_native_function!(|_isolate, _ctx_scope, _arg1: v8_set::V8LocalSet| {
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
            }),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_v8_local_array() {
        define_function_and_call(
            "test([1, 2])",
            "test",
            new_native_function!(|_isolate, _ctx_scope, arg1: v8_array::V8LocalArray| {
                assert_eq!(arg1.len(), 2);
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
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
                |_isolate, _ctx_scope, arg1: v8_array_buffer::V8LocalArrayBuffer| {
                    assert_eq!(arg1.data(), &[255, 255, 255, 255]);
                    Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
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
                |isolate_scope: &isolate_scope::V8IsolateScope,
                 ctx_scope,
                 arg1: v8_object::V8LocalObject| {
                    assert_eq!(
                        arg1.get(ctx_scope, &isolate_scope.new_string("foo").to_value())
                            .unwrap()
                            .to_utf8()
                            .unwrap()
                            .as_str(),
                        "bar"
                    );
                    Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
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
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
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
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
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
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
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
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
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
                 arg3: Option<v8_array::V8LocalArray>| {
                    assert_eq!(arg1, 1);
                    assert_eq!(arg2, 2);
                    assert!(arg3.is_none());
                    Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
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
                 arg3: Option<v8_array::V8LocalArray>| {
                    assert_eq!(arg1, 1);
                    assert_eq!(arg2, 2);
                    assert_eq!(arg3.unwrap().len(), 2);
                    Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
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
        let err = define_function_and_call(
            "test(1, 'foo', [1, 2])",
            "test",
            new_native_function!(
                |_isolate,
                 _ctx_scope,
                 arg1: i64,
                 arg2: i64,
                 arg3: Option<v8_value::V8LocalValue>| {
                    assert_eq!(arg1, 1);
                    assert_eq!(arg2, 2);
                    assert_eq!(arg3.unwrap().is_array(), true);
                    Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
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
            new_native_function!(|_isolate, _ctx_scope, arg: Vec<v8_value::V8LocalValue>| {
                assert_eq!(arg.len(), 3);
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
            }),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_native_function_macro_consume_args_2() {
        define_function_and_call(
            "test(1, 'foo', [1, 2])",
            "test",
            new_native_function!(
                |_isolate, _ctx_scope, arg1: i64, arg2: Vec<v8_value::V8LocalValue>| {
                    assert_eq!(arg1, 1);
                    assert_eq!(arg2.len(), 2);
                    Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
                }
            ),
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
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
            }),
        )
        .expect_err("Did not get error when suppose to.");
        assert_eq!(err, "Failed consuming arguments. Value is not long.");
    }

    #[derive(NativeFunctionArgument, PartialEq, Eq, Debug)]
    struct InnerArgs {
        i: i64,
    }

    #[derive(NativeFunctionArgument, PartialEq, Eq, Debug)]
    struct Args {
        i: i64,
        s: String,
        b: bool,
        o: Option<String>,
        inner: InnerArgs,
        optional_inner: Option<InnerArgs>,
    }

    #[test]
    fn test_object_argument_macro() {
        define_function_and_call(
            "test({i: 1, s: 'foo', b: false, inner: { i: 10 }})",
            "test",
            new_native_function!(|_isolate, _ctx_scope, args: Args| {
                assert_eq!(
                    args,
                    Args {
                        i: 1,
                        s: "foo".to_owned(),
                        b: false,
                        o: None,
                        inner: InnerArgs { i: 10 },
                        optional_inner: None,
                    },
                );
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
            }),
        )
        .expect("Got error on function run");
    }

    #[test]
    fn test_error_on_object_argument_macro() {
        let err = define_function_and_call(
            "test({i: 1, s: 'foo', b: false })",
            "test",
            new_native_function!(|_isolate, _ctx_scope, args: Args| {
                assert_eq!(
                    args,
                    Args {
                        i: 1,
                        s: "foo".to_owned(),
                        b: false,
                        o: None,
                        inner: InnerArgs { i: 10 },
                        optional_inner: None,
                    },
                );
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
            }),
        )
        .expect_err("Did not get error when suppose to.");
        assert!(err.contains("Failed getting field inner, Given argument must be an object"));
    }

    #[test]
    fn test_extra_fields_on_object_argument_macro() {
        let err = define_function_and_call(
            "test({i: 1, s: 'foo', b: false, inner: { i: 10, extra: true }})",
            "test",
            new_native_function!(|_isolate, _ctx_scope, args: Args| {
                assert_eq!(
                    args,
                    Args {
                        i: 1,
                        s: "foo".to_owned(),
                        b: false,
                        o: None,
                        inner: InnerArgs { i: 10 },
                        optional_inner: None,
                    },
                );
                Result::<Option<v8_value::V8LocalValue>, String>::Ok(None)
            }),
        )
        .expect_err("Did not get error when suppose to.");
        assert!(err.contains("Unknown properties given: extra"));
    }
}
