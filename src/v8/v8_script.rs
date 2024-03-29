/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use crate::v8_c_raw::bindings::{
    v8_FreePersistedScript, v8_FreeScript, v8_PersistedScriptToLocal, v8_Run, v8_ScriptPersist,
    v8_local_script, v8_persisted_script,
};

use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_value::V8LocalValue;

use super::isolate::IsolateId;

/// JS script object
#[derive(Debug)]
pub struct V8LocalScript<'isolate_scope, 'isolate> {
    pub(crate) inner_script: *mut v8_local_script,
    pub(crate) isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
}

/// A persisted script is a JavaScript-compiled code, which isn't tied
/// to the isolate it was compiled for. Hence it doesn't have any
/// lifetime boundaries and can live on its own, and converted into a
/// [V8LocalScript] later when it is required.
#[derive(Debug)]
pub struct V8PersistedScript {
    pub(crate) inner_persisted_script: *mut v8_persisted_script,
    /// The ID of the isolate this persisted script was created from.
    pub(crate) isolate_id: IsolateId,
}

impl<'isolate_scope, 'isolate> V8LocalScript<'isolate_scope, 'isolate> {
    /// Run the script.
    pub fn run(&self, ctx: &V8ContextScope) -> Option<V8LocalValue<'isolate_scope, 'isolate>> {
        let inner_val = unsafe { v8_Run(ctx.get_inner(), self.inner_script) };
        if inner_val.is_null() {
            None
        } else {
            Some(V8LocalValue {
                inner_val,
                isolate_scope: self.isolate_scope,
            })
        }
    }

    /// Persists the script by making it not tied to the isolate it was
    /// created for, allowing it to outlive it and not be bound to any
    /// lifetime.
    pub fn persist(&self) -> V8PersistedScript {
        let inner_persisted_script = unsafe {
            v8_ScriptPersist(self.isolate_scope.isolate.inner_isolate, self.inner_script)
        };

        let isolate_id = self
            .isolate_scope
            .isolate
            .get_id()
            .expect("Poisoned isolate");

        V8PersistedScript {
            inner_persisted_script,
            isolate_id,
        }
    }
}

impl<'isolate_scope, 'isolate> From<V8LocalScript<'isolate_scope, 'isolate>> for V8PersistedScript {
    fn from(value: V8LocalScript<'isolate_scope, 'isolate>) -> Self {
        value.persist()
    }
}

impl V8PersistedScript {
    /// Converts this persisted script back into the local object to the
    /// passed isolate.
    pub fn to_local<'isolate_scope, 'isolate>(
        &self,
        isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
    ) -> Result<V8LocalScript<'isolate_scope, 'isolate>, &'static str> {
        if let Some(id) = isolate_scope.isolate.get_id() {
            if id != self.isolate_id {
                Err("The passed isolate is not the isolate this persisted script was created from.")
            } else {
                let inner_script = unsafe {
                    v8_PersistedScriptToLocal(
                        isolate_scope.isolate.inner_isolate,
                        self.inner_persisted_script,
                    )
                };

                Ok(V8LocalScript {
                    inner_script,
                    isolate_scope,
                })
            }
        } else {
            Err("The passed isolate is invalid.")
        }
    }
}

impl<'isolate_scope, 'isolate> Drop for V8LocalScript<'isolate_scope, 'isolate> {
    fn drop(&mut self) {
        unsafe { v8_FreeScript(self.inner_script) }
    }
}

unsafe impl Sync for V8PersistedScript {}
unsafe impl Send for V8PersistedScript {}

impl Drop for V8PersistedScript {
    fn drop(&mut self) {
        unsafe {
            v8_FreePersistedScript(self.inner_persisted_script);
        }
    }
}
