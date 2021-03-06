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
	v8_local_script(v8::Local<v8::Script> s): script(s) {}
};

struct v8_local_module {
	v8::Local<v8::Module> mod;
	v8_local_module(v8::Local<v8::Module> m): mod(m) {}
	v8_local_module(v8::Isolate *isolate, v8::Persistent<v8::Module> *m) {
		mod = v8::Local<v8::Module>::New(isolate, *m);
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

struct v8_local_object {
	v8::Local<v8::Object> obj;
	v8_local_object(v8::Local<v8::Object> o): obj(o) {}
};

struct v8_local_set {
	v8::Local<v8::Set> set;
	v8_local_set(v8::Local<v8::Set> o): set(o) {}
};

struct v8_local_array {
	v8::Local<v8::Array> arr;
	v8_local_array(v8::Local<v8::Array> a): arr(a) {}
};

struct v8_local_array_buff {
	v8::Local<v8::ArrayBuffer> arr_buff;
	v8_local_array_buff(v8::Local<v8::ArrayBuffer> a): arr_buff(a) {}
};

typedef struct v8_native_function_pd v8_native_function_pd;
typedef struct v8_pd_node v8_pd_node;
typedef struct v8_pd_list v8_pd_list;

struct v8_native_function_pd{
	v8_pd_node *node;
	native_funcion func;
	void *pd;
	v8::Persistent<v8::External> *weak;
	void(*freePD)(void *pd);
};

void v8_FreeNaticeFunctionPD(v8_native_function_pd *pd) {
	pd->freePD(pd->pd);
	pd->weak->Reset();
	delete pd->weak;
	V8_FREE(pd);
}

struct v8_pd_node{
	v8_pd_list *list;
	v8_pd_node *prev;
	v8_pd_node *next;
	void *data;
	void (*free_data)(void *data);
};

struct v8_pd_list{
	v8::ArrayBuffer::Allocator *allocator;
	v8_pd_node *start;
	v8_pd_node *end;
};

void v8_ListNodeFree(v8_pd_node *node) {
	if (node->free_data) {
		node->free_data(node->data);
	}
	v8_pd_list *list = node->list;
	if (list->start == node) {
		list->start = node->next;
	}
	if (list->end == node) {
		list->end = node->prev;
	}
	if (node->next) {
		node->next->prev = node->prev;
	}
	if (node->prev) {
		node->prev->next = node->next;
	}
	V8_FREE(node);
}

v8_pd_node* v8_PDListAdd(v8_pd_list *list, void *pd, void (*free_data)(void *data)) {
	v8_pd_node *new_node = (v8_pd_node*)V8_ALLOC(sizeof(*new_node));
	if (list->end) {
		list->end->next = new_node;
	}
	new_node->list = list;
	new_node->prev = list->end;
	new_node->next = NULL;
	new_node->data = pd;
	new_node->free_data = free_data;
	list->end = new_node;
	if (!list->start) {
		list->start = new_node;
	}

	return new_node;
}

void v8_PDListFree(v8_pd_list* pd_list) {
	while (pd_list->end) {
		v8_ListNodeFree(pd_list->end);
	}
	V8_FREE(pd_list);
}

v8_pd_list* v8_PDListCreate(v8::ArrayBuffer::Allocator *alloc) {
	v8_pd_list *native_data = (v8_pd_list*)V8_ALLOC(sizeof(*native_data));
	native_data->start = NULL;
	native_data->end = NULL;
	native_data->allocator = alloc;
	return native_data;
}

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

const char* v8_Version() {
	return v8::V8::GetVersion();
}

void v8_Dispose() {
	v8::V8::Dispose();
}

static void v8_FreeAllocator(v8::ArrayBuffer::Allocator* allocator) {
	delete allocator;
}

v8_isolate* v8_NewIsolate(size_t initial_heap_size_in_bytes, size_t maximum_heap_size_in_bytes) {
	v8::Isolate::CreateParams create_params;
	create_params.array_buffer_allocator = v8::ArrayBuffer::Allocator::NewDefaultAllocator();
	create_params.constraints.ConfigureDefaultsFromHeapSize(initial_heap_size_in_bytes, maximum_heap_size_in_bytes);
	v8::Isolate *isolate = v8::Isolate::New(create_params);

	v8_pd_list *native_data = v8_PDListCreate(create_params.array_buffer_allocator);
	isolate->SetData(0, native_data);

	return (v8_isolate*)isolate;
}

void v8_IsolateSetFatalErrorHandler(v8_isolate* i, void (*fatal_hanlder)(const char* location, const char* message)) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	isolate->SetFatalErrorHandler(fatal_hanlder);
}

void v8_IsolateSetOOMErrorHandler(v8_isolate* i, void (*oom_hanlder)(const char* location, int is_heap_oom)) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	isolate->SetOOMErrorHandler((v8::OOMErrorCallback)oom_hanlder);
}

void v8_IsolateSetNearOOMHandler(v8_isolate* i, size_t (*near_oom_callback)(void* data, size_t current_heap_limit, size_t initial_heap_limit), void *pd, void(*free_pd)(void*)) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8_pd_list *native_data = (v8_pd_list*)isolate->GetData(0);
	v8_PDListAdd(native_data, pd, free_pd);
	isolate->AddNearHeapLimitCallback(near_oom_callback, pd);
	isolate->AutomaticallyRestoreInitialHeapLimit();
}

void v8_TerminateCurrExecution(v8_isolate* i) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	isolate->TerminateExecution();
}

void v8_CancelTerminateExecution(v8_isolate* i) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	isolate->CancelTerminateExecution();
}

void v8_FreeIsolate(v8_isolate* i) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8_pd_list *native_data = (v8_pd_list*)isolate->GetData(0);
	v8::ArrayBuffer::Allocator *allocator = native_data->allocator;
	v8_PDListFree(native_data);
	isolate->Dispose();
	v8_FreeAllocator(allocator);
}

void v8_RequestInterrupt(v8_isolate* i, v8_InterruptCallback callback, void *data) {
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

int v8_TryCatchHasTerminated(v8_trycatch *trycatch) {
	return trycatch->trycatch.HasTerminated() ? 1 : 0;
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
	v8_ctx->SetEmbedderData(index + 2, data);
}

void* v8_GetPrivateData(v8_context* ctx, size_t index) {
	v8::Local<v8::Context> v8_ctx = ctx->persistent_ctx->Get(ctx->isolate);
	v8::Local<v8::External> data = v8::Local<v8::External>::Cast(v8_ctx->GetEmbedderData(index + 2));
	return data->Value();
}

v8_context_ref* v8_ContextEnter(v8_context *v8_ctx) {
	v8_context_ref *ref = (v8_context_ref*) V8_ALLOC(sizeof(*ref));
	ref = new (ref) v8_context_ref(v8_ctx->persistent_ctx->Get(v8_ctx->isolate));
	ref->context->Enter();
	return ref;
}

v8_isolate* v8_ContextRefGetIsolate(v8_context_ref *v8_ctx_ref) {
	return (v8_isolate*)v8_ctx_ref->context->GetIsolate();
}

v8_local_object* v8_ContextRefGetGlobals(v8_context_ref *v8_ctx_ref) {
	v8::Local<v8::Object> globals = v8_ctx_ref->context->Global();

	v8_local_object *v8_globals = (struct v8_local_object*)V8_ALLOC(sizeof(*v8_globals));
	v8_globals = new (v8_globals) v8_local_object(globals);
	return v8_globals;
}

void v8_ExitContextRef(v8_context_ref *v8_ctx_ref) {
	v8_ctx_ref->context->Exit();
}

void v8_FreeContextRef(v8_context_ref *v8_ctx_ref) {
	V8_FREE(v8_ctx_ref);
}

void* v8_GetPrivateDataFromCtxRef(v8_context_ref* ctx_ref, size_t index) {
	v8::Local<v8::External> data = v8::Local<v8::External>::Cast(ctx_ref->context->GetEmbedderData(index + 2));
	return data->Value();
}

void v8_SetPrivateDataOnCtxRef(v8_context_ref* ctx_ref, size_t index, void *pd) {
	v8::Isolate *isolate = ctx_ref->context->GetIsolate();
	v8::Local<v8::External> data = v8::External::New(isolate, (void*)pd);
	ctx_ref->context->SetEmbedderData(index + 2, data);
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

v8_local_object* v8_StringToStringObject(v8_isolate* i, v8_local_string *str) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8::Local<v8::Value> str_obj = v8::StringObject::New(isolate, str->str);
	v8::Local<v8::Object> obj = v8::Local<v8::Object>::Cast(str_obj);
	v8_local_object *res = (v8_local_object*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_object(obj);
	return res;
}

void v8_FreeString(v8_local_string *str) {
	V8_FREE(str);
}

static void v8_NativeBaseFunction(const v8::FunctionCallbackInfo<v8::Value>& info) {
	v8::Local<v8::External> data = v8::Handle<v8::External>::Cast(info.Data());
	v8_native_function_pd *nf_pd = (v8_native_function_pd*)data->Value();
	v8_local_value* val = nf_pd->func((v8_local_value_arr*)&info, info.Length(), nf_pd->pd);
	if (val) {
		info.GetReturnValue().Set(val->val);
		V8_FREE(val);
	}
}

static void v8_FreeNativeFunctionPD(const v8::WeakCallbackInfo<v8_pd_node> &data) {
	v8_pd_node* node = data.GetParameter();
    v8_ListNodeFree(node);
}

v8_local_native_function_template* v8_NewNativeFunctionTemplate(v8_isolate* i, native_funcion func, void *pd, void(*freePD)(void *pd)) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8_native_function_pd *nf_pd = (v8_native_function_pd*)V8_ALLOC(sizeof(*nf_pd));
	nf_pd->func = func;
	nf_pd->pd = pd;
	nf_pd->freePD = freePD;

	v8_pd_list *native_data = (v8_pd_list*)isolate->GetData(0);
	v8_pd_node* node = v8_PDListAdd(native_data, (void*)nf_pd, (void(*)(void*))v8_FreeNaticeFunctionPD);

	v8::Local<v8::External> data = v8::External::New(isolate, (void*)nf_pd);
	nf_pd->weak = new v8::Persistent<v8::External>(isolate, data);
    nf_pd->weak->SetWeak<v8_pd_node>(node, v8_FreeNativeFunctionPD, v8::WeakCallbackType::kParameter);

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

v8_local_native_function* v8_NewNativeFunction(v8_context_ref *ctx_ref, native_funcion func, void *pd, void(*freePD)(void *pd)) {
	v8::Isolate *isolate = ctx_ref->context->GetIsolate();
	v8_native_function_pd *nf_pd = (v8_native_function_pd*)V8_ALLOC(sizeof(*nf_pd));
	nf_pd->func = func;
	nf_pd->pd = pd;
	nf_pd->freePD = freePD;

	v8_pd_list *native_data = (v8_pd_list*)isolate->GetData(0);
	v8_pd_node* node = v8_PDListAdd(native_data, (void*)nf_pd, (void(*)(void*))v8_FreeNaticeFunctionPD);

	v8::Local<v8::External> data = v8::External::New(ctx_ref->context->GetIsolate(), (void*)nf_pd);
	nf_pd->weak = new v8::Persistent<v8::External>(isolate, data);
	nf_pd->weak->SetWeak<v8_pd_node>(node, v8_FreeNativeFunctionPD, v8::WeakCallbackType::kParameter);

	v8::Local<v8::Function> f = v8::Function::New(ctx_ref->context, v8_NativeBaseFunction, data).ToLocalChecked();

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

v8_persisted_script* v8_ScriptPersist(v8_isolate *i, v8_local_script* script) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	return (v8_persisted_script*) new v8::Persistent<v8::Script>(isolate, script->script);
}

v8_local_script* v8_PersistedScriptToLocal(v8_isolate *i, v8_persisted_script* script) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8::Persistent<v8::Script> *persisted_script = (v8::Persistent<v8::Script>*)script;
	v8::Local<v8::Script> s = v8::Local<v8::Script>::New(isolate, *persisted_script);
	v8_local_script *local_script = (struct v8_local_script*)V8_ALLOC(sizeof(*local_script));
	local_script = new (local_script) v8_local_script(s);
	return local_script;
}

void v8_FreePersistedScript(v8_persisted_script* script) {
	v8::Persistent<v8::Script> *persisted_script = (v8::Persistent<v8::Script>*)script;
	persisted_script->Reset();
	delete persisted_script;
}

static v8::MaybeLocal<v8::Module> v8_ResolveModules(v8::Local<v8::Context> context, v8::Local<v8::String> specifier,
													v8::Local<v8::FixedArray> import_assertions, v8::Local<v8::Module> referrer) {
	v8::Local<v8::External> external = v8::Local<v8::External>::Cast(context->GetEmbedderData(1));
	V8_LoadModuleCallback load_module_callback = (V8_LoadModuleCallback)external->Value();

	v8_context_ref *v8_ctx_ref = (struct v8_context_ref*)V8_ALLOC(sizeof(*v8_ctx_ref));
	v8_ctx_ref = new (v8_ctx_ref) v8_context_ref(context);

	v8_local_string* name = (struct v8_local_string*)V8_ALLOC(sizeof(*name));
	int identity_hash = referrer->GetIdentityHash();

	name = new (name) v8_local_string(specifier);

	v8_local_module* m = load_module_callback(v8_ctx_ref, name, identity_hash);

	v8::MaybeLocal<v8::Module> res;
	if (m) {
		res = m->mod;
		v8_FreeModule(m);
	}

	return res;
}

v8_local_module* v8_CompileAsModule(v8_context_ref* v8_ctx_ref, v8_local_string* name, v8_local_string* code, int is_module) {
	v8::Isolate *isolate = v8_ctx_ref->context->GetIsolate();
	v8::ScriptOrigin origin(isolate, name->str, 0, 0, false, -1, v8::Local<v8::Value>(), false, false, is_module, v8::Local<v8::Data>());

	v8::ScriptCompiler::Source source(code->str, origin);
	v8::MaybeLocal<v8::Module> mod = v8::ScriptCompiler::CompileModule(isolate, &source);

	if (mod.IsEmpty()) {
		return NULL;
	}

	v8_local_module *ret = (struct v8_local_module*)V8_ALLOC(sizeof(*ret));
	ret = new (ret) v8_local_module(mod.ToLocalChecked());
	return ret;
}

int v8_InitiateModule(v8_local_module* m, v8_context_ref* v8_ctx_ref, V8_LoadModuleCallback load_module_callback) {
	v8::Isolate *isolate = v8_ctx_ref->context->GetIsolate();
	v8::Local<v8::External> data = v8::External::New(isolate, (void*)load_module_callback);
	v8_ctx_ref->context->SetEmbedderData(1, data);
	v8::Maybe<bool> res = m->mod->InstantiateModule(v8_ctx_ref->context, v8_ResolveModules);
	return res.IsNothing() ? 0 : 1;
}

int v8_ModuleGetIdentityHash(v8_local_module* m) {
	return m->mod->GetIdentityHash();
}

v8_local_value* v8_EvaluateModule(v8_local_module* m, v8_context_ref* v8_ctx_ref) {
	v8::MaybeLocal<v8::Value> res = m->mod->Evaluate(v8_ctx_ref->context);
	if (res.IsEmpty()) {
		return NULL;
	}

	v8_local_value *val = (struct v8_local_value*)V8_ALLOC(sizeof(*val));
	val = new (val) v8_local_value(res.ToLocalChecked());
	return val;
}

v8_persisted_module* v8_ModulePersist(v8_isolate *i, v8_local_module* m) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	return (v8_persisted_module*) new v8::Persistent<v8::Module>(isolate, m->mod);
}

v8_local_module* v8_ModuleToLocal(v8_isolate *i, v8_persisted_module* m) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8::Persistent<v8::Module> *persisted_module = (v8::Persistent<v8::Module>*)m;
	v8_local_module *local_module = (struct v8_local_module*)V8_ALLOC(sizeof(*local_module));
	local_module = new (local_module) v8_local_module(isolate, persisted_module);
	return local_module;
}

void v8_FreePersistedModule(v8_persisted_module* m) {
	v8::Persistent<v8::Module> *persisted_module = (v8::Persistent<v8::Module>*)m;
	persisted_module->Reset();
	delete persisted_module;
}

void v8_FreeModule(v8_local_module* m) {
	V8_FREE(m);
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

int v8_ValueIsStringObject(v8_local_value *val) {
	return val->val->IsStringObject();
}

v8_local_string* v8_ValueAsString(v8_local_value *val) {
	v8_local_string *v8_str = (struct v8_local_string*)V8_ALLOC(sizeof(*v8_str));
	v8_str = new (v8_str) v8_local_string(v8::Local<v8::String>::Cast(val->val));
	return v8_str;
}

v8_local_value* v8_ValueFromLong(v8_isolate *i, long long val) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8::Local<v8::BigInt> big_int = v8::BigInt::New(isolate, val);
	v8::Local<v8::Value> v = v8::Local<v8::Value>::Cast(big_int);

	v8_local_value *v8_val = (struct v8_local_value*)V8_ALLOC(sizeof(*v8_val));
	v8_val = new (v8_val) v8_local_value(v);
	return v8_val;
}

int v8_ValueIsBigInt(v8_local_value *val) {
	return val->val->IsBigInt() || val->val->IsInt32();
}

long long v8_GetBigInt(v8_local_value *val) {
	if (val->val->IsInt32()) {
		v8::Local<v8::Int32> integer = v8::Local<v8::Int32>::Cast(val->val);
		int64_t res = integer->Value();
		return res;
	}
	v8::Local<v8::BigInt> big_int = v8::Local<v8::BigInt>::Cast(val->val);
	int64_t res = big_int->Int64Value();
	return res;
}

int v8_ValueIsNumber(v8_local_value *val) {
	return val->val->IsNumber();
}

double v8_GetNumber(v8_local_value *val) {
	v8::Local<v8::Number> number = v8::Local<v8::Number>::Cast(val->val);
	return number->Value();
}

int v8_ValueIsBool(v8_local_value *val) {
	return val->val->IsBoolean();
}

int v8_GetBool(v8_local_value *val){
	v8::Local<v8::Boolean> boolean = v8::Local<v8::Boolean>::Cast(val->val);
	return boolean->Value();
}


v8_local_value* v8_ValueFromDouble(v8_isolate *i, double val) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8::Local<v8::Number> number = v8::Number::New(isolate, val);
	v8::Local<v8::Value> v = v8::Local<v8::Value>::Cast(number);

	v8_local_value *v8_val = (struct v8_local_value*)V8_ALLOC(sizeof(*v8_val));
	v8_val = new (v8_val) v8_local_value(v);
	return v8_val;
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

v8_local_array* v8_ValueGetPropertyNames(v8_context_ref *ctx_ref, v8_local_object *obj) {
	v8::MaybeLocal<v8::Array> maybe_res = obj->obj->GetPropertyNames(ctx_ref->context);
	if (maybe_res.IsEmpty()) {
		return NULL;
	}
	v8::Local<v8::Array> arr = maybe_res.ToLocalChecked();
	v8_local_array *res = (v8_local_array*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_array(arr);
	return res;
}

int v8_ValueIsArray(v8_local_value *val) {
	return val->val->IsArray();
}

int v8_ValueIsArrayBuffer(v8_local_value *val) {
	return val->val->IsArrayBuffer();
}

v8_local_object* v8_NewObject(v8_isolate *i) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8::Local<v8::Object> obj = v8::Object::New(isolate);
	v8_local_object *res = (v8_local_object*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_object(obj);
	return res;
}

v8_local_value* v8_NewObjectFromJsonString(v8_context_ref *ctx_ref, v8_local_string *str) {
	v8::MaybeLocal<v8::Value> result = v8::JSON::Parse(ctx_ref->context, str->str);
	if (result.IsEmpty()) {
		return NULL;
	}
	v8::Local<v8::Value> val = result.ToLocalChecked();
	v8_local_value *res = (v8_local_value*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_value(val);
	return res;
}

v8_local_object* v8_ValueAsObject(v8_local_value *val) {
	v8::Local<v8::Object> obj = v8::Local<v8::Object>::Cast(val->val);
	v8_local_object *res = (v8_local_object*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_object(obj);
	return res;
}

v8_local_resolver* v8_ValueAsResolver(v8_local_value *val) {
	v8::Local<v8::Promise::Resolver> resolver = v8::Local<v8::Promise::Resolver>::Cast(val->val);
	v8_local_resolver *res = (v8_local_resolver*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_resolver(resolver);
	return res;
}

v8_local_value* v8_ObjectGet(v8_context_ref *ctx_ref, v8_local_object *obj, v8_local_value *key) {
	v8::MaybeLocal<v8::Value> maybe_val = obj->obj->Get(ctx_ref->context, key->val);
	if (maybe_val.IsEmpty()) {
		return NULL;
	}
	v8::Local<v8::Value> val = maybe_val.ToLocalChecked();
	v8_local_value *res = (v8_local_value*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_value(val);
	return res;
}

void v8_ObjectSet(v8_context_ref *ctx_ref, v8_local_object *obj, v8_local_value *key, v8_local_value *val) {
	v8::Maybe<bool> res = obj->obj->Set(ctx_ref->context, key->val, val->val);
}

void v8_ObjectFreeze(v8_context_ref *ctx_ref, v8_local_object *obj) {
	obj->obj->SetIntegrityLevel(ctx_ref->context, v8::IntegrityLevel::kFrozen);
}

void v8_FreeObject(v8_local_object *obj) {
	V8_FREE(obj);
}

v8_local_value* v8_ObjectToValue(v8_local_object *obj) {
	v8::Local<v8::Value> val = v8::Local<v8::Value>::Cast(obj->obj);
	v8_local_value *res = (v8_local_value*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_value(val);
	return res;
}

v8_local_set* v8_NewSet(v8_isolate *i) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8::Local<v8::Set> set = v8::Set::New(isolate);
	v8_local_set *res = (v8_local_set*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_set(set);
	return res;
}

/* Add a value to the set */
void v8_SetAdd(v8_context_ref *ctx_ref, v8_local_set *set, v8_local_value *val) {
	v8::MaybeLocal<v8::Set> res = set->set->Add(ctx_ref->context, val->val);
}

/* Convert the given JS set into JS generic value */
v8_local_value* v8_SetToValue(v8_local_set *set) {
	v8::Local<v8::Value> val = v8::Local<v8::Value>::Cast(set->set);
	v8_local_value *res = (v8_local_value*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_value(val);
	return res;
}

/* Convert the generic JS value into a JS set */
v8_local_set* v8_ValueAsSet(v8_local_value *val) {
	v8::Local<v8::Set> set = v8::Local<v8::Set>::Cast(val->val);
	v8_local_set *res = (v8_local_set*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_set(set);
	return res;
}

/* Return 1 if the given JS value is a set and 0 otherwise */
int v8_ValueIsSet(v8_local_value *val) {
	return val->val->IsSet();
}

void v8_FreeSet(v8_local_set *set) {
	V8_FREE(set);
}

v8_local_value* v8_NewBool(v8_isolate *i, int val) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8::Local<v8::Boolean> b = v8::Boolean::New(isolate, val);
	v8::Local<v8::Value> v = v8::Local<v8::Value>::Cast(b);
	v8_local_value *res = (v8_local_value*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_value(v);
	return res;
}

v8_local_value* v8_NewNull(v8_isolate *i) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8::Local<v8::Primitive> n = v8::Null(isolate);
	v8::Local<v8::Value> v = v8::Local<v8::Value>::Cast(n);

	v8_local_value *v8_val = (struct v8_local_value*)V8_ALLOC(sizeof(*v8_val));
	v8_val = new (v8_val) v8_local_value(v);
	return v8_val;
}

int v8_ValueIsNull(v8_local_value *val) {
	return val->val->IsNull();
}

v8_local_array_buff* v8_NewArrayBuffer(v8_isolate *i, const char *data, size_t len) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8::Local<v8::ArrayBuffer> arr_buff = v8::ArrayBuffer::New(isolate, len);
	void *buff = arr_buff->GetBackingStore()->Data();
	memcpy(buff, data, len);
	v8_local_array_buff *res = (v8_local_array_buff*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_array_buff(arr_buff);
	return res;
}

v8_local_value* v8_ArrayBufferToValue(v8_local_array_buff *arr_buffer) {
	v8::Local<v8::Value> val = v8::Local<v8::Value>::Cast(arr_buffer->arr_buff);
	v8_local_value *res = (v8_local_value*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_value(val);
	return res;
}

const void* v8_ArrayBufferGetData(v8_local_array_buff *arr_buffer, size_t *len) {
	*len = arr_buffer->arr_buff->ByteLength();
	return arr_buffer->arr_buff->GetBackingStore()->Data();
}

void v8_FreeArrayBuffer(v8_local_array_buff *arr_buffer) {
	V8_FREE(arr_buffer);
}

v8_local_array* v8_NewArray(v8_isolate *i, v8_local_value *const *vals, size_t len) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8::Local<v8::Value> vals_arr[len];
	for (size_t i = 0 ; i < len ; ++i) {
		vals_arr[i] = vals[i]->val;
	}
	v8::Local<v8::Array> arr = v8::Array::New(isolate, vals_arr, len);
	v8_local_array *res = (v8_local_array*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_array(arr);
	return res;
}

void v8_FreeArray(v8_local_array *arr) {
	V8_FREE(arr);
}

size_t v8_ArrayLen(v8_local_array *arr) {
	return arr->arr->Length();
}

v8_local_value* v8_ArrayGet(v8_context_ref *ctx_ref, v8_local_array *arr, size_t index) {
	v8::MaybeLocal<v8::Value> maybe_val = arr->arr->Get(ctx_ref->context, index);
	if (maybe_val.IsEmpty()) {
		return NULL;
	}
	v8::Local<v8::Value> val = maybe_val.ToLocalChecked();
	v8_local_value *res = (v8_local_value*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_value(val);
	return res;
}

v8_local_value* v8_ArrayToValue(v8_local_array *arr) {
	v8::Local<v8::Value> val = v8::Local<v8::Value>::Cast(arr->arr);
	v8_local_value *res = (v8_local_value*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_value(val);
	return res;
}

v8_local_array* v8_ValueAsArray(v8_local_value *val) {
	v8::Local<v8::Array> arr = v8::Local<v8::Array>::Cast(val->val);
	v8_local_array *res = (v8_local_array*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_array(arr);
	return res;
}

v8_local_array_buff* v8_ValueAsArrayBuffer(v8_local_value *val) {
	v8::Local<v8::ArrayBuffer> arr = v8::Local<v8::ArrayBuffer>::Cast(val->val);
	v8_local_array_buff *res = (v8_local_array_buff*) V8_ALLOC(sizeof(*res));
	res = new (res) v8_local_array_buff(arr);
	return res;
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

v8_unlocker* v8_NewUnlocker(v8_isolate *i) {
	v8::Isolate *isolate = (v8::Isolate*)i;
	v8::Unlocker *unlocker = new v8::Unlocker(isolate);
	return (v8_unlocker*)unlocker;
}

void v8_FreeUnlocker(v8_unlocker* u) {
	v8::Unlocker *unlocker = (v8::Unlocker*)u;
	delete unlocker;
}

}
