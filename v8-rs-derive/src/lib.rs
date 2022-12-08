extern crate proc_macro;
use proc_macro::TokenStream;
use quote::ToTokens;
use quote::quote;
use syn;
use syn::ExprClosure;
use syn::TypePath;

#[proc_macro]
pub fn new_native_function(item: TokenStream) -> TokenStream {
    let ast: ExprClosure = syn::parse(item).expect("Can not parse token as closure");
    let is_move = ast.capture;
    let mut res = ast.clone();
    res.capture = None;
    
    let mut names = Vec::new();
    let mut min_index = 0;
    let mut max_index = 0;
    let mut types = Vec::new();
    let mut types_str = Vec::new();
    let mut types_for_closure = Vec::new();
    let inputs = ast.inputs.into_iter();
    let inputs = inputs.skip(2); // skip the isolate and ctx_scope
    for input in inputs {
        let input = match input {
            syn::Pat::Type(input) => input,
            _ => panic!("Given argument type is not supported"),
        };
        if input.pat.to_token_stream().to_string() == "__callback__" {
            panic!("__callback__ argument name is not allowed")
        }
        names.push(input.pat.to_token_stream());
        let input_type = match input.ty.as_ref() {
            syn::Type::Path(t) => t,
            _ => panic!("Given argument do not have proper type"),
        };
        types.push(input_type.to_token_stream());
        let type_str = input_type.to_token_stream().to_string();
        types_str.push(type_str.clone());
        let (type_str, is_option) = if type_str.starts_with("Option <") {
            (&type_str[8 .. type_str.len() - 1], true)
        } else {
            (type_str.as_str(), false)
        };
        let type_for_closure = if type_str.contains("V8") {
            format!("{}<'i_s, 'i>", type_str)
        } else {
            type_str.to_string()
        };
        let type_for_closure = if is_option {
            format!("Option<{}>", type_for_closure)
        } else {
            if min_index < max_index {
                panic!("All optional arguments must appear at the end.");
            }
            min_index += 1;
            type_for_closure
        };
        types_for_closure.push(syn::parse_str::<TypePath>(&type_for_closure).unwrap().to_token_stream());
        max_index += 1;
    }

    let max_args_len = max_index;
    let min_args_len = min_index;

    let mut get_argument_code = Vec::new();
    for i in 0 .. min_args_len {
        let t = types_str.get(i).unwrap();
        get_argument_code.push(quote!{match __args.get(#i).into(){
            Ok(r) => r,
            Err(e) => {
                __isolate.raise_exception_str(&format!("Can not convert value at position {} into {}. {}.", #i, #t, e));
                return None
            }
        }});
    }

    for i in min_args_len .. max_args_len {
        let t = types_str.get(i).unwrap();
        get_argument_code.push(quote!{if #i < __args.len() {Some(match __args.get(#i).into(){
            Ok(r) => r,
            Err(e) => {
                __isolate.raise_exception_str(&format!("Can not convert value at position {} into {}. {}.", #i, #t, e));
                return None
            }
        })} else {None}});
    }

    let gen = quote! {
        |__args, __isolate, __ctx_scope| {
            if __args.len() < #min_args_len || __args.len() > #max_args_len {
                __isolate.raise_exception_str(&format!("Worng number of argument given, expected at least {} or at most {} but got {}.", #min_args_len, #max_args_len, __args.len()));
                return None
            };

            #(
                let #names: #types = #get_argument_code;
            )*
            
            fn __create_closure__<F, E>(f: F) -> F
                where
                F: for<'i_s, 'i> Fn(&'i_s v8_rs::v8::isolate_scope::V8IsolateScope<'i>, &v8_rs::v8::v8_context_scope::V8ContextScope<'i_s, 'i>, #(#types_for_closure, )*) -> Result<Option<v8_rs::v8::v8_value::V8LocalValue<'i_s, 'i>>, E>,
                E: std::fmt::Display,
            {
                f
            }

            let __callback__ = __create_closure__(#res);
            let res = __callback__(__isolate, __ctx_scope, #(#names, )*);
            match res {
                Ok(res) => res,
                Err(e) => {
                    __isolate.raise_exception_str(&format!("{}", e));
                    None
                }
            }
        }
    };

    let mut ast: ExprClosure = syn::parse(gen.into()).unwrap();
    ast.capture = is_move;

    ast.into_token_stream().into()
}