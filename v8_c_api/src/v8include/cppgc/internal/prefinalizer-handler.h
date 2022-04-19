// Copyright 2020 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef V8INCLUDE_CPPGC_INTERNAL_PREFINALIZER_HANDLER_H_
#define V8INCLUDE_CPPGC_INTERNAL_PREFINALIZER_HANDLER_H_

#include "../../../v8include/cppgc/heap.h"
#include "../../../v8include/cppgc/liveness-broker.h"

namespace cppgc {
namespace internal {

class V8_EXPORT PreFinalizerRegistrationDispatcher final {
 public:
  using PreFinalizerCallback = bool (*)(const LivenessBroker&, void*);
  struct PreFinalizer {
    void* object;
    PreFinalizerCallback callback;

    bool operator==(const PreFinalizer& other) const;
  };

  static void RegisterPrefinalizer(PreFinalizer pre_finalizer);
};

}  // namespace internal
}  // namespace cppgc

#endif  // V8INCLUDE_CPPGC_INTERNAL_PREFINALIZER_HANDLER_H_
