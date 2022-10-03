#ifndef SRC_V8_C_API_H_
#define SRC_V8_C_API_H_

#include <stddef.h>

/* Allocator definition
 * Note: only structs memory will be allocated using the allocator,
 *       v8 memory will be allocate and manage by b8. */
typedef struct v8_alloctor {
	void* (*v8_Alloc)(size_t bytes);
	void* (*v8_Realloc)(void *ptr, size_t bytes);
	void  (*v8_Free)(void *ptr);
	void* (*v8_Calloc)(size_t nmemb, size_t size);
	char* (*v8_Strdup)(const char *str);
}v8_alloctor;

/* Opaque struct representing a v8 interpreter.
 * There is no limit to the amount of isolates that can be
 * created in a single processes. */
typedef struct v8_isolate v8_isolate;

/* Represent a scope to run JS code inside the isolate. */
typedef struct v8_isolate_scope v8_isolate_scope;

/* An isolate JS environment to run JS code.
 * There is no limit to the amount of contexts that can be
 * create in a single isolate. Each context has its own globals
 * separate from other contexts. It is only possible to run a single
 * contexts in a given time (in each isolate) */
typedef struct v8_context v8_context;

/* Represent a scope to run JS code inside the context. */
typedef struct v8_context_ref v8_context_ref;

/* Try catch scope, any error that will be raise during JS execution
 * will be catch by this object. */
typedef struct v8_trycatch v8_trycatch;

/* Responsible for all local handlers. When freed, all the locals
 * handlers that were manager by the handlers score will be freed. */
typedef struct v8_handlers_scope v8_handlers_scope;

/* JS String object */
typedef struct v8_local_string v8_local_string;

/* JS native function template */
typedef struct v8_local_native_function_template v8_local_native_function_template;

/* JS native function */
typedef struct v8_local_native_function v8_local_native_function;

/* JS native object template */
typedef struct v8_local_object_template v8_local_object_template;

/* JS native object */
typedef struct v8_local_object v8_local_object;

/* JS native set */
typedef struct v8_local_set v8_local_set;

/* JS native array*/
typedef struct v8_local_array v8_local_array;

/* JS native array buffer*/
typedef struct v8_local_array_buff v8_local_array_buff;

/* JS script object */
typedef struct v8_local_script v8_local_script;

typedef struct v8_persisted_script v8_persisted_script;

/* JS module object */
typedef struct v8_local_module v8_local_module;

/* JS persisted module object */
typedef struct v8_persisted_module v8_persisted_module;

/* JS generic value */
typedef struct v8_local_value v8_local_value;

/* JS promise object */
typedef struct v8_local_promise v8_local_promise;

/* JS promise resolver object */
typedef struct v8_local_resolver v8_local_resolver;

/* Represent a native function arguments */
typedef struct v8_local_value_arr v8_local_value_arr;

/* JS utf8 object */
typedef struct v8_utf8_value v8_utf8_value;

/* JS persisted object, can outlive the handlers score. */
typedef struct v8_persisted_value v8_persisted_value;

/* JS persisted object, can outlive the handlers score. */
typedef struct v8_unlocker v8_unlocker;

typedef void (*v8_InterruptCallback)(v8_isolate *isolate, void* data);

/* Initialize v8, must be called before any v8 API.
 * if allocator is NULL, use default memory functions. */
void v8_Initialize(v8_alloctor *allocator);

const char* v8_Version();

/* Dispose v8 initialization */
void v8_Dispose();

/* Create a new v8 isolate. An isolate is a v8 interpreter that responsible to run JS code.
 * Impeder may create as many isolates as wishes.
 * initial_heap_size_in_bytes - the initial isolate heap size
 * maximum_heap_size_in_bytes - maximum isolate heap size. when this value reached,
 * the isolate will try to perform GC. If GC do not help free the memory, the isolate
 * will abort the processes with OOM error. */
v8_isolate* v8_NewIsolate(size_t initial_heap_size_in_bytes, size_t maximum_heap_size_in_bytes);

/* Set fatal error handler, this method should write the error to some log file, when return the processes will exit */
void v8_IsolateSetFatalErrorHandler(v8_isolate* i, void (*fatal_hanlder)(const char* location, const char* message));

/* Set OOM error handler, this method should write the error to some log file, when return the processes will exit */
void v8_IsolateSetOOMErrorHandler(v8_isolate* i, void (*oom_hanlder)(const char* location, int is_heap_oom));

/* Set near OOM handler, the callback will be called when almost reaching OOM and allow to increase the max memory to avoid OOM error. */
void v8_IsolateSetNearOOMHandler(v8_isolate* i, size_t (*near_oom_callback)(void* data, size_t current_heap_limit, size_t initial_heap_limit), void *pd, void(*free_pd)(void*));

/* Return the currently used heap size */
size_t v8_IsolateUsedHeapSize(v8_isolate* i);

/* Return the currently total heap size */
size_t v8_IsolateTotalHeapSize(v8_isolate* i);

void v8_IsolateNotifyMemoryPressure(v8_isolate* i);

/* Return the currently used isolate or NULL if not isolate is in used */
v8_isolate* v8_IsolateGetCurrent();

/* Terminate the current JS code running on the given isolate */
void v8_TerminateCurrExecution(v8_isolate* i);

/* Cancel the termination triggered by v8_TerminateCurrExecution, after calling this API it is possible to continue using the isolate */
void v8_CancelTerminateExecution(v8_isolate* i);

/* Free the give isolate */
void v8_FreeIsolate(v8_isolate* isolate);

void v8_RequestInterrupt(v8_isolate* isolate, v8_InterruptCallback callback, void *data);

/* Enter the given isolate. This function should be called before running any JS
 * code on the isolate. */
v8_isolate_scope* v8_IsolateEnter(v8_isolate *v8_isolate);

/* Exit the isolate, after execute this function it is not allowed to run
 * any more JS on this isolate until `v8_IsolateEnter` is called again. */
void v8_IsolateExit(v8_isolate_scope *v8_isolate_scope);

/* Raise an exception, the given value will be treated as the exception value. */
void v8_IsolateRaiseException(v8_isolate *isolate, v8_local_value *value);

/* Get current run context from the of this isolate */
v8_context_ref* v8_GetCurrentCtxRef(v8_isolate *isolate);

void v8_IdleNotificationDeadline(v8_isolate *isolate, double deadline_in_seconds);

/* Create a new try catch object, any exception that will be raise during the JS execution
 * will be catch by this object. */
v8_trycatch* v8_NewTryCatch(v8_isolate *isolate);

/* Return the exception that was catch by the try catch object */
v8_local_value* v8_TryCatchGetException(v8_trycatch *trycatch);

/* Return true if the execution was terminated using v8_TerminateCurrExecution */
int v8_TryCatchHasTerminated(v8_trycatch *trycatch);

/* Free the try catch object */
void v8_FreeTryCatch(v8_trycatch *trycatch);

/* Create a new handlers scope, the handler scope is responsible to collect all local
 * handlers that was created while this object is alive. When freed, mark all the local
 * handlers that was collected for GC. */
v8_handlers_scope* v8_NewHandlersScope(v8_isolate *v8_isolate);

/* Free the given handlers crope */
void v8_FreeHandlersScope(v8_handlers_scope* v8_handlersScope);

/* Create a new JS context, a context is an isolate environment to run JS code.
 * A context has his own globals which are not shared with other contexts.
 * It is only possible to run a single context on a given time (per isolate). */
v8_context* v8_NewContext(v8_isolate* v8_isolate, v8_local_object_template *globals);

/* Free the given context */
void v8_FreeContext(v8_context* ctx);

/* Set a private data on the given context.
 * The private data can later be retrieve using `v8_GetPrivateData`. */
void v8_SetPrivateData(v8_context* ctx, size_t index, void *pd);

/* Return the private data that was set using `v8_SetPrivateData` or NULL
 * if no data was set on the given slot. */
void* v8_GetPrivateData(v8_context* ctx, size_t index);

/* Enter the given context, this function must be called befor running any
 * JS code on the given context. */
v8_context_ref* v8_ContextEnter(v8_context *v8_ctx);

v8_isolate* v8_ContextRefGetIsolate(v8_context_ref *v8_ctx_ref);

v8_local_object* v8_ContextRefGetGlobals(v8_context_ref *v8_ctx_ref);

/* Exit the JS context */
void v8_ExitContextRef(v8_context_ref *v8_ctx_ref);

/* Free the JS context */
void v8_FreeContextRef(v8_context_ref *v8_ctx_ref);

/* Same as `v8_GetPrivateData` but works on `v8_context_ref` */
void* v8_GetPrivateDataFromCtxRef(v8_context_ref* ctx_ref, size_t index);

/* Same as `v8_SetPrivateData` but works on `v8_context_ref` */
void v8_SetPrivateDataOnCtxRef(v8_context_ref* ctx_ref, size_t index, void *pd);

/* Create a new JS string object */
v8_local_string* v8_NewString(v8_isolate* v8_isolate, const char *str, size_t len);

/* Convert the JS string to JS generic value */
v8_local_value* v8_StringToValue(v8_local_string *str);

/* Convert the JS string to JS string object (same as writing 'new String(...)')*/
v8_local_object* v8_StringToStringObject(v8_isolate* v8_isolate, v8_local_string *str);

/* Free the given JS string */
void v8_FreeString(v8_local_string *str);

/* Native function callback definition */
typedef v8_local_value* (*native_funcion)(v8_local_value_arr *args, size_t len, void *pd);

/* Create a native function callback template */
v8_local_native_function_template* v8_NewNativeFunctionTemplate(v8_isolate* i, native_funcion func, void *pd, void(*freePD)(void *pd));

/* Create a native JS function from the given JS native function template */
v8_local_native_function* v8_NativeFunctionTemplateToFunction(v8_context_ref *ctx_ref, v8_local_native_function_template *func);

v8_local_native_function* v8_NewNativeFunction(v8_context_ref *ctx_ref, native_funcion func, void *pd, void(*freePD)(void *pd));

/* Free the given native function template */
void v8_FreeNativeFunctionTemplate(v8_local_native_function_template *func);

/* Free the given native function */
void v8_FreeNativeFunction(v8_local_native_function *func);

/* Convert the native function into a generic JS value */
v8_local_value* v8_NativeFunctionToValue(v8_local_native_function *func);

/* Return the i-th index from the native function arguments */
v8_local_value* v8_ArgsGet(v8_local_value_arr *args, size_t i);

/* Return current isolate from the native function arguments */
v8_isolate* v8_GetCurrentIsolate(v8_local_value_arr *args);

/* Create a new JS object template */
v8_local_object_template* v8_NewObjectTemplate(v8_isolate* v8_isolate);

/* Free the given JS object template */
void v8_FreeObjectTemplate(v8_local_object_template *obj);

/* Set a function template on the given object template at the given key */
void v8_ObjectTemplateSetFunction(v8_local_object_template *obj, v8_local_string *name, v8_local_native_function_template *f);

/* Set an object template on the given object template at the given key */
void v8_ObjectTemplateSetObject(v8_local_object_template *obj, v8_local_string *name, v8_local_object_template *o);

/* Set a generic JS value on the given object template at the given key */
void v8_ObjectTemplateSetValue(v8_local_object_template *obj, v8_local_string *name, v8_local_value *val);

/* Convert the given object template to a generic JS value */
v8_local_value* v8_ObjectTemplateToValue(v8_context_ref *ctx_ref, v8_local_object_template *obj);

/* Compile the given code into a script object */
v8_local_script* v8_Compile(v8_context_ref* v8_ctx_ref, v8_local_string* str);

v8_persisted_script* v8_ScriptPersist(v8_isolate *i, v8_local_script* script);

v8_local_script* v8_PersistedScriptToLocal(v8_isolate *i, v8_persisted_script* script);

void v8_FreePersistedScript(v8_persisted_script* script);

typedef v8_local_module* (*V8_LoadModuleCallback)(v8_context_ref* v8_ctx_ref, v8_local_string* name, int identity_hash);

/* Compile the given code as a module */
v8_local_module* v8_CompileAsModule(v8_context_ref* v8_ctx_ref, v8_local_string* name, v8_local_string* code, int is_module);

/* Initialize the module, return 1 on success and 0 on failure */
int v8_InitiateModule(v8_local_module* m, v8_context_ref* v8_ctx_ref, V8_LoadModuleCallback load_module_callback);

int v8_ModuleGetIdentityHash(v8_local_module* m);

/* Evaluate the module code */
v8_local_value* v8_EvaluateModule(v8_local_module* m, v8_context_ref* v8_ctx_ref);

v8_persisted_module* v8_ModulePersist(v8_isolate *i, v8_local_module* m);

v8_local_module* v8_ModuleToLocal(v8_isolate *i, v8_persisted_module* m);

void v8_FreePersistedModule(v8_persisted_module* m);

/* Evaluate the module code */
void v8_FreeModule(v8_local_module* m);

/* Free the given script object */
void v8_FreeScript(v8_local_script *script);

/* Run the given script object */
v8_local_value* v8_Run(v8_context_ref* v8_ctx_ref, v8_local_script* script);

/* Return 1 if the given JS value is a function and 0 otherwise */
int v8_ValueIsFunction(v8_local_value *val);

/* Invoke the given function */
v8_local_value* v8_FunctionCall(v8_context_ref *v8_ctx_ref, v8_local_value *val, size_t argc, v8_local_value* const* argv);

/* Return 1 if the given JS value is an async function and 0 otherwise */
int v8_ValueIsAsyncFunction(v8_local_value *val);

/* Return 1 if the given JS value is a string and 0 otherwise */
int v8_ValueIsString(v8_local_value *val);

/* Return 1 if the given JS value is a string object */
int v8_ValueIsStringObject(v8_local_value *val);

/* Convert the generic JS value into a JS string */
v8_local_string* v8_ValueAsString(v8_local_value *val);

v8_local_value* v8_ValueFromLong(v8_isolate *i, long long val);

/* Return 1 if the given JS value is a big integer and 0 otherwise */
int v8_ValueIsBigInt(v8_local_value *val);

long long v8_GetBigInt(v8_local_value *val);

/* Return 1 if the given JS value is a number and 0 otherwise */
int v8_ValueIsNumber(v8_local_value *val);

double v8_GetNumber(v8_local_value *val);

/* Return 1 if the given JS value is a number and 0 otherwise */
int v8_ValueIsBool(v8_local_value *val);

int v8_GetBool(v8_local_value *val);


v8_local_value* v8_ValueFromDouble(v8_isolate *i, double val);

/* Return 1 if the given JS value is a promise and 0 otherwise */
int v8_ValueIsPromise(v8_local_value *val);

/* Convert the generic JS value into a JS promise */
v8_local_promise* v8_ValueAsPromise(v8_local_value *val);

/* Return 1 if the given JS value is an object and 0 otherwise */
int v8_ValueIsObject(v8_local_value *val);

/* Return an array contains the propery names of the given object */
v8_local_array* v8_ValueGetPropertyNames(v8_context_ref *ctx_ref, v8_local_object *obj);

/* Return 1 if the given JS value is an array and 0 otherwise */
int v8_ValueIsArray(v8_local_value *val);

/* Return 1 if the given JS value is an array buffer and 0 otherwise */
int v8_ValueIsArrayBuffer(v8_local_value *val);

/* Create a new JS object */
v8_local_object* v8_NewObject(v8_isolate *i);

/* create a js object form json string */
v8_local_value* v8_NewObjectFromJsonString(v8_context_ref *ctx_ref, v8_local_string *str);

/* create a json string representation of a JS value */
v8_local_string* v8_JsonStringify(v8_context_ref *ctx_ref, v8_local_value *val);

/* Convert the generic JS value into a JS object */
v8_local_object* v8_ValueAsObject(v8_local_value *val);

/* Convert the generic JS value into a JS resolver*/
v8_local_resolver* v8_ValueAsResolver(v8_local_value *val);

/* Return the value of a given key from the given JS object */
v8_local_value* v8_ObjectGet(v8_context_ref *ctx_ref, v8_local_object *obj, v8_local_value *key);

/* Set a value inside the object at a given key */
void v8_ObjectSet(v8_context_ref *ctx_ref, v8_local_object *obj, v8_local_value *key, v8_local_value *val);

/* Freeze the object, same as Object.freeze. */
void v8_ObjectFreeze(v8_context_ref *ctx_ref, v8_local_object *obj);

/* Free the given JS object */
void v8_FreeObject(v8_local_object *obj);

/* Convert the given JS object into JS generic value */
v8_local_value* v8_ObjectToValue(v8_local_object *obj);

/* Create a new set */
v8_local_set* v8_NewSet(v8_isolate *i);

/* Add a value to the set */
void v8_SetAdd(v8_context_ref *ctx_ref, v8_local_set *set, v8_local_value *val);

/* Convert the given JS set into JS generic value */
v8_local_value* v8_SetToValue(v8_local_set *set);

/* Convert the generic JS value into a JS set */
v8_local_set* v8_ValueAsSet(v8_local_value *val);

/* Return 1 if the given JS value is a set and 0 otherwise */
int v8_ValueIsSet(v8_local_value *val);

/* Free the given JS set */
void v8_FreeSet(v8_local_set *set);

/* Create a new boolean */
v8_local_value* v8_NewBool(v8_isolate *i, int val);

/* Create a new JS null */
v8_local_value* v8_NewNull(v8_isolate *i);

/* Return 1 if the given JS value is null 0 otherwise */
int v8_ValueIsNull(v8_local_value *val);

/* Create a js ArrayBuffer */
v8_local_array_buff* v8_NewArrayBuffer(v8_isolate *i, const char *data, size_t len);

v8_local_value* v8_ArrayBufferToValue(v8_local_array_buff *arr_buffer);

/* Return the underline data of an array buffer */
const void* v8_ArrayBufferGetData(v8_local_array_buff *arr_buffer, size_t *len);

/* Free a js ArrayBuffer */
void v8_FreeArrayBuffer(v8_local_array_buff *arr_buffer);

v8_local_array* v8_NewArray(v8_isolate *i, v8_local_value *const *vals, size_t len);

/* Free the given JS array */
void v8_FreeArray(v8_local_array *arr);

size_t v8_ArrayLen(v8_local_array *arr);

v8_local_value* v8_ArrayGet(v8_context_ref *ctx_ref, v8_local_array *arr, size_t index);

v8_local_value* v8_ArrayToValue(v8_local_array *obj);

/* Convert the generic JS value into a JS array */
v8_local_array* v8_ValueAsArray(v8_local_value *val);

/* Convert the generic JS value into a JS array buffer */
v8_local_array_buff* v8_ValueAsArrayBuffer(v8_local_value *val);

/* Promise state */
typedef enum v8_PromiseState{
	v8_PromiseState_Unknown, v8_PromiseState_Fulfilled, v8_PromiseState_Rejected, v8_PromiseState_Pending
}v8_PromiseState;

/* Free the given promise object */
void v8_FreePromise(v8_local_promise* promise);

/* Return the state of the given promise object */
v8_PromiseState v8_PromiseGetState(v8_local_promise* promise);

/* Return the result of the given promise object
 * Only applicable when the promise state is v8_PromiseState_Fulfilled or v8_PromiseState_Rejected*/
v8_local_value* v8_PromiseGetResult(v8_local_promise* promise);

/* Set the promise fulfilled/rejected callbacks */
void v8_PromiseThen(v8_local_promise* promise, v8_context_ref *ctx_ref, v8_local_native_function *resolve, v8_local_native_function *reject);

/* Convert the given promise object into a generic JS value. */
v8_local_value* v8_PromiseToValue(v8_local_promise *promise);

/* Create a new resolver object */
v8_local_resolver* v8_NewResolver(v8_context_ref *ctx_ref);

/* Free the given resolver object */
void v8_FreeResolver(v8_local_resolver *resolver);

/* Return a promise object attached to this resolver */
v8_local_promise* v8_ResolverGetPromise(v8_local_resolver *resolver);

/* Resolve the resolver object with the given value */
void v8_ResolverResolve(v8_context_ref *ctx_ref, v8_local_resolver *resolver, v8_local_value *val);

/* Reject the resolver object with the given value */
void v8_ResolverReject(v8_context_ref *ctx_ref, v8_local_resolver *resolver, v8_local_value *val);

/* Convert the given resolver object into a generic JS value */
v8_local_value* v8_ResolverToValue(v8_local_resolver *resolver);

/* Persist the generic JS value, this function allows to escape the handlers scope and save
 * the given object for unlimited time with out worry about GC. */
v8_persisted_value* v8_PersistValue(v8_isolate *i, v8_local_value *val);

/* Turn the persisted value back to local value */
v8_local_value* v8_PersistedValueToLocal(v8_isolate *i, v8_persisted_value *val);

/* Free the given persisted value */
void v8_FreePersistedValue(v8_persisted_value *val);

/* Free the given generic JS value */
void v8_FreeValue(v8_local_value *val);

/* Convert the given generic JS value to utf8.
 * On failure, returns NULL.*/
v8_utf8_value* v8_ToUtf8(v8_isolate *isolate, v8_local_value *val);

/* Free the given uft8 object */
void v8_FreeUtf8(v8_utf8_value *val);

/* Return const pointer and length of the utf8 object */
const char* v8_Utf8PtrLen(v8_utf8_value *val, size_t *len);

/* Create an unlocker object that will unlock the v8 lock,
 * to re-aquire the lock the unlocker need to be freed */
v8_unlocker* v8_NewUnlocker(v8_isolate *i);

/* Free the unlocker and re-aquire the lock */
void v8_FreeUnlocker(v8_unlocker* unlocker);

#endif /* SRC_V8_C_API_H_ */
