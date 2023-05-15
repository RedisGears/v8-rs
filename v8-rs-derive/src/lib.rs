extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use quote::quote_spanned;
use quote::ToTokens;
use syn;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::Data;
use syn::DeriveInput;
use syn::ExprClosure;
use syn::Fields;
use syn::GenericArgument;
use syn::PathArguments;

/// This derive proc macro can be specified on a struct and provide the ability to automatically generate the
/// struct from the native function JS argument. It should be used along side the `new_native_function` proc
/// macro in the following maner:
///
/// ```rust,no_run,ignore
/// #[derive(NativeFunctionArgument)]
/// struct InnerArgs {
///     i: i64,
/// }

/// #[derive(NativeFunctionArgument)]
/// struct Args {
///     i: i64,
///     s: String,
///     b: bool,
///     o: Option<String>,
///     inner: InnerArgs,
///     optional_inner: Option<InnerArgs>,
/// }
///
/// let native_function = isolate_scope.new_native_function_template(new_native_function!(|_isolate, _ctx_scope, args: Args| { /* put your code here */});
/// ```
///
/// The above example will automatically generate a code that takes the argument given from the JS and translate it to `Args`.
///
/// This macro expect that the JS will pass a single JS object that matches the give struct. For example, the following
/// JS object will match our `Args` struct:
///
/// ```JS
/// {i: 1, s: 'foo', b: false, inner: {i: 10} }
/// ```
///
/// Notice that any optional field is not mandatory and will be set to `None` if not given.
///
/// And error will be raised if the given argument do not match the struct definition.
///
/// The following table demonstrate how JS objects are parsed into rust types:
///
/// | JS type        | rust type                 |
/// |----------------|---------------------------|
/// | `string`       | [String] | [V8LocalUtf8]  |
/// | `array_buffer` | [V8LocalArrayBuffer]      |
/// | `bool`         | [bool]                    |
/// | `big integer`  | [i64]                     |
/// | `number`       | [f64]                     |
/// | `array`        | [V8LocalArray]            |
/// | `map`          | [V8LocalObject]           |
/// | `set`          | [V8LocalSet]              |
///
#[proc_macro_derive(NativeFunctionArgument)]
pub fn object_argument(item: TokenStream) -> TokenStream {
    let struct_input: DeriveInput = parse_macro_input!(item);
    let struct_data = match struct_input.data {
        Data::Struct(s) => s,
        _ => {
            return syn::Error::new(
                struct_input.span(),
                "Input must be a struct. Not enum nor union",
            )
            .to_compile_error()
            .into()
        }
    };

    let fields = match struct_data.fields {
        Fields::Named(f) => f,
        _ => {
            return syn::Error::new(
                struct_data.fields.span(),
                "Struct must contains a names fieds.",
            )
            .to_compile_error()
            .into()
        }
    };

    let fields: Vec<_> = fields.named
        .into_iter()
        .map(|v| {
            let fname = v.ident;
            let fname_str = fname.to_token_stream().to_string();
            let t = v.ty;
            if t.to_token_stream().to_string().starts_with("Option") {
                // handle optional field
                quote! {
                    #fname: obj.pop_str_field(ctx_scope, #fname_str).map_or(Result::<#t, String>::Ok(None), |v| {
                        if v.is_null() || v.is_undefined() {
                            return Ok(None);
                        }
                        Ok(Some(v8_rs::v8::v8_value::V8CtxValue::new(&v, ctx_scope).try_into().map_err(|e| format!("Failed getting field {}, {}.", #fname_str, e))?))
                    })?
                }
            } else {
                quote! {
                    #fname: {
                        let field = obj.pop_str_field(ctx_scope, #fname_str).ok_or(stringify!(#fname was not given).to_owned())?;
                        if field.is_null() || field.is_undefined() {
                            return Err(format!("Field {} does not exists.", #fname_str));
                        }
                        v8_rs::v8::v8_value::V8CtxValue::new(&field, ctx_scope).try_into().map_err(|e| format!("Failed getting field {}, {}.", #fname_str, e))?
                    }
                }
            }
        })
        .collect();

    let struct_name = struct_input.ident;
    let generics = struct_input.generics;

    let gen = quote! {
        impl<'isolate_scope, 'isolate, 'ctx_scope, 'a> TryFrom<&mut v8_rs::v8::v8_native_function_template::V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'ctx_scope, 'a>> for #struct_name #generics {
            type Error = String;

            fn try_from(it: &mut v8_rs::v8::v8_native_function_template::V8LocalNativeFunctionArgsIter<'isolate_scope, 'isolate, 'ctx_scope, 'a>) -> Result<Self, Self::Error> {
                let ctx_scope = it.get_ctx_scope();
                let next_value = it.next().ok_or("Wrong number of arguments given".to_owned())?;
                if !next_value.is_object() {
                    return Err("Given argument must be an object".to_owned());
                }
                let obj = next_value.as_object();
                let res = #struct_name {
                    #(#fields,)*
                };

                let properties_left = obj.get_own_property_names(ctx_scope);
                if !properties_left.is_empty() {
                    let properties: Vec<_> = properties_left.iter(ctx_scope).map(|v| v.to_utf8().map(|v| v.as_str().to_owned()).unwrap_or("property name is not valid utf8".to_owned())).collect();
                    return Err(format!("Unknown properties given: {}", properties.join(",")));
                }

                Ok(res)
            }
        }

        impl<'isolate_scope, 'isolate, 'value, 'ctx_value> TryFrom<v8_rs::v8::v8_value::V8CtxValue<'isolate_scope, 'isolate, 'value, 'ctx_value>> for #struct_name #generics {
            type Error = String;

            fn try_from(ctx_value: v8_rs::v8::v8_value::V8CtxValue<'isolate_scope, 'isolate, 'value, 'ctx_value>) -> Result<Self, Self::Error> {
                let val = ctx_value.get_value();
                if !val.is_object() {
                    return Err("Given argument must be an object".to_owned());
                }
                let ctx_scope = ctx_value.get_ctx_scope();;
                let obj = val.as_object();
                let res = #struct_name {
                    #(#fields,)*
                };

                let properties_left = obj.get_own_property_names(ctx_scope);
                if !properties_left.is_empty() {
                    let properties: Vec<_> = properties_left.iter(ctx_scope).map(|v| v.to_utf8().map(|v| v.as_str().to_owned()).unwrap_or("property name is not valid utf8".to_owned())).collect();
                    return Err(format!("Unknown properties given: {}", properties.join(",")));
                }

                Ok(res)
            }
        }
    };

    gen.into()
}

#[proc_macro]
pub fn new_native_function(item: TokenStream) -> TokenStream {
    let ast: ExprClosure = match syn::parse(item) {
        Ok(res) => res,
        Err(e) => return e.to_compile_error().into(),
    };
    let is_move = ast.capture;
    let mut res = ast.clone();
    res.capture = None;

    let mut names = Vec::new();
    let mut min_index = 0;
    let mut max_index = 0;
    let mut types = Vec::new();
    let mut types_span = Vec::new();
    let mut types_str = Vec::new();
    let mut types_for_closure = Vec::new();
    let inputs = ast.inputs.into_iter();
    let inputs = inputs.skip(2); // skip the isolate and ctx_scope
    let mut consume_all_args = false;
    for input in inputs {
        if consume_all_args {
            return syn::Error::new(
                input.span(),
                "Can not add more arguments after consuming all arguments with a vector",
            )
            .to_compile_error()
            .into();
        }
        let input = match input {
            syn::Pat::Type(input) => input,
            _ => {
                return syn::Error::new(input.span(), "Given argument type is not supported")
                    .to_compile_error()
                    .into()
            }
        };
        if input.pat.to_token_stream().to_string() == "__callback__" {
            return syn::Error::new(input.span(), "__callback__ argument name is not allowed")
                .to_compile_error()
                .into();
        }
        names.push(input.pat.to_token_stream());
        let input_type = match input.ty.as_ref() {
            syn::Type::Path(t) => t,
            _ => {
                return syn::Error::new(input.span(), "Given argument do not have proper type")
                    .to_compile_error()
                    .into()
            }
        };
        types_span.push(input_type.span());
        types.push(input_type.to_token_stream());
        let type_str = input_type.to_token_stream().to_string();
        types_str.push(type_str.clone());
        let option_arg = match input_type.path.segments.last() {
            Some(v) => v,
            None => {
                return syn::Error::new(input_type.span(), "Failed parsing argument type")
                    .to_compile_error()
                    .into()
            }
        };
        let outer_type = option_arg.ident.to_token_stream().to_string();
        let type_str = if outer_type == "Option" || outer_type == "Vec" {
            let generic_types = match input_type.path.segments.last() {
                Some(res) => res,
                None => {
                    return syn::Error::new(
                        input_type.span(),
                        "Failed extracting Option internal type",
                    )
                    .to_compile_error()
                    .into()
                }
            };
            if let PathArguments::AngleBracketed(args) = &generic_types.arguments {
                let arg = match args.args.last() {
                    Some(res) => res,
                    None => {
                        return syn::Error::new(
                            input_type.span(),
                            "Failed extracting Option internal type",
                        )
                        .to_compile_error()
                        .into()
                    }
                };
                if let GenericArgument::Type(t) = arg {
                    if let syn::Type::Path(p) = t {
                        p
                    } else {
                        return syn::Error::new(
                            input_type.span(),
                            "Failed parse Option internal argument",
                        )
                        .to_compile_error()
                        .into();
                    }
                } else {
                    return syn::Error::new(
                        input_type.span(),
                        "Failed parse Option internal argument",
                    )
                    .to_compile_error()
                    .into();
                }
            } else {
                return syn::Error::new(input_type.span(), "Failed parse Option internal argument")
                    .to_compile_error()
                    .into();
            }
        } else {
            input_type
        };
        let type_for_closure = if type_str.to_token_stream().to_string().contains("V8") {
            quote_spanned! {input_type.span() => #type_str<'i_s, 'i>}
        } else {
            type_str.to_token_stream()
        };
        let type_for_closure = if outer_type == "Option" {
            quote_spanned! {input_type.span() => Option<#type_for_closure>}
        } else if outer_type == "Vec" {
            consume_all_args = true;
            quote_spanned! {input_type.span() => Vec<#type_for_closure>}
        } else {
            if min_index < max_index {
                return syn::Error::new(
                    input_type.span(),
                    "All optional arguments must appear at the end.",
                )
                .to_compile_error()
                .into();
            }
            min_index += 1;
            type_for_closure
        };
        match syn::parse::<syn::Type>(type_for_closure.clone().into()) {
            Err(e) => return syn::Error::new(input_type.span(), format!("Failed generating proper type with lifetime, generated type: '{}', error: '{}'", type_for_closure.clone().to_token_stream().to_string(), e.to_string())).to_compile_error().into(),
            _ => (),
        };
        types_for_closure.push(quote_spanned! {input_type.span() => #type_for_closure});
        if !consume_all_args {
            max_index += 1;
        }
    }

    let max_args_len = max_index;
    let min_args_len = min_index;

    let mut get_argument_code = Vec::new();
    for i in 0..min_args_len {
        let t = types_str.get(i).unwrap();
        get_argument_code.push(quote_spanned!{types_span.get(i).unwrap().clone() =>
            match (&mut __args_iter).try_into() {
                Ok(r) => r,
                Err(e) => {
                    __isolate.raise_exception_str(&format!("Can not convert value at position {} into {}. {}.", #i, #t, e));
                    return None
                }
            }
        });
    }

    for i in min_args_len..max_args_len {
        let t = types_str.get(i).unwrap();
        get_argument_code.push(quote_spanned!{types_span.get(i).unwrap().clone() =>
            match v8_rs::v8::OptionalTryFrom::optional_try_from(&mut __args_iter) {
                Ok(r) => r,
                Err(e) => {
                    __isolate.raise_exception_str(&format!("Can not convert value at position {} into {}. {}.", #i, #t, e));
                    return None
                }
            }
        });
    }

    if consume_all_args {
        get_argument_code.push(quote_spanned! {types_span.last().unwrap().clone() =>
            match (&mut __args_iter).try_into() {
                Ok(res) => res,
                Err(e) => {
                    __isolate.raise_exception_str(&format!("Failed consuming arguments. {}.", e));
                    return None
                }
            }
        });
    }

    let gen = quote! {
        |__args, __isolate, __ctx_scope| {

            let mut __args_iter = __args.iter(__ctx_scope);

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

    let mut ast: ExprClosure = match syn::parse(gen.into()) {
        Ok(res) => res,
        Err(e) => return e.to_compile_error().into(),
    };
    ast.capture = is_move;
    ast.into_token_stream().into()
}
