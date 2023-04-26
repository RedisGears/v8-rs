# v8-rs
Rust wrapper for the [Google V8 JavaScript Engine](https://chromium.googlesource.com/v8/v8).

## Example:

```rust
use v8_rs::v8::*;

// Initialise the V8 engine:
v8_init(1);

// Create a new isolate:
let isolate = isolate::V8Isolate::new();

// Enter the isolate created:
let i_scope = isolate.enter();

// Create the code string object:
let code_str = i_scope.new_string("1+1");

// Create a JS execution context for code invocation:""
let ctx = i_scope.new_context(None);

// Enter the created execution context:
let ctx_scope = ctx.enter(&i_scope);

// Compile the code:
let script = ctx_scope.compile(&code_str).unwrap();

// Run the compiled code:
let res = script.run(&ctx_scope).unwrap();

// Get the result:
let res_utf8 = res.to_utf8().unwrap();
assert_eq!(res_utf8.as_str(), "2");
```

## Build Options

Usually, just adding the crate as a dependency in your project will be enough. That said it is possible to change the following build option using evironment variables.

* `V8_VERSION` - will change the default V8 version to use.
* `V8_UPDATE_HEADERS` - will update the V8 headers according to the set version, allow to also set the following options:
  * `V8_HEADERS_PATH` - control where to download the headers zip file, default `v8_c_api/libv8.include.zip`.
  * `V8_FORCE_DOWNLOAD_V8_HEADERS` - download the V8 headers zip file even if it is already exists.
  * `V8_HEADERS_URL` - url from where to download the V8 headers zip file.
* `V8_MONOLITH_PATH` - control where to download the V8 monolith, default `v8_c_api/libv8_monolith.a`
* `V8_FORCE_DOWNLOAD_V8_MONOLITH` - download the V8 monolith even if it is already exists.
* `V8_MONOLITH_URL` - url from where to download the V8 monolith file.
