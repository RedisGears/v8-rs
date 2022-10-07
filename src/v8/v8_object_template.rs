use crate::v8_c_raw::bindings::{
    v8_FreeObjectTemplate, v8_ObjectTemplateSetFunction, v8_ObjectTemplateSetObject,
    v8_ObjectTemplateSetValue, v8_ObjectTemplateToValue, v8_local_object_template,
};

use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_native_function_template::{
    V8LocalNativeFunctionArgs, V8LocalNativeFunctionTemplate,
};
use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_value::V8LocalValue;

/// JS object template
pub struct V8LocalObjectTemplate<'isolate_scope, 'isolate> {
    pub(crate) inner_obj: *mut v8_local_object_template,
    pub(crate) isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> V8LocalObjectTemplate<'isolate_scope, 'isolate> {
    /// Set a native function to the object template as a given key
    pub fn set_native_function(
        &mut self,
        name: &V8LocalString,
        func: &V8LocalNativeFunctionTemplate,
    ) {
        unsafe { v8_ObjectTemplateSetFunction(self.inner_obj, name.inner_string, func.inner_func) };
    }

    /// Same as `set_native_function` but gets the key as &str and the native function as closure.
    pub fn add_native_function<
        T: for<'d, 'e> Fn(
            &V8LocalNativeFunctionArgs<'d, 'e>,
            &V8IsolateScope<'e>,
            &V8ContextScope<'d, 'e>,
        ) -> Option<V8LocalValue<'d, 'e>>,
    >(
        &mut self,
        name: &str,
        func: T,
    ) {
        let native_func = self.isolate_scope.new_native_function_template(func);
        let func_name = self.isolate_scope.new_string(name);
        self.set_native_function(&func_name, &native_func);
    }

    /// Set the given object to the object template on a given key
    pub fn set_object(&mut self, name: &V8LocalString, obj: &Self) {
        unsafe { v8_ObjectTemplateSetObject(self.inner_obj, name.inner_string, obj.inner_obj) };
    }

    /// Same as `set_object` but gets the key as &str
    pub fn add_object(&mut self, name: &str, obj: &Self) {
        let obj_name = self.isolate_scope.new_string(name);
        self.set_object(&obj_name, obj);
    }

    /// Set a generic JS value into the object template as a given key
    pub fn set_value(&mut self, name: &V8LocalString, obj: &V8LocalValue) {
        unsafe { v8_ObjectTemplateSetValue(self.inner_obj, name.inner_string, obj.inner_val) };
    }

    /// Same as `set_value` but gets the key as &str
    pub fn add_value(&mut self, name: &str, obj: &V8LocalValue) {
        let val_name = self.isolate_scope.new_string(name);
        self.set_value(&val_name, obj);
    }

    /// Convert the object template into a generic JS value
    #[must_use]
    pub fn to_value(&self, ctx_scope: &V8ContextScope) -> V8LocalValue<'isolate_scope, 'isolate> {
        let inner_val =
            unsafe { v8_ObjectTemplateToValue(ctx_scope.inner_ctx_ref, self.inner_obj) };
        V8LocalValue {
            inner_val: inner_val,
            isolate_scope: self.isolate_scope,
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for V8LocalObjectTemplate<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeObjectTemplate(self.inner_obj) }
    }
}
