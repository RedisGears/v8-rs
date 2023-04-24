/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */
//! Contains the object template facilities.

use crate::v8_c_raw::bindings::{
    v8_FreeObjectTemplate, v8_FreePersistedObjectTemplate, v8_NewObjectTemplate,
    v8_ObjectTemplateNewInstance, v8_ObjectTemplatePersist, v8_ObjectTemplateSetFunction,
    v8_ObjectTemplateSetInternalFieldCount, v8_ObjectTemplateSetObject, v8_ObjectTemplateSetValue,
    v8_PersistedObjectTemplateToLocal, v8_local_object_template, v8_persisted_object_template,
};

use crate::v8::context_scope::ContextScope;
use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::native_function_template::{
    LocalNativeFunctionArgs, LocalNativeFunctionTemplate,
};
use crate::v8::types::object::LocalObject;
use crate::v8::types::string::LocalString;
use crate::v8::types::ScopedValue;

use super::any::LocalValueAny;

/// The JavaScript object template, which is essentially a JavaScript
/// class builder: allows to set children objects (as in a map), add
/// member functions.
#[derive(Debug, Clone)]
pub struct LocalObjectTemplate<'isolate_scope, 'isolate>(
    pub(crate) ScopedValue<'isolate_scope, 'isolate, v8_local_object_template>,
);

impl<'isolate_scope, 'isolate> LocalObjectTemplate<'isolate_scope, 'isolate> {
    /// Creates a new object template.
    pub fn new(isolate_scope: &'isolate_scope IsolateScope<'isolate>) -> Self {
        let inner_val = unsafe { v8_NewObjectTemplate(isolate_scope.isolate.inner_isolate) };
        LocalObjectTemplate(ScopedValue {
            inner_val,
            isolate_scope,
        })
    }
    /// Set a native function to the object template as a given key
    pub fn set_native_function(&mut self, name: &LocalString, func: &LocalNativeFunctionTemplate) {
        unsafe {
            v8_ObjectTemplateSetFunction(self.0.inner_val, name.0.inner_val, func.0.inner_val)
        };
    }

    /// Same as [set_native_function] but gets the key as &str and the
    /// native function as closure.
    pub fn add_native_function<
        T: for<'d, 'e> Fn(
            &LocalNativeFunctionArgs<'d, 'e>,
            &'d IsolateScope<'e>,
            &ContextScope<'d, 'e>,
        ) -> Option<LocalValueAny<'d, 'e>>,
    >(
        &mut self,
        name: &str,
        func: T,
    ) {
        let native_func = self
            .0
            .isolate_scope
            .create_native_function_template(func)
            .try_into()
            .unwrap();
        let func_name = self.0.isolate_scope.create_string(name).try_into().unwrap();
        self.set_native_function(&func_name, &native_func);
    }

    /// Set the given object to the object template on a given key
    pub fn set_object(&mut self, name: &LocalString, obj: &Self) {
        unsafe { v8_ObjectTemplateSetObject(self.0.inner_val, name.0.inner_val, obj.0.inner_val) };
    }

    /// Sets the number of internal fields for objects generated from
    /// this template.
    pub fn set_internal_field_count(&mut self, count: usize) {
        unsafe { v8_ObjectTemplateSetInternalFieldCount(self.0.inner_val, count) };
    }

    /// Same as [set_object] but gets the key as an [str] slice.
    pub fn add_object(&mut self, name: &str, obj: &Self) {
        let obj_name = self.0.isolate_scope.create_string(name).try_into().unwrap();
        self.set_object(&obj_name, obj);
    }

    /// Set a generic JS value into the object template as a given key
    pub fn set_value(&mut self, name: &LocalString, obj: &LocalValueAny) {
        unsafe { v8_ObjectTemplateSetValue(self.0.inner_val, name.0.inner_val, obj.0.inner_val) };
    }

    /// Same as [set_value] but gets the key as an [str] slice.
    pub fn add_value(&mut self, name: &str, obj: &LocalValueAny) {
        let val_name = self.0.isolate_scope.create_string(name).try_into().unwrap();
        self.set_value(&val_name, obj);
    }

    /// Convert the object template into a generic JS value
    #[must_use]
    pub fn new_instance(&self, ctx_scope: &ContextScope) -> LocalObject<'isolate_scope, 'isolate> {
        let inner_val =
            unsafe { v8_ObjectTemplateNewInstance(ctx_scope.inner_ctx_ref, self.0.inner_val) };
        LocalObject(ScopedValue {
            inner_val,
            isolate_scope: self.0.isolate_scope,
        })
    }

    /// Persists the [LocalObjectTemplate] by converting it into a
    /// [PersistedObjectTemplate] which can outlive its [IsolateScope].
    pub fn persist(&self) -> PersistedObjectTemplate {
        let inner_persist = unsafe {
            v8_ObjectTemplatePersist(self.0.isolate_scope.isolate.inner_isolate, self.0.inner_val)
        };
        PersistedObjectTemplate {
            inner_persisted_obj_template: inner_persist,
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalObjectTemplate<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeObjectTemplate(self.0.inner_val) }
    }
}

/// The same as [LocalObjectTemplate] but not associated with an
/// [IsolateScope].
pub struct PersistedObjectTemplate {
    pub(crate) inner_persisted_obj_template: *mut v8_persisted_object_template,
}

impl PersistedObjectTemplate {
    /// Converts the [PersistedObjectTemplate] into a
    /// [LocalObjectTemplate] within the provided [IsolateScope].
    pub fn to_local<'isolate_scope, 'isolate>(
        &self,
        isolate_scope: &'isolate_scope IsolateScope<'isolate>,
    ) -> LocalObjectTemplate<'isolate_scope, 'isolate> {
        let inner_val = unsafe {
            v8_PersistedObjectTemplateToLocal(
                isolate_scope.isolate.inner_isolate,
                self.inner_persisted_obj_template,
            )
        };
        LocalObjectTemplate(ScopedValue {
            inner_val,
            isolate_scope,
        })
    }
}

unsafe impl Sync for PersistedObjectTemplate {}
unsafe impl Send for PersistedObjectTemplate {}

impl Drop for PersistedObjectTemplate {
    fn drop(&mut self) {
        unsafe { v8_FreePersistedObjectTemplate(self.inner_persisted_obj_template) }
    }
}
