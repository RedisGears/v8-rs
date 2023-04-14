/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_ContextRefGetIsolate, v8_EvaluateModule, v8_FreeModule, v8_FreePersistedModule,
    v8_InitiateModule, v8_ModuleGetIdentityHash, v8_ModulePersist, v8_ModuleToLocal,
    v8_context_ref, v8_local_module, v8_local_string, v8_persisted_module,
};
use crate::RawIndex;

use crate::v8::context_scope::ContextScope;
use crate::v8::isolate::Isolate;
use crate::v8::isolate_scope::IsolateScope;
use crate::v8::types::LocalString;
use crate::v8::types::LocalValueGeneric;
use std::os::raw::c_int;
use std::ptr;

/// JS script object
pub struct LocalModule<'isolate_scope, 'isolate> {
    pub(crate) inner_module: *mut v8_local_module,
    pub(crate) isolate_scope: &'isolate_scope IsolateScope<'isolate>,
}

pub struct PersistedModule {
    pub(crate) inner_persisted_module: *mut v8_persisted_module,
}

pub(crate) extern "C" fn load_module<
    T: for<'isolate, 'isolate_scope, 'c> Fn(
        &'isolate IsolateScope<'c>,
        &'isolate ContextScope<'isolate_scope, 'c>,
        &'isolate LocalString<'isolate_scope, 'c>,
        i64,
    ) -> Option<LocalModule<'isolate_scope, 'c>>,
>(
    v8_ctx_ref: *mut v8_context_ref,
    name: *mut v8_local_string,
    identity_hash: c_int,
) -> *mut v8_local_module {
    let isolate = Isolate {
        inner_isolate: unsafe { v8_ContextRefGetIsolate(v8_ctx_ref) },
        no_release: true,
    };
    let isolate_scope = IsolateScope::new(&isolate);
    let ctx_scope = ContextScope {
        inner_ctx_ref: v8_ctx_ref,
        exit_on_drop: false,
        isolate_scope: &isolate_scope,
    };
    let name_obj = LocalString {
        inner_string: name,
        isolate_scope: &isolate_scope,
    };
    let load_callback: &T = ctx_scope.get_private_data_mut_raw(RawIndex(0)).unwrap();
    let res = load_callback(&isolate_scope, &ctx_scope, &name_obj, identity_hash as i64);
    match res {
        Some(mut r) => {
            let inner_module = r.inner_module;
            r.inner_module = ptr::null_mut();
            inner_module
        }
        None => ptr::null_mut(),
    }
}

impl<'isolate_scope, 'isolate> LocalModule<'isolate_scope, 'isolate> {
    pub fn initialize<
        T: for<'c, 'd, 'e> Fn(
            &'c IsolateScope<'e>,
            &'c ContextScope<'d, 'e>,
            &'c LocalString<'d, 'e>,
            i64,
        ) -> Option<LocalModule<'d, 'e>>,
    >(
        &self,
        ctx_scope: &ContextScope,
        load_module_callback: T,
    ) -> bool {
        ctx_scope.set_private_data_raw(RawIndex(0), &load_module_callback);
        let res = unsafe {
            v8_InitiateModule(
                self.inner_module,
                ctx_scope.inner_ctx_ref,
                Some(load_module::<T>),
            )
        };
        ctx_scope.reset_private_data_raw(RawIndex(0));
        res != 0
    }

    pub fn evaluate(
        &self,
        ctx_scope: &ContextScope,
    ) -> Option<LocalValueGeneric<'isolate_scope, 'isolate>> {
        let res = unsafe { v8_EvaluateModule(self.inner_module, ctx_scope.inner_ctx_ref) };
        if res.is_null() {
            None
        } else {
            Some(LocalValueGeneric {
                inner_val: res,
                isolate_scope: self.isolate_scope,
            })
        }
    }

    /// Convert the module into a generic JS value
    #[must_use]
    pub fn persist(&self, isolate: &Isolate) -> PersistedModule {
        let inner_persisted_module =
            unsafe { v8_ModulePersist(isolate.inner_isolate, self.inner_module) };
        PersistedModule {
            inner_persisted_module,
        }
    }

    pub fn get_identity_hash(&self) -> i64 {
        unsafe { v8_ModuleGetIdentityHash(self.inner_module) as i64 }
    }
}

impl PersistedModule {
    pub fn to_local<'isolate_scope, 'isolate>(
        &self,
        isolate_scope: &'isolate_scope IsolateScope<'isolate>,
    ) -> LocalModule<'isolate_scope, 'isolate> {
        let inner_module = unsafe {
            v8_ModuleToLocal(
                isolate_scope.isolate.inner_isolate,
                self.inner_persisted_module,
            )
        };
        LocalModule {
            inner_module,
            isolate_scope,
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for LocalModule<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        if !self.inner_module.is_null() {
            unsafe { v8_FreeModule(self.inner_module) }
        }
    }
}

impl Drop for PersistedModule {
    fn drop(&mut self) {
        unsafe { v8_FreePersistedModule(self.inner_persisted_module) }
    }
}
