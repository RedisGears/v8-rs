#include "v8include/v8.h"
#include "v8include/libplatform/libplatform.h"

std::unique_ptr<v8::Platform> platform;

extern "C" {

#include "v8_c_api.h"
#include <stdlib.h>
#include <string.h>

static v8_alloctor DefaultAllocator = {
		.v8_Alloc = malloc,
		.v8_Realloc = realloc,
		.v8_Free = free,
		.v8_Calloc = calloc,
		.v8_Strdup = strdup,
};

static v8_alloctor *allocator;
#define V8_ALLOC allocator->v8_Alloc
#define V8_REALLOC allocator->v8_Realloc
#define V8_FREE allocator->v8_Free
#define V8_CALLOC allocator->v8_Calloc
#define V8_STRDUP allocator->v8_Strdup

struct v8_isolate {
	v8::Isolate* isolate;
};

struct v8_isolate_scope {
	v8_isolate *isolate;
	v8::Locker locker;
	v8_isolate_scope(v8_isolate *v8_isolate): locker(v8_isolate->isolate), isolate(v8_isolate) {}
	~v8_isolate_scope() {}
};

struct v8_context {
	v8_isolate *isolate;
	v8::Persistent<v8::Context> *persistent_ctx;
};

struct v8_handlers_scope {
	v8::HandleScope handle_scope;
	v8_handlers_scope(v8_isolate *v8_isolate): handle_scope(v8_isolate->isolate){}
	~v8_handlers_scope(){}
};

struct v8_local_string {
	v8::Local<v8::String> str;
	v8_local_string(v8_isolate *isolate, const char *buff, size_t len) {
		str = v8::String::NewFromUtf8(isolate->isolate, buff, v8::NewStringType::kNormal, len).ToLocalChecked();
	}
	~v8_local_string() {}
};

struct v8_local_script {
	v8::Local<v8::Script> script;
	v8_local_script(v8_context* v8_ctx, v8_local_string *code) {
		v8::Local<v8::Context> v8_local_ctx = v8_ctx->persistent_ctx->Get(v8_ctx->isolate->isolate);
		v8::MaybeLocal<v8::Script> compilation_res = v8::Script::Compile(v8_local_ctx, code->str);
		if (!compilation_res.IsEmpty()) {
			script = compilation_res.ToLocalChecked();
		}
	}
};

struct v8_local_value {
	v8::Local<v8::Value> val;
	v8_local_value(v8::Local<v8::Value> value): val(value) {}
};

struct v8_utf8_value {
	v8::String::Utf8Value utf8_val;
	v8_utf8_value(v8_isolate *isolate, v8::Local<v8::Value> val): utf8_val(isolate->isolate, val) {}
};

struct v8_local_native_function {
	v8::Local<v8::FunctionTemplate> func;
	v8_local_native_function(v8::Local<v8::FunctionTemplate> f): func(f) {}
};

struct v8_local_object {
	v8::Local<v8::ObjectTemplate> obj;
	v8_local_object(v8::Local<v8::ObjectTemplate> o): obj(o) {};
};

void v8_Initialize(v8_alloctor *alloc) {
	platform = v8::platform::NewDefaultPlatform();
	v8::V8::InitializePlatform(platform.get());
	v8::V8::Initialize();
	if (alloc) {
		allocator = alloc;
	} else {
		allocator = &DefaultAllocator;
	}
}

void v8_Despose() {
	v8::V8::Dispose();
}

v8_isolate* v8_NewIsolate() {
	v8::Isolate::CreateParams create_params;
	create_params.array_buffer_allocator = v8::ArrayBuffer::Allocator::NewDefaultAllocator();
	v8::Isolate *isolate = v8::Isolate::New(create_params);
	v8_isolate *v8_isolate = (struct v8_isolate*)V8_ALLOC(sizeof(*v8_isolate));
	v8_isolate->isolate = isolate;
	return v8_isolate;
}

void v8_FreeIsolate(v8_isolate* isolate) {
	isolate->isolate->Dispose();
	V8_FREE(isolate);
}

v8_isolate_scope* v8_IsolateEnter(v8_isolate *v8_isolate) {
	v8_isolate_scope *v8_isolateScope = (struct v8_isolate_scope*)V8_ALLOC(sizeof(*v8_isolateScope));
	v8_isolateScope = new(v8_isolateScope) v8_isolate_scope(v8_isolate);
	v8_isolate->isolate->Enter();
	return v8_isolateScope;
}

void v8_IsolateExit(v8_isolate_scope *v8_isolate_scope) {
	v8_isolate_scope->isolate->isolate->Exit();
	v8_isolate_scope->~v8_isolate_scope();
	V8_FREE(v8_isolate_scope);
}

v8_handlers_scope* v8_NewHandlersScope(v8_isolate *v8_isolate) {
	v8_handlers_scope *v8_handlersScope = (struct v8_handlers_scope*)V8_ALLOC(sizeof(*v8_handlersScope));
	v8_handlersScope = new (v8_handlersScope) v8_handlers_scope(v8_isolate);
	return v8_handlersScope;
}

void v8_FreeHandlersScope(v8_handlers_scope* v8_handlersScope) {
	v8_handlersScope->~v8_handlers_scope();
	V8_FREE(v8_handlersScope);
}

static v8::Local<v8::Context> v8_NewContexInternal(v8_isolate* v8_isolate, v8_local_object *globals) {
	if (globals) {
		return v8::Context::New(v8_isolate->isolate, nullptr, globals->obj);
	} else {
		return v8::Context::New(v8_isolate->isolate);
	}
}

v8_context* v8_NewContext(v8_isolate* v8_isolate, v8_local_object *globals) {
	v8::HandleScope handle_scope(v8_isolate->isolate);
	v8::Local<v8::Context> context = v8_NewContexInternal(v8_isolate, globals);
	v8::Persistent<v8::Context> *persistent_ctx = new v8::Persistent<v8::Context>(v8_isolate->isolate, context);
	v8_context *v8_context = (struct v8_context*)V8_ALLOC(sizeof(*v8_context));
	v8_context->persistent_ctx = persistent_ctx;
	v8_context->isolate = v8_isolate;
	return v8_context;
}

void v8_FreeContext(v8_context* ctx) {
	delete ctx->persistent_ctx;
	V8_FREE(ctx);
}

void v8_ContextEnter(v8_context *v8_ctx) {
	v8_ctx->persistent_ctx->Get(v8_ctx->isolate->isolate)->Enter();
}

void v8_ContextExit(v8_context *v8_ctx) {
	v8_ctx->persistent_ctx->Get(v8_ctx->isolate->isolate)->Exit();
}

v8_local_string* v8_NewString(v8_isolate* v8_isolate, const char *str, size_t len) {
	v8_local_string *v8_str = (struct v8_local_string*)V8_ALLOC(sizeof(*v8_str));
	v8_str = new (v8_str) v8_local_string(v8_isolate, str, len);
	return v8_str;
}

void v8_FreeString(v8_local_string *str) {
	V8_FREE(str);
}

typedef struct v8_native_function_pd{
	native_funcion func;
	void *pd;
}v8_native_function_pd;

static void v8_NativeBaseFunction(const v8::FunctionCallbackInfo<v8::Value>& info) {
	v8::Local<v8::External> data = v8::Handle<v8::External>::Cast(info.Data());
	v8_native_function_pd *nf_pd = (v8_native_function_pd*)data->Value();
	nf_pd->func((v8_local_value_arr*)&info, info.Length(), nf_pd->pd);
}

v8_local_native_function* v8_NewNativeFunction(v8_isolate* v8_isolate, native_funcion func, void *pd) {
	v8_native_function_pd *nf_pd = (v8_native_function_pd*)V8_ALLOC(sizeof(*nf_pd));
	nf_pd->func = func;
	nf_pd->pd = pd;

	v8::Local<v8::External> data = v8::External::New(v8_isolate->isolate, (void*)nf_pd);
	v8::Local<v8::FunctionTemplate> f = v8::FunctionTemplate::New(v8_isolate->isolate, v8_NativeBaseFunction, data);
	v8_local_native_function *v8_native = (struct v8_local_native_function*)V8_ALLOC(sizeof(*v8_native));
	v8_native = new (v8_native) v8_local_native_function(f);
	return v8_native;
}

void v8_FreeNativeFunction(v8_local_native_function *func) {
	V8_FREE(func);
}

v8_local_value* v8_ArgsGet(v8_local_value_arr *args, size_t i) {
	v8::FunctionCallbackInfo<v8::Value> *info = (v8::FunctionCallbackInfo<v8::Value> *)args;
	v8::Handle<v8::Value> v = (*info)[i];
	v8_local_value *v8_val = (struct v8_local_value*)V8_ALLOC(sizeof(*v8_val));
	v8_val = new (v8_val) v8_local_value(v);
	return v8_val;
}

v8_local_object* v8_NewObject(v8_isolate* v8_isolate) {
	v8::Local<v8::ObjectTemplate> obj = v8::ObjectTemplate::New(v8_isolate->isolate);
	v8_local_object *v8_obj = (struct v8_local_object*)V8_ALLOC(sizeof(*v8_obj));
	v8_obj = new (v8_obj) v8_local_object(obj);
	return v8_obj;
}

void v8_FreeObject(v8_local_object *obj) {
	V8_FREE(obj);
}

void v8_ObjectSetFunction(v8_local_object *obj, v8_local_string *name, v8_local_native_function *f) {
	obj->obj->Set(name->str, f->func);
}

v8_local_script* v8_Compile(v8_context* v8_ctx, v8_local_string* str) {
	v8_local_script *v8_script = (struct v8_local_script*)V8_ALLOC(sizeof(*v8_script));
	v8_script = new (v8_script) v8_local_script(v8_ctx, str);
	if (v8_script->script.IsEmpty()) {
		V8_FREE(v8_script);
		return NULL;
	}
	return v8_script;
}

void v8_FreeScript(v8_local_script *script) {
	V8_FREE(script);
}

v8_local_value* v8_Run(v8_context* v8_ctx, v8_local_script* script) {
	v8::Local<v8::Context> v8_local_ctx = v8_ctx->persistent_ctx->Get(v8_ctx->isolate->isolate);
	v8::MaybeLocal<v8::Value> result = script->script->Run(v8_local_ctx);
	if (result.IsEmpty()) {
		return NULL;
	}

	v8::Local<v8::Value> res = result.ToLocalChecked();

	v8_local_value *v8_val = (struct v8_local_value*)V8_ALLOC(sizeof(*v8_val));
	v8_val = new (v8_val) v8_local_value(res);
	return v8_val;
}

void v8_FreeValue(v8_local_value *val) {
	V8_FREE(val);
}

v8_utf8_value* v8_ToUtf8(v8_isolate *isolate, v8_local_value* val) {
	v8_utf8_value *utf8_val = (struct v8_utf8_value*)V8_ALLOC(sizeof(*utf8_val));
	utf8_val = new (utf8_val) v8_utf8_value(isolate, val->val);
	return utf8_val;
}

void v8_FreeUtf8(v8_utf8_value *val) {
	val->~v8_utf8_value();
	V8_FREE(val);
}

const char* v8_Utf8PtrLen(v8_utf8_value *val, size_t *len) {
	*len = val->utf8_val.length();
	return *val->utf8_val;
}

}
