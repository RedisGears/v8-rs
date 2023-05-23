/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use std::borrow::Borrow;

use crate::v8_c_raw::bindings::{
    v8_FreePersistedScript, v8_FreePersistedValue, v8_FreeScript, v8_PersistedScriptToLocal,
    v8_Run, v8_ScriptPersist, v8_local_script, v8_local_string, v8_persisted_script,
};

use crate::v8::isolate_scope::V8IsolateScope;
use crate::v8::v8_context_scope::V8ContextScope;
use crate::v8::v8_value::V8LocalValue;

use super::v8_string::V8LocalString;
use super::v8_value::V8PersistValue;

/// JS script object
pub struct V8LocalScript<'isolate_scope, 'isolate> {
    pub(crate) inner_script: *mut v8_local_script,
    pub(crate) code: ScriptCode<'isolate_scope, 'isolate>,
    pub(crate) isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
}

/// A persisted script is a JavaScript-compiled code, which isn't tied
/// to the isolate it was compiled for. Hence it doesn't have any
/// lifetime boundaries and can live on its own, and converted into a
/// [V8LocalScript] later when it is required.
#[derive(Debug, Clone)]
pub struct V8PersistedScript {
    pub(crate) inner_persisted_script: *mut v8_persisted_script,
    /// The [ScriptCode] stored as a persisted value.
    pub(crate) code: String,
}

/// The JavaScript code that can be compiled into a [V8LocalScript].
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ScriptCode<'isolate_scope, 'isolate>(V8LocalString<'isolate_scope, 'isolate>);
impl<'isolate_scope, 'isolate> ScriptCode<'isolate_scope, 'isolate> {
    /// Creates a new [ScriptCode] object for the isolate used by the
    /// passed [V8ContextScope].
    pub fn new<T: AsRef<str>, I: Borrow<V8IsolateScope<'isolate>>>(
        code_string: T,
        isolate: &'isolate_scope I,
    ) -> Self {
        let isolate_scope: &V8IsolateScope = isolate.borrow();
        Self(isolate_scope.new_string(code_string.as_ref()))
    }

    /// Compiles the code into a script.
    pub fn compile(&self, ctx: &V8ContextScope<'isolate_scope, 'isolate>) -> Option<V8LocalScript> {
        ctx.compile(&self.0)
    }
}

impl<'isolate_scope, 'isolate> From<V8LocalString<'isolate_scope, 'isolate>>
    for ScriptCode<'isolate_scope, 'isolate>
{
    fn from(value: V8LocalString<'isolate_scope, 'isolate>) -> Self {
        Self(value)
    }
}

impl<'isolate_scope, 'isolate> V8LocalScript<'isolate_scope, 'isolate> {
    /// Run the script.
    pub fn run(&self, ctx: &V8ContextScope) -> Option<V8LocalValue<'isolate_scope, 'isolate>> {
        let inner_val = unsafe { v8_Run(ctx.inner_ctx_ref, self.inner_script) };
        if inner_val.is_null() {
            None
        } else {
            Some(V8LocalValue {
                inner_val,
                isolate_scope: self.isolate_scope,
            })
        }
    }

    /// Recompiles the script mutating the object (in-place).
    pub fn recompile(&mut self, ctx: &V8ContextScope<'isolate_scope, 'isolate>) -> bool {
        if let Some(script) = self.code.compile(ctx) {
            // unsafe { v8_FreeScript(self.inner_script) }
            self.inner_script = script.inner_script;
            true
        } else {
            false
        }
    }

    /// Persists the script by making it not tied to the isolate it was
    /// created for, allowing it to outlive it and not be bound to any
    /// lifetime.
    pub fn persist(&self) -> V8PersistedScript {
        let inner_persisted_script = unsafe {
            v8_ScriptPersist(self.isolate_scope.isolate.inner_isolate, self.inner_script)
        };
        let code = String::from(&self.code.0);
        V8PersistedScript {
            inner_persisted_script,
            code,
        }
    }
}

impl<'isolate_scope, 'isolate> From<V8LocalScript<'isolate_scope, 'isolate>> for V8PersistedScript {
    fn from(value: V8LocalScript<'isolate_scope, 'isolate>) -> Self {
        value.persist()
    }
}

impl V8PersistedScript {
    pub fn to_local<'isolate_scope, 'isolate>(
        &self,
        isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
    ) -> V8LocalScript<'isolate_scope, 'isolate> {
        let inner_script = unsafe {
            v8_PersistedScriptToLocal(
                isolate_scope.isolate.inner_isolate,
                self.inner_persisted_script,
            )
        };
        let code = ScriptCode::new(&self.code, isolate_scope);
        V8LocalScript {
            inner_script,
            code,
            isolate_scope,
        }
    }

    /// Returns the code of this script.
    pub fn get_script_code<'isolate_scope, 'isolate>(
        &self,
        isolate_scope: &'isolate_scope V8IsolateScope<'isolate>,
    ) -> ScriptCode<'isolate_scope, 'isolate> {
        ScriptCode::new(&self.code, isolate_scope)
    }

    /// Recompiles the script in-place.
    pub fn recompile<'isolate_scope, 'isolate>(
        &mut self,
        ctx: &V8ContextScope<'isolate_scope, 'isolate>,
    ) -> bool {
        let code = self.get_script_code(ctx.isolate_scope);
        let ret = if let Some(new_script) = code.compile(ctx) {
            let persisted_script = new_script.persist();
            unsafe {
                v8_FreePersistedScript(self.inner_persisted_script);
            }
            self.inner_persisted_script = persisted_script.inner_persisted_script;
            std::mem::forget(persisted_script);
            true
        } else {
            false
        };
        ret
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
