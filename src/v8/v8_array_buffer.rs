use crate::v8_c_raw::bindings::{
    v8_ArrayBufferGetData, v8_ArrayBufferToValue, v8_FreeArrayBuffer, v8_local_array_buff,
};

use crate::v8::v8_value::V8LocalValue;

/// JS object
pub struct V8LocalArrayBuffer {
    pub(crate) inner_array_buffer: *mut v8_local_array_buff,
}

impl V8LocalArrayBuffer {
    pub fn data(&self) -> &[u8] {
        let mut size = 0;
        let data =
            unsafe { v8_ArrayBufferGetData(self.inner_array_buffer, &mut size as *mut usize) };
        unsafe { std::slice::from_raw_parts(data.cast::<u8>(), size) }
    }

    pub fn to_value(&self) -> V8LocalValue {
        let inner_val = unsafe { v8_ArrayBufferToValue(self.inner_array_buffer) };
        V8LocalValue { inner_val }
    }
}

impl Drop for V8LocalArrayBuffer {
    fn drop(&mut self) {
        unsafe { v8_FreeArrayBuffer(self.inner_array_buffer) }
    }
}
