# v8-rs
Rust wrapper for v8

Example:

```rust
use crate::v8::*;

// initialized v8
v8_init();

// Create a new isolate
let isolate = isolate::V8Isolate::new();

// Create a new isolate scope
let _h_scope = isolate.new_handlers_scope();

// Enter the isolate
let i_scope = isolate.enter();

// Create the code string object
let code_str = isolate.new_string("1+1");

// Create a JS context for code invocation.
let ctx = i_scope.new_context(None);

// Enter the created context
let ctx_scope = ctx.enter();

// Compile the code
let script = ctx_scope.compile(&code_str).unwrap();

// Run the code
let res = script.run(&ctx_scope).unwrap();

// Get the result
let res_utf8 = res.to_utf8(&isolate).unwrap();
assert_eq!(res_utf8.as_str(), "2");
```
