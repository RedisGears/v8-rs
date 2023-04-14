/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_FreeObjectTemplate, v8_FreePersistedObjectTemplate, v8_ObjectTemplateNewInstance,
    v8_ObjectTemplatePersist, v8_ObjectTemplateSetFunction, v8_ObjectTemplateSetInternalFieldCount,
    v8_ObjectTemplateSetObject, v8_ObjectTemplateSetValue, v8_PersistedObjectTemplateToLocal,
    v8_local_object_template, v8_persisted_object_template,
};

use crate::v8::context_scope::ContextScope;
use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::native_function_template::{
    LocalNativeFunctionArgs, LocalNativeFunctionTemplate,
};
use crate::v8::types::object::LocalObject;
use crate::v8::types::string::LocalString;
use crate::v8::types::LocalValueGeneric;

/// JS object template
pub struct LocalObjectTemplate<'isolate_scope, 'isolate> {
    pub(crate) inner_obj: *mut v8_local_object_template,
    pub(crate) isolate_scope: &'isolate_scope IsolateScope<'isolate>,
}

impl<'isolate_scope, 'isolate> LocalObjectTemplate<'isolate_scope, 'isolate> {
    /// Set a native function to the object template as a given key
    pub fn set_native_function(&mut self, name: &LocalString, func: &LocalNativeFunctionTemplate) {
        unsafe { v8_ObjectTemplateSetFunction(self.inner_obj, name.inner_string, func.inner_func) };
    }

    /// Same as `set_native_function` but gets the key as &str and the native function as closure.
    pub fn add_native_function<
        T: for<'d, 'e> Fn(
            &LocalNativeFunctionArgs<'d, 'e>,
            &'d IsolateScope<'e>,
            &ContextScope<'d, 'e>,
        ) -> Option<LocalValueGeneric<'d, 'e>>,
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
    pub fn set_object(&mut self, name: &LocalString, obj: &Self) {
        unsafe { v8_ObjectTemplateSetObject(self.inner_obj, name.inner_string, obj.inner_obj) };
    }

    pub fn set_internal_field_count(&mut self, count: usize) {
        unsafe { v8_ObjectTemplateSetInternalFieldCount(self.inner_obj, count) };
    }

    /// Same as `set_object` but gets the key as &str
    pub fn add_object(&mut self, name: &str, obj: &Self) {
        let obj_name = self.isolate_scope.new_string(name);
        self.set_object(&obj_name, obj);
    }

    /// Set a generic JS value into the object template as a given key
    pub fn set_value(&mut self, name: &LocalString, obj: &LocalValueGeneric) {
        unsafe { v8_ObjectTemplateSetValue(self.inner_obj, name.inner_string, obj.inner_val) };
    }

    /// Same as `set_value` but gets the key as &str
    pub fn add_value(&mut self, name: &str, obj: &LocalValueGeneric) {
        let val_name = self.isolate_scope.new_string(name);
        self.set_value(&val_name, obj);
    }

    /// Convert the object template into a generic JS value
    #[must_use]
    pub fn new_instance(&self, ctx_scope: &ContextScope) -> LocalObject<'isolate_scope, 'isolate> {
        let inner_obj =
            unsafe { v8_ObjectTemplateNewInstance(ctx_scope.inner_ctx_ref, self.inner_obj) };
        LocalObject {
            inner_obj,
            isolate_scope: self.isolate_scope,
        }
    }

    pub fn persist(&self) -> V8PersistedObjectTemplate {
        let inner_persist = unsafe {
            v8_ObjectTemplatePersist(self.isolate_scope.isolate.inner_isolate, self.inner_obj)
        };
        V8PersistedObjectTemplate {
            inner_persisted_obj_template: inner_persist,
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalObjectTemplate<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeObjectTemplate(self.inner_obj) }
    }
}

pub struct V8PersistedObjectTemplate {
    pub(crate) inner_persisted_obj_template: *mut v8_persisted_object_template,
}

impl V8PersistedObjectTemplate {
    pub fn to_local<'isolate_scope, 'isolate>(
        &self,
        isolate_scope: &'isolate_scope IsolateScope<'isolate>,
    ) -> LocalObjectTemplate<'isolate_scope, 'isolate> {
        let inner_obj = unsafe {
            v8_PersistedObjectTemplateToLocal(
                isolate_scope.isolate.inner_isolate,
                self.inner_persisted_obj_template,
            )
        };
        LocalObjectTemplate {
            inner_obj,
            isolate_scope,
        }
    }
}

unsafe impl Sync for V8PersistedObjectTemplate {}
unsafe impl Send for V8PersistedObjectTemplate {}

impl Drop for V8PersistedObjectTemplate {
    fn drop(&mut self) {
        unsafe { v8_FreePersistedObjectTemplate(self.inner_persisted_obj_template) }
    }
}
