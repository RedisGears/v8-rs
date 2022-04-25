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

struct v8_isolate_scope {
	v8::Isolate *isolate;
	v8::Locker locker;
	v8_isolate_scope(v8::Isolate *v8_isolate): locker(v8_isolate), isolate(v8_isolate) {}
	~v8_isolate_scope() {}
};

struct v8_context {
	v8::Isolate *isolate;
	v8::Persistent<v8::Context> *persistent_ctx;
};

struct v8_handlers_scope {
	v8::HandleScope handle_scope;
	v8_handlers_scope(v8::Isolate *v8_isolate): handle_scope(v8_isolate){}
};

struct v8_local_string {
	v8::Local<v8::String> str;
	v8_local_string(v8::Isolate *isolate, const char *buff, size_t len) {
		str = v8::String::NewFromUtf8(isolate, buff, v8::NewStringType::kNormal, len).ToLocalChecked();
	}
	v8_local_string(v8::Local<v8::String> val): str(val) {}
	~v8_local_string() {}
};

struct v8_local_script {
	v8::Local<v8::Script> script;
	v8_local_script(v8::Local<v8::Context> v8_local_ctx, v8_local_string *code) {
		v8::MaybeLocal<v8::Script> compilation_res = v8::Script::Compile(v8_local_ctx, code->str);
		if (!compilation_res.IsEmpty()) {
			script = compilation_res.ToLocalChecked();
		}
	}
};

struct v8_local_value {
	v8::Local<v8::Value> val;
	v8_local_value(v8::Local<v8::Value> value): val(value) {}
	v8_local_value(v8::Isolate *isolate, v8::Persistent<v8::Value> *value) {
		val = v8::Local<v8::Value>::New(isolate, *value);
	}
};

struct v8_utf8_value {
	v8::String::Utf8Value utf8_val;
	v8_utf8_value(v8::Isolate *isolate, v8::Local<v8::Value> val): utf8_val(isolate, val) {}
};

struct v8_local_native_function_template {
	v8::Local<v8::FunctionTemplate> func;
	v8_local_native_function_template(v8::Local<v8::FunctionTemplate> f): func(f) {}
};

struct v8_local_native_function {
	v8::Local<v8::Function> func;
	v8_local_native_function(v8::Local<v8::Function> f): func(f) {}
};

struct v8_local_object_template {
	v8::Local<v8::ObjectTemplate> obj;
	v8_local_object_template(v8::Local<v8::ObjectTemplate> o): obj(o) {};
};

struct v8_trycatch {
	v8::TryCatch trycatch;
	v8_trycatch(v8::Isolate *isolate): trycatch(isolate){}
};

struct v8_context_ref {
	v8::Local<v8::Context> context;
	v8_context_ref(v8::Local<v8::Context> ctx): context(ctx){}
};

struct v8_local_promise {
	v8::Local<v8::Promise> promise;
	v8_local_promise(v8::Local<v8::Promise> p): promise(p) {}
};

struct v8_local_resolver {
	v8::Local<v8::Promise::Resolver> resolver;
	v8_local_resolver(v8::Local<v8::Promise::Resolver> r): resolver(r) {}
};

void v8_Initialize(v8_alloctor *alloc) {
//	v8::V8::SetFlagsFromString("--expose_gc");
//	v8::V8::SetFlagsFromString("--log-all");
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

v8_isolate* v8_NewIsolate(size_t initial_heap_size_in_bytes, size_t maximum_heap_size_in_bytes) {
	v8::Isolate::CreateParams create_params;
	create_params.array_buffer_allocator = v8::ArrayBuffer::Allocator::NewDefaultAllocator();
	create_params.constraints.ConfigureDefaultsFromHeapSize(initial_heap_size_in_bytes, maximum_heap_size_in_bytes);
	v8::Isolate *isolate = v8::Isolate::New(create_params);
	return (v8_isolate*)isolate;
}

void v8_FreeIsolate(v8_isolate* i) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	isolate->Dispose();
}

void v8_SetInterrupt(v8_isolate* i, v8_InterruptCallback callback, void *data) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	isolate->RequestInterrupt((v8::InterruptCallback)callback, data);
}

v8_isolate_scope* v8_IsolateEnter(v8_isolate *i) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8_isolate_scope *v8_isolateScope = (struct v8_isolate_scope*)V8_ALLOC(sizeof(*v8_isolateScope));
	v8_isolateScope = new(v8_isolateScope) v8_isolate_scope(isolate);
	isolate->Enter();
	return v8_isolateScope;
}

void v8_IsolateExit(v8_isolate_scope *v8_isolate_scope) {
	v8_isolate_scope->isolate->Exit();
	v8_isolate_scope->~v8_isolate_scope();
	V8_FREE(v8_isolate_scope);
}

void v8_IsolateRaiseException(v8_isolate *i, v8_local_value *exception) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	isolate->ThrowException(exception->val);
}

v8_context_ref* v8_GetCurrentCtxRef(v8_isolate *i) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8_context_ref *ref = (v8_context_ref*) V8_ALLOC(sizeof(*ref));
	ref = new (ref) v8_context_ref(isolate->GetCurrentContext());
	return ref;
}

void v8_IdleNotificationDeadline(v8_isolate *i, double deadline_in_seconds) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	isolate->IdleNotificationDeadline(deadline_in_seconds);
}

v8_trycatch* v8_NewTryCatch(v8_isolate *i) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8_trycatch *trycatch = (v8_trycatch*) V8_ALLOC(sizeof(*trycatch));
	trycatch = new (trycatch) v8_trycatch(isolate);
	return trycatch;
}

v8_local_value* v8_TryCatchGetException(v8_trycatch *trycatch) {
	v8::Local<v8::Value> exception = trycatch->trycatch.Exception();
	v8_local_value *v8_val = (struct v8_local_value*)V8_ALLOC(sizeof(*v8_val));
	v8_val = new (v8_val) v8_local_value(exception);
	return v8_val;
}

void v8_FreeTryCatch(v8_trycatch *trycatch) {
	trycatch->~v8_trycatch();
	V8_FREE(trycatch);
}

v8_handlers_scope* v8_NewHandlersScope(v8_isolate *i) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8_handlers_scope *v8_handlersScope = (struct v8_handlers_scope*)V8_ALLOC(sizeof(*v8_handlersScope));
	v8_handlersScope = new (v8_handlersScope) v8_handlers_scope(isolate);
	return v8_handlersScope;
}

void v8_FreeHandlersScope(v8_handlers_scope* v8_handlersScope) {
	v8_handlersScope->~v8_handlers_scope();
	V8_FREE(v8_handlersScope);
}

static v8::Local<v8::Context> v8_NewContexInternal(v8::Isolate* v8_isolate, v8_local_object_template *globals) {
	if (globals) {
		return v8::Context::New(v8_isolate, nullptr, globals->obj);
	} else {
		return v8::Context::New(v8_isolate);
	}
}

v8_context* v8_NewContext(v8_isolate* i, v8_local_object_template *globals) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8::Local<v8::Context> context = v8_NewContexInternal(isolate, globals);
	v8::Persistent<v8::Context> *persistent_ctx = new v8::Persistent<v8::Context>(isolate, context);
	v8_context *v8_context = (struct v8_context*)V8_ALLOC(sizeof(*v8_context));
	v8_context->persistent_ctx = persistent_ctx;
	v8_context->isolate = isolate;
	return v8_context;
}

void v8_FreeContext(v8_context* ctx) {
	ctx->persistent_ctx->Reset();
	delete ctx->persistent_ctx;
	V8_FREE(ctx);
}

void v8_SetPrivateData(v8_context* ctx, size_t index, void *pd) {
	v8::Local<v8::Context> v8_ctx = ctx->persistent_ctx->Get(ctx->isolate);
	v8::Local<v8::External> data = v8::External::New(ctx->isolate, (void*)pd);
	v8_ctx->SetEmbedderData(index, data);
}

void* v8_GetPrivateData(v8_context* ctx, size_t index) {
	v8::Local<v8::Context> v8_ctx = ctx->persistent_ctx->Get(ctx->isolate);
	v8::Local<v8::External> data = v8::Local<v8::External>::Cast(v8_ctx->GetEmbedderData(index));
	return data->Value();
}

v8_context_ref* v8_ContextEnter(v8_context *v8_ctx) {
	v8_context_ref *ref = (v8_context_ref*) V8_ALLOC(sizeof(*ref));
	ref = new (ref) v8_context_ref(v8_ctx->persistent_ctx->Get(v8_ctx->isolate));
	ref->context->Enter();
	return ref;
}

void v8_FreeContextRef(v8_context_ref *v8_ctx_ref) {
	v8_ctx_ref->context->Exit();
	V8_FREE(v8_ctx_ref);
}

void* v8_GetPrivateDataFromCtxRef(v8_context_ref* ctx_ref, size_t index) {
	v8::Local<v8::External> data = v8::Local<v8::External>::Cast(ctx_ref->context->GetEmbedderData(index));
	return data->Value();
}

v8_local_string* v8_NewString(v8_isolate* i, const char *str, size_t len) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8_local_string *v8_str = (struct v8_local_string*)V8_ALLOC(sizeof(*v8_str));
	v8_str = new (v8_str) v8_local_string(isolate, str, len);
	return v8_str;
}

v8_local_value* v8_StringToValue(v8_local_string *str) {
	v8_local_value *v8_val = (struct v8_local_value*)V8_ALLOC(sizeof(*v8_val));
	v8_val = new (v8_val) v8_local_value(str->str);
	return v8_val;
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
	v8_local_value* val = nf_pd->func((v8_local_value_arr*)&info, info.Length(), nf_pd->pd);
	if (val) {
		info.GetReturnValue().Set(val->val);
		V8_FREE(val);
	}
}

v8_local_native_function_template* v8_NewNativeFunctionTemplate(v8_isolate* i, native_funcion func, void *pd) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8_native_function_pd *nf_pd = (v8_native_function_pd*)V8_ALLOC(sizeof(*nf_pd));
	nf_pd->func = func;
	nf_pd->pd = pd;

	v8::Local<v8::External> data = v8::External::New(isolate, (void*)nf_pd);
	v8::Local<v8::FunctionTemplate> f = v8::FunctionTemplate::New(isolate, v8_NativeBaseFunction, data);
	v8_local_native_function_template *v8_native = (struct v8_local_native_function_template*)V8_ALLOC(sizeof(*v8_native));
	v8_native = new (v8_native) v8_local_native_function_template(f);
	return v8_native;
}

v8_local_native_function* v8_NativeFunctionTemplateToFunction(v8_context_ref *ctx_ref, v8_local_native_function_template *func) {
	v8::Local<v8::Function> f = func->func->GetFunction(ctx_ref->context).ToLocalChecked();
	v8_local_native_function *ret = (v8_local_native_function*) V8_ALLOC(sizeof(*ret));
	ret = new (ret) v8_local_native_function(f);
	return ret;
}

void v8_FreeNativeFunctionTemplate(v8_local_native_function_template *func) {
	V8_FREE(func);
}

void v8_FreeNativeFunction(v8_local_native_function *func) {
	V8_FREE(func);
}

v8_local_value* v8_NativeFunctionToValue(v8_local_native_function *func) {
	v8::Local<v8::Value> v = v8::Local<v8::Value>::Cast(func->func);
	v8_local_value *v8_val = (struct v8_local_value*)V8_ALLOC(sizeof(*v8_val));
	v8_val = new (v8_val) v8_local_value(v);
	return v8_val;
}

v8_local_value* v8_ArgsGet(v8_local_value_arr *args, size_t i) {
	v8::FunctionCallbackInfo<v8::Value> *info = (v8::FunctionCallbackInfo<v8::Value> *)args;
	v8::Handle<v8::Value> v = (*info)[i];
	v8_local_value *v8_val = (struct v8_local_value*)V8_ALLOC(sizeof(*v8_val));
	v8_val = new (v8_val) v8_local_value(v);
	return v8_val;
}

v8_isolate* v8_GetCurrentIsolate(v8_local_value_arr *args) {
	v8::FunctionCallbackInfo<v8::Value> *info = (v8::FunctionCallbackInfo<v8::Value> *)args;
	v8::Isolate* isolate = info->GetIsolate();
	return (v8_isolate*)isolate;
}

v8_local_object_template* v8_NewObjectTemplate(v8_isolate* i) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8::Local<v8::ObjectTemplate> obj = v8::ObjectTemplate::New(isolate);
	v8_local_object_template *v8_obj = (struct v8_local_object_template*)V8_ALLOC(sizeof(*v8_obj));
	v8_obj = new (v8_obj) v8_local_object_template(obj);
	return v8_obj;
}

void v8_FreeObjectTemplate(v8_local_object_template *obj) {
	V8_FREE(obj);
}

void v8_ObjectTemplateSetFunction(v8_local_object_template *obj, v8_local_string *name, v8_local_native_function_template *f) {
	obj->obj->Set(name->str, f->func);
}

void v8_ObjectTemplateSetObject(v8_local_object_template *obj, v8_local_string *name, v8_local_object_template *o) {
	obj->obj->Set(name->str, o->obj);
}

void v8_ObjectTemplateSetValue(v8_local_object_template *obj, v8_local_string *name, v8_local_value *val) {
	obj->obj->Set(name->str, val->val);
}

v8_local_value* v8_ObjectTemplateToValue(v8_context_ref *ctx_ref, v8_local_object_template *obj) {
	v8::Local<v8::Value> v = obj->obj->NewInstance(ctx_ref->context).ToLocalChecked();
	v8_local_value *v8_val = (struct v8_local_value*)V8_ALLOC(sizeof(*v8_val));
	v8_val = new (v8_val) v8_local_value(v);
	return v8_val;
}

v8_local_script* v8_Compile(v8_context_ref* v8_ctx_ref, v8_local_string* str) {
	v8_local_script *v8_script = (struct v8_local_script*)V8_ALLOC(sizeof(*v8_script));
	v8_script = new (v8_script) v8_local_script(v8_ctx_ref->context, str);
	if (v8_script->script.IsEmpty()) {
		V8_FREE(v8_script);
		return NULL;
	}
	return v8_script;
}

void v8_FreeScript(v8_local_script *script) {
	V8_FREE(script);
}

v8_local_value* v8_Run(v8_context_ref* v8_ctx_ref, v8_local_script* script) {
	v8::MaybeLocal<v8::Value> result = script->script->Run(v8_ctx_ref->context);
	if (result.IsEmpty()) {
		return NULL;
	}

	v8::Local<v8::Value> res = result.ToLocalChecked();

	v8_local_value *v8_val = (struct v8_local_value*)V8_ALLOC(sizeof(*v8_val));
	v8_val = new (v8_val) v8_local_value(res);
	return v8_val;
}

int v8_ValueIsFunction(v8_local_value *val){
	return val->val->IsFunction();
}

v8_local_value* v8_FunctionCall(v8_context_ref *v8_ctx_ref, v8_local_value *val, size_t argc, v8_local_value* const* argv) {
	v8::Local<v8::Value> argv_arr[argc];
	for (size_t i = 0 ; i < argc ; ++i) {
		argv_arr[i] = argv[i]->val;
	}
	v8::Local<v8::Function> function = v8::Local<v8::Function>::Cast(val->val);
	v8::MaybeLocal<v8::Value> result = function->Call(v8_ctx_ref->context, v8_ctx_ref->context->Global(), argc, argv_arr);
	if (result.IsEmpty()) {
		return NULL;
	}

	v8::Local<v8::Value> res = result.ToLocalChecked();

	v8_local_value *v8_val = (struct v8_local_value*)V8_ALLOC(sizeof(*v8_val));
	v8_val = new (v8_val) v8_local_value(res);
	return v8_val;
}

int v8_ValueIsAsyncFunction(v8_local_value *val) {
	return val->val->IsAsyncFunction();
}

int v8_ValueIsString(v8_local_value *val) {
	return val->val->IsString();
}

v8_local_string* v8_ValueAsString(v8_local_value *val) {
	v8_local_string *v8_str = (struct v8_local_string*)V8_ALLOC(sizeof(*v8_str));
	v8_str = new (v8_str) v8_local_string(v8::Local<v8::String>::Cast(val->val));
	return v8_str;
}

int v8_ValueIsBigInt(v8_local_value *val) {
	return val->val->IsBigInt();
}

int v8_ValueIsNumber(v8_local_value *val) {
	return val->val->IsNumber();
}

int v8_ValueIsPromise(v8_local_value *val) {
	return val->val->IsPromise();
}

v8_local_promise* v8_ValueAsPromise(v8_local_value *val) {
	v8::Local<v8::Promise> promise = v8::Local<v8::Promise>::Cast(val->val);
	v8_local_promise* p = (v8_local_promise*)V8_ALLOC(sizeof(*p));
	p = new (p) v8_local_promise(promise);
	return p;
}

void v8_FreePromise(v8_local_promise* promise) {
	V8_FREE(promise);
}
v8_PromiseState v8_PromiseGetState(v8_local_promise* promise) {
	v8::Promise::PromiseState s = promise->promise->State();
	switch(s) {
	case v8::Promise::PromiseState::kPending:
		return v8_PromiseState_Pending;
	case v8::Promise::PromiseState::kFulfilled:
		return v8_PromiseState_Fulfilled;
	case v8::Promise::PromiseState::kRejected:
		return v8_PromiseState_Rejected;
	}
	return v8_PromiseState_Unknown;
}

v8_local_value* v8_PromiseGetResult(v8_local_promise* promise) {
	v8_local_value* val = (v8_local_value*)V8_ALLOC(sizeof(*val));
	val = new (val) v8_local_value(promise->promise->Result());
	return val;
}

void v8_PromiseThen(v8_local_promise* promise, v8_context_ref *ctx_ref, v8_local_native_function *resolve, v8_local_native_function *reject) {
	v8::MaybeLocal<v8::Promise> _may_local = promise->promise->Then(ctx_ref->context, resolve->func, reject->func);
}

v8_local_value* v8_PromiseToValue(v8_local_promise *promise) {
	v8::Local<v8::Value> val = v8::Local<v8::Value>::Cast(promise->promise);
	v8_local_value *res = (v8_local_value*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_value(val);
	return res;
}

v8_local_resolver* v8_NewResolver(v8_context_ref *ctx_ref) {
	v8::Local<v8::Promise::Resolver> resolver = v8::Promise::Resolver::New(ctx_ref->context).ToLocalChecked();
	v8_local_resolver *res = (v8_local_resolver*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_resolver(resolver);
	return res;
}

void v8_FreeResolver(v8_local_resolver *resolver) {
	V8_FREE(resolver);
}

v8_local_promise* v8_ResolverGetPromise(v8_local_resolver *resolver) {
	v8::Local<v8::Promise> promise = resolver->resolver->GetPromise();
	v8_local_promise *res = (v8_local_promise*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_promise(promise);
	return res;
}

void v8_ResolverResolve(v8_context_ref *ctx_ref, v8_local_resolver *resolver, v8_local_value *val) {
	v8::Maybe<bool> res = resolver->resolver->Resolve(ctx_ref->context, val->val);
}

void v8_ResolverReject(v8_context_ref *ctx_ref, v8_local_resolver *resolver, v8_local_value *val) {
	v8::Maybe<bool> res = resolver->resolver->Reject(ctx_ref->context, val->val);
}

v8_local_value* v8_ResolverToValue(v8_local_resolver *resolver) {
	v8::Local<v8::Value> val = v8::Local<v8::Value>::Cast(resolver->resolver);
	v8_local_value *res = (v8_local_value*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_value(val);
	return res;
}

int v8_ValueIsObject(v8_local_value *val) {
	return val->val->IsObject();
}

v8_persisted_value* v8_PersistValue(v8_isolate *i, v8_local_value *val) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	return (v8_persisted_value*) new v8::Persistent<v8::Value>(isolate, val->val);
}

v8_local_value* v8_PersistedValueToLocal(v8_isolate *i, v8_persisted_value *val) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8::Persistent<v8::Value> *persisted_val = (v8::Persistent<v8::Value>*)val;
	v8_local_value *local_val = (struct v8_local_value*)V8_ALLOC(sizeof(*local_val));
	local_val = new (local_val) v8_local_value(isolate, persisted_val);
	return local_val;
}

void v8_FreePersistedValue(v8_persisted_value *val) {
	v8::Persistent<v8::Value> *persisted_val = (v8::Persistent<v8::Value>*)val;
	persisted_val->Reset();
	delete persisted_val;
}

void v8_FreeValue(v8_local_value *val) {
	V8_FREE(val);
}

v8_utf8_value* v8_ToUtf8(v8_isolate *i, v8_local_value* val) {
	v8::Isolate *isolate = (v8::Isolate*)i;
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
