use crate::v8_c_raw::bindings::{
    v8_FreeNativeFunction, v8_NativeFunctionToValue, v8_local_native_function,
};

use crate::v8::v8_value::V8LocalValue;

/// Native function object
pub struct V8LocalNativeFunction {
    pub(crate) inner_func: *mut v8_local_native_function,
}

impl V8LocalNativeFunction {
    /// Convert the native function into a JS generic value
    #[must_use]
    pub fn to_value(&self) -> V8LocalValue {
        let inner_val = unsafe { v8_NativeFunctionToValue(self.inner_func) };
        V8LocalValue { inner_val }
    }
}

impl Drop for V8LocalNativeFunction {
    fn drop(&mut self) {
        unsafe { v8_FreeNativeFunction(self.inner_func) }
    }
}
