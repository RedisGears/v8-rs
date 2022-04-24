use crate::v8_c_raw::bindings::{
    v8_local_object,
    v8_FreeObject,
    v8_ObjectSetFunction,
    v8_ObjectSetObject,
    v8_ObjectSetValue,
};

use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_native_function::V8LocalNativeFunction;
use crate::v8::handler_scope::V8HandlersScope;
use crate::v8::v8_native_function::V8LocalNativeFunctionArgs;
use crate::v8::v8_value::V8LocalValue;

pub struct V8LocalObject {
    pub (crate) inner_obj: *mut v8_local_object,
}

impl V8LocalObject {
    pub fn set_native_function(&mut self, name: &V8LocalString, func: &V8LocalNativeFunction) {
        unsafe{v8_ObjectSetFunction(self.inner_obj, name.inner_string, func.inner_func)};
    }

    pub fn add_native_function<T:Fn(&V8LocalNativeFunctionArgs)>(&mut self, h_scope: &V8HandlersScope, name: &str, func: T) {
        let native_func = h_scope.new_native_function(func);
        let func_name = h_scope.new_string(name);
        self.set_native_function(&func_name, &native_func);
    }

    pub fn set_object(&mut self, name: &V8LocalString, obj: &V8LocalObject) {
        unsafe{v8_ObjectSetObject(self.inner_obj, name.inner_string, obj.inner_obj)};
    }

    pub fn add_object(&mut self, h_scope: &V8HandlersScope, name: &str, obj: &V8LocalObject) {
        let obj_name = h_scope.new_string(name);
        self.set_object(&obj_name, obj);
    }

    pub fn set_value(&mut self, name: &V8LocalString, obj: &V8LocalValue) {
        unsafe{v8_ObjectSetValue(self.inner_obj, name.inner_string, obj.inner_val)};
    }

    pub fn add_value(&mut self, h_scope: &V8HandlersScope, name: &str, obj: &V8LocalValue) {
        let val_name = h_scope.new_string(name);
        self.set_value(&val_name, obj);
    }
}

impl Drop for V8LocalObject {
    fn drop(&mut self) {
        unsafe {v8_FreeObject(self.inner_obj)}
    }
}
