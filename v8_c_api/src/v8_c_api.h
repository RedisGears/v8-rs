#ifndef SRC_V8_C_API_H_
#define SRC_V8_C_API_H_

#include <stddef.h>

typedef struct v8_alloctor {
	void* (*v8_Alloc)(size_t bytes);
	void* (*v8_Realloc)(void *ptr, size_t bytes);
	void  (*v8_Free)(void *ptr);
	void* (*v8_Calloc)(size_t nmemb, size_t size);
	char* (*v8_Strdup)(const char *str);
}v8_alloctor;

typedef struct v8_isolate v8_isolate;
typedef struct v8_trycatch v8_trycatch;
typedef struct v8_isolate_scope v8_isolate_scope;
typedef struct v8_context v8_context;
typedef struct v8_context_ref v8_context_ref;
typedef struct v8_handlers_scope v8_handlers_scope;
typedef struct v8_local_string v8_local_string;
typedef struct v8_local_native_function_template v8_local_native_function_template;
typedef struct v8_local_native_function v8_local_native_function;
typedef struct v8_local_object_template v8_local_object_template;
typedef struct v8_local_object v8_local_object;
typedef struct v8_local_script v8_local_script;
typedef struct v8_local_value v8_local_value;
typedef struct v8_local_promise v8_local_promise;
typedef struct v8_local_resolver v8_local_resolver;
typedef struct v8_local_value_arr v8_local_value_arr;
typedef struct v8_utf8_value v8_utf8_value;
typedef struct v8_persisted_value v8_persisted_value;

typedef void (*v8_InterruptCallback)(v8_isolate *isolate, void* data);

void v8_Initialize(v8_alloctor *allocator);
void v8_Despose();

v8_isolate* v8_NewIsolate(size_t initial_heap_size_in_bytes, size_t maximum_heap_size_in_bytes);
void v8_FreeIsolate(v8_isolate* isolate);
void v8_RequestInterrupt(v8_isolate* isolate, v8_InterruptCallback callback, void *data);
v8_isolate_scope* v8_IsolateEnter(v8_isolate *v8_isolate);
void v8_IsolateExit(v8_isolate_scope *v8_isolate_scope);
void v8_IsolateRaiseException(v8_isolate *isolate, v8_local_value *value);
v8_context_ref* v8_GetCurrentCtxRef(v8_isolate *isolate);
void v8_IdleNotificationDeadline(v8_isolate *isolate, double deadline_in_seconds);

v8_trycatch* v8_NewTryCatch(v8_isolate *isolate);
v8_local_value* v8_TryCatchGetException(v8_trycatch *trycatch);
void v8_FreeTryCatch(v8_trycatch *trycatch);

v8_handlers_scope* v8_NewHandlersScope(v8_isolate *v8_isolate);
void v8_FreeHandlersScope(v8_handlers_scope* v8_handlersScope);

v8_context* v8_NewContext(v8_isolate* v8_isolate, v8_local_object_template *globals);
void v8_FreeContext(v8_context* ctx);
void v8_SetPrivateData(v8_context* ctx, size_t index, void *pd);
void* v8_GetPrivateData(v8_context* ctx, size_t index);
v8_context_ref* v8_ContextEnter(v8_context *v8_ctx);
void v8_FreeContextRef(v8_context_ref *v8_ctx_ref);
void* v8_GetPrivateDataFromCtxRef(v8_context_ref* ctx_ref, size_t index);

v8_local_string* v8_NewString(v8_isolate* v8_isolate, const char *str, size_t len);
v8_local_value* v8_StringToValue(v8_local_string *str);
void v8_FreeString(v8_local_string *str);

typedef v8_local_value* (*native_funcion)(v8_local_value_arr *args, size_t len, void *pd);
v8_local_native_function_template* v8_NewNativeFunctionTemplate(v8_isolate* v8_isolate, native_funcion func, void *pd);
v8_local_native_function* v8_NativeFunctionTemplateToFunction(v8_context_ref *ctx_ref, v8_local_native_function_template *func);
void v8_FreeNativeFunctionTemplate(v8_local_native_function_template *func);
void v8_FreeNativeFunction(v8_local_native_function *func);
v8_local_value* v8_NativeFunctionToValue(v8_local_native_function *func);

v8_local_value* v8_ArgsGet(v8_local_value_arr *args, size_t i);
v8_isolate* v8_GetCurrentIsolate(v8_local_value_arr *args);

v8_local_object_template* v8_NewObjectTemplate(v8_isolate* v8_isolate);
void v8_FreeObjectTemplate(v8_local_object_template *obj);
void v8_ObjectTemplateSetFunction(v8_local_object_template *obj, v8_local_string *name, v8_local_native_function_template *f);
void v8_ObjectTemplateSetObject(v8_local_object_template *obj, v8_local_string *name, v8_local_object_template *o);
void v8_ObjectTemplateSetValue(v8_local_object_template *obj, v8_local_string *name, v8_local_value *val);
v8_local_value* v8_ObjectTemplateToValue(v8_context_ref *ctx_ref, v8_local_object_template *obj);

v8_local_script* v8_Compile(v8_context_ref* v8_ctx_ref, v8_local_string* str);
void v8_FreeScript(v8_local_script *script);

v8_local_value* v8_Run(v8_context_ref* v8_ctx_ref, v8_local_script* script);

int v8_ValueIsFunction(v8_local_value *val);
v8_local_value* v8_FunctionCall(v8_context_ref *v8_ctx_ref, v8_local_value *val, size_t argc, v8_local_value* const* argv);
int v8_ValueIsAsyncFunction(v8_local_value *val);
int v8_ValueIsString(v8_local_value *val);
v8_local_string* v8_ValueAsString(v8_local_value *val);
int v8_ValueIsBigInt(v8_local_value *val);
int v8_ValueIsNumber(v8_local_value *val);
int v8_ValueIsPromise(v8_local_value *val);
v8_local_promise* v8_ValueAsPromise(v8_local_value *val);
int v8_ValueIsObject(v8_local_value *val);
v8_local_object* v8_ValueAsObject(v8_local_value *val);

v8_local_value* v8_ObjectGet(v8_context_ref *ctx_ref, v8_local_object *obj, v8_local_value *key);
void v8_FreeObject(v8_local_object *obj);
v8_local_value* v8_ObjectToValue(v8_local_object *obj);

typedef enum v8_PromiseState{
	v8_PromiseState_Unknown, v8_PromiseState_Fulfilled, v8_PromiseState_Rejected, v8_PromiseState_Pending
}v8_PromiseState;

void v8_FreePromise(v8_local_promise* promise);
v8_PromiseState v8_PromiseGetState(v8_local_promise* promise);
v8_local_value* v8_PromiseGetResult(v8_local_promise* promise);
void v8_PromiseThen(v8_local_promise* promise, v8_context_ref *ctx_ref, v8_local_native_function *resolve, v8_local_native_function *reject);
v8_local_value* v8_PromiseToValue(v8_local_promise *promise);

v8_local_resolver* v8_NewResolver(v8_context_ref *ctx_ref);
void v8_FreeResolver(v8_local_resolver *resolver);
v8_local_promise* v8_ResolverGetPromise(v8_local_resolver *resolver);
void v8_ResolverResolve(v8_context_ref *ctx_ref, v8_local_resolver *resolver, v8_local_value *val);
void v8_ResolverReject(v8_context_ref *ctx_ref, v8_local_resolver *resolver, v8_local_value *val);
v8_local_value* v8_ResolverToValue(v8_local_resolver *resolver);

v8_persisted_value* v8_PersistValue(v8_isolate *i, v8_local_value *val);
v8_local_value* v8_PersistedValueToLocal(v8_isolate *i, v8_persisted_value *val);
void v8_FreePersistedValue(v8_persisted_value *val);
void v8_FreeValue(v8_local_value *val);

v8_utf8_value* v8_ToUtf8(v8_isolate *isolate, v8_local_value *val);
void v8_FreeUtf8(v8_utf8_value *val);
const char* v8_Utf8PtrLen(v8_utf8_value *val, size_t *len);

#endif /* SRC_V8_C_API_H_ */
