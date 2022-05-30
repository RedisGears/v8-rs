use crate::v8_c_raw::bindings::{v8_FreeUnlocker, v8_unlocker};

pub struct V8Unlocker {
    pub(crate) inner_unlocker: *mut v8_unlocker,
}

impl Drop for V8Unlocker {
    fn drop(&mut self) {
        unsafe { v8_FreeUnlocker(self.inner_unlocker) };
    }
}
