
pub mod v8;
mod v8_c_raw;

#[cfg(test)]
mod json_path_tests {
    use crate::v8::*;

    static mut IS_INITIALIZED: bool = false;

    fn initialize() {
        unsafe{
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
        let h_scope = isolate.new_handlers_scope();
        let _s = h_scope.new_string("test");
    }

    #[test]
    fn test_simple_object_creation() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let h_scope = isolate.new_handlers_scope();
        let _o = h_scope.new_object();
    }

    #[test]
    fn test_simple_native_function_creation() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let h_scope = isolate.new_handlers_scope();
        let _o = h_scope.new_native_function(|_args| println!("test"));
    }

    #[test]
    fn test_native_function_args() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let h_scope = isolate.new_handlers_scope();
        let native = h_scope.new_native_function(|args| {
            let v = args.get(0);
            let s = v.to_utf8(&isolate);
            assert_eq!(s.as_str(), "2");
        });
        let native_funciton_name = h_scope.new_string("foo");
        let mut globals = h_scope.new_object();
        globals.set_native_function(&native_funciton_name, &native);
        let code_str = h_scope.new_string("foo(2)");
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
        let h_scope = isolate.new_handlers_scope();
        
        let foo1 = h_scope.new_native_function(|args| {
            let v = args.get(0);
            let ctx_scope = args.get_current_isolate().get_curr_context_scope();
            v.call(&ctx_scope);
        });
        let foo1_name = h_scope.new_string("foo1");
        
        let foo2 = h_scope.new_native_function(|args| {
            let v = args.get(0);
            let s = v.to_utf8(&isolate);
            assert_eq!(s.as_str(), "2");
        });
        let foo2_name = h_scope.new_string("foo2");

        let mut globals = h_scope.new_object();
        globals.set_native_function(&foo1_name, &foo1);
        globals.set_native_function(&foo2_name, &foo2);
        
        let code_str = h_scope.new_string("foo1(()=>{foo2(2)})");
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
        let h_scope = isolate.new_handlers_scope();
        
        let native = h_scope.new_native_function(|args| {
            let isolate = args.get_current_isolate();
            isolate.raise_exception_str("this is an error");
        });
        let native_funciton_name = h_scope.new_string("foo");
        let mut globals = h_scope.new_object();
        globals.set_native_function(&native_funciton_name, &native);
        let code_str = h_scope.new_string("foo(2)");
        let i_scope = isolate.enter();
        let ctx = i_scope.new_context(Some(&globals));
        let ctx_scope = ctx.enter();
        let script = ctx_scope.compile(&code_str).unwrap();
        let trycatch = isolate.new_try_catch();
        assert!(script.run(&ctx_scope).is_none());
        let exception = trycatch.get_exception();
        let exception_msg = exception.to_utf8(&isolate);
        assert_eq!(exception_msg.as_str(), "this is an error");
    }

    #[test]
    fn test_simple_code_run() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let h_scope = isolate.new_handlers_scope();
        let code_str = h_scope.new_string("1+1");
        let i_scope = isolate.enter();
        let ctx = i_scope.new_context(None);
        let ctx_scope = ctx.enter();
        let script = ctx_scope.compile(&code_str).unwrap();
        let res = script.run(&ctx_scope).unwrap();
        let res_utf8 = res.to_utf8(&isolate);
        assert_eq!(res_utf8.as_str(), "2");
    }

    #[test]
    fn test_compilation_error() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let h_scope = isolate.new_handlers_scope();
        let code_str = h_scope.new_string("foo(");
        let i_scope = isolate.enter();
        let ctx = i_scope.new_context(None);
        let ctx_scope = ctx.enter();
        let trycatch = isolate.new_try_catch();
        let script = ctx_scope.compile(&code_str);
        assert!(script.is_none());
        assert_eq!(trycatch.get_exception().to_utf8(&isolate).as_str(), "SyntaxError: Unexpected end of input");
    }

    #[test]
    fn test_run_error() {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let h_scope = isolate.new_handlers_scope();
        let code_str = h_scope.new_string("foo()");
        let i_scope = isolate.enter();
        let ctx = i_scope.new_context(None);
        let ctx_scope = ctx.enter();
        let trycatch = isolate.new_try_catch();
        let script = ctx_scope.compile(&code_str).unwrap();
        let res = script.run(&ctx_scope);
        assert!(res.is_none());
        assert_eq!(trycatch.get_exception().to_utf8(&isolate).as_str(), "ReferenceError: foo is not defined");
    }

    fn test_value_is_functions<F:Fn(&v8_native_function::V8LocalNativeFunctionArgs)>(code: &str, f: F) {
        initialize();
        let isolate = isolate::V8Isolate::new();
        let h_scope = isolate.new_handlers_scope();
        let native = h_scope.new_native_function(f);
        let native_funciton_name = h_scope.new_string("foo");
        let mut globals = h_scope.new_object();
        globals.set_native_function(&native_funciton_name, &native);
        let code_str = h_scope.new_string(code);
        let i_scope = isolate.enter();
        let ctx = i_scope.new_context(Some(&globals));
        let ctx_scope = ctx.enter();
        let script = ctx_scope.compile(&code_str).unwrap();
        script.run(&ctx_scope).unwrap();
    }

    #[test]
    fn test_value_is_object() {
        test_value_is_functions("foo({})", |args| {
            assert!(args.get(0).is_object());
        })
    }

    #[test]
    fn test_value_is_function() {
        test_value_is_functions("foo(()=>{})", |args| {
            assert!(args.get(0).is_function());
        })
    }

    #[test]
    fn test_value_is_async_function() {
        test_value_is_functions("foo(async function(){})", |args| {
            assert!(args.get(0).is_async_function());
        })
    }

    #[test]
    fn test_value_is_string() {
        test_value_is_functions("foo(\"foo\")", |args| {
            assert!(args.get(0).is_string());
        })
    }

    #[test]
    fn test_value_is_number() {
        test_value_is_functions("foo(1)", |args| {
            assert!(args.get(0).is_number());
        })
    }

    #[test]
    fn test_value_is_promise() {
        test_value_is_functions("foo(async function(){}())", |args| {
            assert!(args.get(0).is_promise());
        })
    }
}