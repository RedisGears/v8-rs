use crate::v8_c_raw::bindings::{
    v8_local_object_template,
    v8_FreeObjectTemplate,
    v8_ObjectTemplateSetFunction,
    v8_ObjectTemplateSetObject,
    v8_ObjectTemplateSetValue,
    v8_ObjectTemplateToValue,
};

use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_native_function_template::{
    V8LocalNativeFunctionTemplate,
    V8LocalNativeFunctionArgs
};
use crate::v8::isolate::V8Isolate;
use crate::v8::v8_value::V8LocalValue;
use crate::v8::v8_context_scope::V8ContextScope;

pub struct V8LocalObjectTemplate {
    pub (crate) inner_obj: *mut v8_local_object_template,
}

impl V8LocalObjectTemplate {
    pub fn set_native_function(&mut self, name: &V8LocalString, func: &V8LocalNativeFunctionTemplate) {
        unsafe{v8_ObjectTemplateSetFunction(self.inner_obj, name.inner_string, func.inner_func)};
    }

    pub fn add_native_function<T:Fn(&V8LocalNativeFunctionArgs, &V8Isolate, &V8ContextScope) -> Option<V8LocalValue>>(&mut self, isolate: &V8Isolate, name: &str, func: T) {
        let native_func = isolate.new_native_function_template(func);
        let func_name = isolate.new_string(name);
        self.set_native_function(&func_name, &native_func);
    }

    pub fn set_object(&mut self, name: &V8LocalString, obj: &V8LocalObjectTemplate) {
        unsafe{v8_ObjectTemplateSetObject(self.inner_obj, name.inner_string, obj.inner_obj)};
    }

    pub fn add_object(&mut self, isolate: &V8Isolate, name: &str, obj: &V8LocalObjectTemplate) {
        let obj_name = isolate.new_string(name);
        self.set_object(&obj_name, obj);
    }

    pub fn set_value(&mut self, name: &V8LocalString, obj: &V8LocalValue) {
        unsafe{v8_ObjectTemplateSetValue(self.inner_obj, name.inner_string, obj.inner_val)};
    }

    pub fn add_value(&mut self, isolate: &V8Isolate, name: &str, obj: &V8LocalValue) {
        let val_name = isolate.new_string(name);
        self.set_value(&val_name, obj);
    }

    pub fn to_value(&self, ctx_scope: &V8ContextScope) -> V8LocalValue{
        let innver_val = unsafe{v8_ObjectTemplateToValue(ctx_scope.inner_ctx_ref, self.inner_obj)};
        V8LocalValue {
            inner_val: innver_val,
        }
    }
}

impl Drop for V8LocalObjectTemplate {
    fn drop(&mut self) {
        unsafe {v8_FreeObjectTemplate(self.inner_obj)}
    }
}
