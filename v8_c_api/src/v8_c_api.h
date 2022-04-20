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
typedef struct v8_isolate_scope v8_isolate_scope;
typedef struct v8_context v8_context;
typedef struct v8_handlers_scope v8_handlers_scope;
typedef struct v8_local_string v8_local_string;
typedef struct v8_local_native_function v8_local_native_function;
typedef struct v8_local_object v8_local_object;
typedef struct v8_local_script v8_local_script;
typedef struct v8_local_value v8_local_value;
typedef struct v8_local_value_arr v8_local_value_arr;
typedef struct v8_utf8_value v8_utf8_value;
typedef struct v8_persisted_value v8_persisted_value;

void v8_Initialize(v8_alloctor *allocator);
void v8_Despose();

v8_isolate* v8_NewIsolate();
void v8_FreeIsolate(v8_isolate* isolate);
v8_isolate_scope* v8_IsolateEnter(v8_isolate *v8_isolate);
void v8_IsolateExit(v8_isolate_scope *v8_isolate_scope);

v8_handlers_scope* v8_NewHandlersScope(v8_isolate *v8_isolate);
void v8_FreeHandlersScope(v8_handlers_scope* v8_handlersScope);

v8_context* v8_NewContext(v8_isolate* v8_isolate, v8_local_object *globals);
void v8_FreeContext(v8_context* ctx);
void v8_ContextEnter(v8_context *v8_ctx);
void v8_ContextExit(v8_context *v8_ctx);

v8_local_string* v8_NewString(v8_isolate* v8_isolate, const char *str, size_t len);
void v8_FreeString(v8_local_string *str);

typedef void (*native_funcion)(v8_local_value_arr *args, size_t len, void *pd);
v8_local_native_function* v8_NewNativeFunction(v8_isolate* v8_isolate, native_funcion func, void *pd);
void v8_FreeNativeFunction(v8_local_native_function *func);

v8_local_value* v8_ArgsGet(v8_local_value_arr *args, size_t i);

v8_local_object* v8_NewObject(v8_isolate* v8_isolate);
void v8_FreeObject(v8_local_object *obj);
void v8_ObjectSetFunction(v8_local_object *obj, v8_local_string *name, v8_local_native_function *f);

v8_local_script* v8_Compile(v8_context* v8_ctx, v8_local_string* str);
void v8_FreeScript(v8_local_script *script);

v8_local_value* v8_Run(v8_context* v8_ctx, v8_local_script* script);

void v8_FreeValue(v8_local_value *val);

v8_utf8_value* v8_ToUtf8(v8_isolate *isolate, v8_local_value *val);
void v8_FreeUtf8(v8_utf8_value *val);
const char* v8_Utf8PtrLen(v8_utf8_value *val, size_t *len);

#endif /* SRC_V8_C_API_H_ */
