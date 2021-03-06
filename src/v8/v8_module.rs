use crate::v8_c_raw::bindings::{
    v8_EvaluateModule, v8_FreeModule, v8_FreePersistedModule, v8_InitiateModule,
    v8_ModuleGetIdentityHash, v8_ModulePersist, v8_ModuleToLocal, v8_context_ref, v8_local_module,
    v8_local_string, v8_persisted_module,
};

use crate::v8::isolate::V8Isolate;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_string::V8LocalString;
use crate::v8::v8_value::V8LocalValue;
use std::os::raw::c_int;
use std::ptr;

/// JS script object
pub struct V8LocalModule {
    pub(crate) inner_module: *mut v8_local_module,
}

pub struct V8PersistedModule {
    pub(crate) inner_persisted_module: *mut v8_persisted_module,
}

pub(crate) extern "C" fn load_module<
    T: Fn(&V8Isolate, &V8ContextScope, &V8LocalString, i64) -> Option<V8LocalModule>,
>(
    v8_ctx_ref: *mut v8_context_ref,
    name: *mut v8_local_string,
    identity_hash: c_int,
) -> *mut v8_local_module {
    let ctx_scope = V8ContextScope {
        inner_ctx_ref: v8_ctx_ref,
        exit_on_drop: false,
    };
    let isolate = ctx_scope.get_isolate();
    let name_obj = V8LocalString { inner_string: name };
    let load_callback: &T = ctx_scope.get_private_data_mut_raw(0).unwrap();
    let res = load_callback(&isolate, &ctx_scope, &name_obj, identity_hash as i64);
    match res {
        Some(mut r) => {
            let inner_module = r.inner_module;
            r.inner_module = ptr::null_mut();
            inner_module
        }
        None => ptr::null_mut(),
    }
}

impl V8LocalModule {
    pub fn initialize<
        T: Fn(&V8Isolate, &V8ContextScope, &V8LocalString, i64) -> Option<V8LocalModule>,
    >(
        &self,
        ctx_scope: &V8ContextScope,
        load_module_callback: T,
    ) -> bool {
        ctx_scope.set_private_data_raw(0, Some(&load_module_callback));
        let res = unsafe {
            v8_InitiateModule(
                self.inner_module,
                ctx_scope.inner_ctx_ref,
                Some(load_module::<T>),
            )
        };
        ctx_scope.set_private_data_raw::<T>(0, None);
        if res != 0 {
            true
        } else {
            false
        }
    }

    pub fn evaluate(&self, ctx_scope: &V8ContextScope) -> Option<V8LocalValue> {
        let res = unsafe { v8_EvaluateModule(self.inner_module, ctx_scope.inner_ctx_ref) };
        if res.is_null() {
            None
        } else {
            Some(V8LocalValue { inner_val: res })
        }
    }

    /// Convert the module into a generic JS value
    #[must_use]
    pub fn persist(&self, isolate: &V8Isolate) -> V8PersistedModule {
        let inner_persisted_module =
            unsafe { v8_ModulePersist(isolate.inner_isolate, self.inner_module) };
        V8PersistedModule {
            inner_persisted_module,
        }
    }

    pub fn get_identity_hash(&self) -> i64 {
        unsafe { v8_ModuleGetIdentityHash(self.inner_module) as i64 }
    }
}

impl V8PersistedModule {
    pub fn to_local(&self, isolate: &V8Isolate) -> V8LocalModule {
        let inner_module =
            unsafe { v8_ModuleToLocal(isolate.inner_isolate, self.inner_persisted_module) };
        V8LocalModule { inner_module }
    }
}

impl Drop for V8LocalModule {
    fn drop(&mut self) {
        if !self.inner_module.is_null() {
            unsafe { v8_FreeModule(self.inner_module) }
        }
    }
}

impl Drop for V8PersistedModule {
    fn drop(&mut self) {
        unsafe { v8_FreePersistedModule(self.inner_persisted_module) }
    }
}
