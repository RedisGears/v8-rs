
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
        let script = ctx.compile(&code_str);
        script.run(&ctx_scope);
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
        let script = ctx.compile(&code_str);
        let res = script.run(&ctx_scope);
        let res_utf8 = res.to_utf8(&isolate);
        assert_eq!(res_utf8.as_str(), "2");
    }
}