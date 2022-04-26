pub mod v8;
mod v8_c_raw;

#[cfg(test)]
mod json_path_tests {
    use crate::v8::{isolate, v8_context_scope, v8_init, v8_native_function_template, v8_value};

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
        let _h_scope = isolate.new_handlers_scope();
        let _i_scope = isolate.enter();
    }

    #[test]
    fn test_simple_string_creation() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let _h_scope = isolate.new_handlers_scope();
        let _s = isolate.new_string("test");
    }

    #[test]
    fn test_simple_object_creation() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let _h_scope = isolate.new_handlers_scope();
        let _o = isolate.new_object_template();
    }

    #[test]
    fn test_simple_native_function_creation() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let _h_scope = isolate.new_handlers_scope();
        let _o = isolate.new_native_function_template(|_args, _isolate, _ctx_scope| {
            println!("test");
            None
        });
    }

    #[test]
    fn test_native_function_args() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let _h_scope = isolate.new_handlers_scope();
        let native = isolate.new_native_function_template(|args, isolate, _ctx_scope| {
            let v = args.get(0);
            let s = v.to_utf8(isolate).unwrap();
            assert_eq!(s.as_str(), "2");
            None
        });
        let native_funciton_name = isolate.new_string("foo");
        let mut globals = isolate.new_object_template();
        globals.set_native_function(&native_funciton_name, &native);
        let code_str = isolate.new_string("foo(2)");
        let i_scope = isolate.enter();
        let ctx = i_scope.new_context(Some(&globals));
        let ctx_scope = ctx.enter();
        let script = ctx_scope.compile(&code_str).unwrap();
        script.run(&ctx_scope).unwrap();
    }

    #[test]
    fn test_native_function_call_js() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let _h_scope = isolate.new_handlers_scope();

        let foo1 = isolate.new_native_function_template(|args, _isolate, ctx_scope| {
            let v = args.get(0);
            let _res = v.call(ctx_scope, None);
            None
        });
        let foo1_name = isolate.new_string("foo1");

        let foo2 = isolate.new_native_function_template(|args, isolate, _ctx_scope| {
            let v = args.get(0);
            let s = v.to_utf8(isolate).unwrap();
            assert_eq!(s.as_str(), "2");
            None
        });
        let foo2_name = isolate.new_string("foo2");

        let mut globals = isolate.new_object_template();
        globals.set_native_function(&foo1_name, &foo1);
        globals.set_native_function(&foo2_name, &foo2);

        let code_str = isolate.new_string("foo1(()=>{foo2(2)})");
        let i_scope = isolate.enter();
        let ctx = i_scope.new_context(Some(&globals));
        let ctx_scope = ctx.enter();
        let script = ctx_scope.compile(&code_str).unwrap();
        script.run(&ctx_scope).unwrap();
    }

    #[test]
    fn test_native_function_call_with_args() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let _h_scope = isolate.new_handlers_scope();

        let foo1 = isolate.new_native_function_template(|args, isolate, ctx_scope| {
            let _h_scope = isolate.new_handlers_scope();
            let foo = isolate.new_string("foo");
            let v = args.get(0);
            let _res = v.call(ctx_scope, Some(&[&foo.to_value()]));
            None
        });
        let foo1_name = isolate.new_string("foo1");

        let foo2 = isolate.new_native_function_template(|args, isolate, _ctx_scope| {
            let v = args.get(0);
            let s = v.to_utf8(isolate).unwrap();
            assert_eq!(s.as_str(), "foo");
            None
        });
        let foo2_name = isolate.new_string("foo2");

        let mut globals = isolate.new_object_template();
        globals.set_native_function(&foo1_name, &foo1);
        globals.set_native_function(&foo2_name, &foo2);

        let code_str = isolate.new_string("foo1((a)=>{foo2(a)})");
        let i_scope = isolate.enter();
        let ctx = i_scope.new_context(Some(&globals));
        let ctx_scope = ctx.enter();
        let script = ctx_scope.compile(&code_str).unwrap();
        script.run(&ctx_scope).unwrap();
    }

    #[test]
    fn test_native_function_raise_exception() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let _h_scope = isolate.new_handlers_scope();

        let native = isolate.new_native_function_template(|_args, isolate, _ctx_scope| {
            isolate.raise_exception_str("this is an error");
            None
        });
        let native_funciton_name = isolate.new_string("foo");
        let mut globals = isolate.new_object_template();
        globals.set_native_function(&native_funciton_name, &native);
        let code_str = isolate.new_string("foo(2)");
        let i_scope = isolate.enter();
        let ctx = i_scope.new_context(Some(&globals));
        let ctx_scope = ctx.enter();
        let script = ctx_scope.compile(&code_str).unwrap();
        let trycatch = isolate.new_try_catch();
        assert!(script.run(&ctx_scope).is_none());
        let exception = trycatch.get_exception();
        let exception_msg = exception.to_utf8(&isolate).unwrap();
        assert_eq!(exception_msg.as_str(), "this is an error");
    }

    #[test]
    fn test_simple_code_run() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let _h_scope = isolate.new_handlers_scope();
        let code_str = isolate.new_string("1+1");
        let i_scope = isolate.enter();
        let ctx = i_scope.new_context(None);
        let ctx_scope = ctx.enter();
        let script = ctx_scope.compile(&code_str).unwrap();
        let res = script.run(&ctx_scope).unwrap();
        let res_utf8 = res.to_utf8(&isolate).unwrap();
        assert_eq!(res_utf8.as_str(), "2");
    }

    #[test]
    fn test_async_function() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let _h_scope = isolate.new_handlers_scope();
        let code_str = isolate.new_string("async function f(){return 1}; f");
        let i_scope = isolate.enter();
        let ctx = i_scope.new_context(None);
        let ctx_scope = ctx.enter();
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
        let res_utf8 = promise_res.to_utf8(&isolate).unwrap();
        assert_eq!(res_utf8.as_str(), "1");
    }

    #[test]
    fn test_promise_resolver() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let _h_scope = isolate.new_handlers_scope();
        let mut globals = isolate.new_object_template();
        globals.add_native_function(&isolate, "foo", |_args, isolate, ctx_scope| {
            let resolver = ctx_scope.new_resolver();
            resolver.resolve(ctx_scope, &isolate.new_string("foo").to_value());
            Some(resolver.get_promise().to_value())
        });
        let code_str = isolate.new_string("foo()");
        let i_scope = isolate.enter();
        let ctx = i_scope.new_context(Some(&globals));
        let ctx_scope = ctx.enter();
        let script = ctx_scope.compile(&code_str).unwrap();
        let res = script.run(&ctx_scope).unwrap();
        println!("{}", res.to_utf8(&isolate).unwrap().as_str());
        assert!(res.is_promise());
        let promise = res.as_promise();
        assert_eq!(
            promise.state(),
            crate::v8::v8_promise::V8PromiseState::Fulfilled
        );
        let promise_res = promise.get_result();
        let res_utf8 = promise_res.to_utf8(&isolate).unwrap();
        assert_eq!(res_utf8.as_str(), "foo");
    }

    #[test]
    fn test_compilation_error() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let _h_scope = isolate.new_handlers_scope();
        let code_str = isolate.new_string("foo(");
        let i_scope = isolate.enter();
        let ctx = i_scope.new_context(None);
        let ctx_scope = ctx.enter();
        let trycatch = isolate.new_try_catch();
        let script = ctx_scope.compile(&code_str);
        assert!(script.is_none());
        assert_eq!(
            trycatch.get_exception().to_utf8(&isolate).unwrap().as_str(),
            "SyntaxError: Unexpected end of input"
        );
    }

    #[test]
    fn test_run_error() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let _h_scope = isolate.new_handlers_scope();
        let code_str = isolate.new_string("foo()");
        let i_scope = isolate.enter();
        let ctx = i_scope.new_context(None);
        let ctx_scope = ctx.enter();
        let trycatch = isolate.new_try_catch();
        let script = ctx_scope.compile(&code_str).unwrap();
        let res = script.run(&ctx_scope);
        assert!(res.is_none());
        assert_eq!(
            trycatch.get_exception().to_utf8(&isolate).unwrap().as_str(),
            "ReferenceError: foo is not defined"
        );
    }

    fn test_value_is_functions<
        F: Fn(
            &v8_native_function_template::V8LocalNativeFunctionArgs,
            &isolate::V8Isolate,
            &v8_context_scope::V8ContextScope,
        ) -> Option<v8_value::V8LocalValue>,
    >(
        code: &str,
        f: F,
    ) {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let _h_scope = isolate.new_handlers_scope();
        let native = isolate.new_native_function_template(f);
        let native_funciton_name = isolate.new_string("foo");
        let mut globals = isolate.new_object_template();
        globals.set_native_function(&native_funciton_name, &native);
        let code_str = isolate.new_string(code);
        let i_scope = isolate.enter();
        let ctx = i_scope.new_context(Some(&globals));
        let ctx_scope = ctx.enter();
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
