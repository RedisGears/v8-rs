pub mod v8;
mod v8_c_raw;

#[cfg(test)]
mod json_path_tests {
    use crate::v8::{
        isolate, isolate_scope, v8_context_scope, v8_init, v8_native_function_template, v8_value,
    };

    static mut IS_INITIALIZED: bool = false;

    fn initialize() {
        unsafe {
            if !IS_INITIALIZED {
                v8_init();
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

    fn test_value_is_functions<
        F: for<'d, 'e> Fn(
            &v8_native_function_template::V8LocalNativeFunctionArgs<'d, 'e>,
            &isolate_scope::V8IsolateScope<'e>,
            &v8_context_scope::V8ContextScope<'d, 'e>,
        ) -> Option<v8_value::V8LocalValue<'d, 'e>>,
    >(
        code: &str,
        f: F,
    ) {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let isolate_scope = isolate.enter();
        let native = isolate_scope.new_native_function_template(f);
        let native_funciton_name = isolate_scope.new_string("foo");
        let mut globals = isolate_scope.new_object_template();
        globals.set_native_function(&native_funciton_name, &native);
        let code_str = isolate_scope.new_string(code);
        let ctx = isolate_scope.new_context(Some(&globals));
        let ctx_scope = ctx.enter(&isolate_scope);
        let script = ctx_scope.compile(&code_str).unwrap();
        script.run(&ctx_scope).unwrap();
    }

    #[test]
    fn test_value_is_object() {
        test_value_is_functions("foo({})", |args, _isolate, _ctx_scope| {
            assert!(args.get(0).is_object());
            None
        });
    }

    #[test]
    fn test_value_is_function() {
        test_value_is_functions("foo(()=>{})", |args, _isolate, _ctx_scope| {
            assert!(args.get(0).is_function());
            None
        });
    }

    #[test]
    fn test_value_is_async_function() {
        test_value_is_functions("foo(async function(){})", |args, _isolate, _ctx_scope| {
            assert!(args.get(0).is_async_function());
            None
        });
    }

    #[test]
    fn test_value_is_string() {
        test_value_is_functions("foo(\"foo\")", |args, _isolate, _ctx_scope| {
            assert!(args.get(0).is_string());
            None
        });
    }

    #[test]
    fn test_value_is_number() {
        test_value_is_functions("foo(1)", |args, _isolate, _ctx_scope| {
            assert!(args.get(0).is_number());
            None
        });
    }

    #[test]
    fn test_value_is_promise() {
        test_value_is_functions("foo(async function(){}())", |args, _isolate, _ctx_scope| {
            assert!(args.get(0).is_promise());
            None
        });
    }
}
