use crate::v8_c_raw::bindings::{
    v8_FreeTryCatch, v8_TryCatchGetException, v8_TryCatchHasTerminated, v8_trycatch,
};

use crate::v8::v8_value::V8LocalValue;

/// An object that responsible to catch any exception which raised
/// during the JS code invocation.
pub struct V8TryCatch {
    pub(crate) inner_trycatch: *mut v8_trycatch,
}

impl V8TryCatch {
    /// Return the exception that was raise during the JS code invocation.
    #[must_use]
    pub fn get_exception(&self) -> V8LocalValue {
        let inner_val = unsafe { v8_TryCatchGetException(self.inner_trycatch) };
        V8LocalValue { inner_val }
    }

    #[must_use]
    pub fn has_terminated(&self) -> bool {
        let res = unsafe { v8_TryCatchHasTerminated(self.inner_trycatch) };
        res > 0
    }
}

impl Drop for V8TryCatch {
    fn drop(&mut self) {
        unsafe { v8_FreeTryCatch(self.inner_trycatch) }
    }
}
