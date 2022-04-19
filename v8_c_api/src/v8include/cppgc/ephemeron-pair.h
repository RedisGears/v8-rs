// Copyright 2020 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef V8INCLUDE_CPPGC_EPHEMERON_PAIR_H_
#define V8INCLUDE_CPPGC_EPHEMERON_PAIR_H_

#include "../../v8include/cppgc/liveness-broker.h"
#include "../../v8include/cppgc/member.h"

namespace cppgc {

/**
 * An ephemeron pair is used to conditionally retain an object.
 * The `value` will be kept alive only if the `key` is alive.
 */
template <typename K, typename V>
struct EphemeronPair {
  EphemeronPair(K* k, V* v) : key(k), value(v) {}
  WeakMember<K> key;
  Member<V> value;

  void ClearValueIfKeyIsDead(const LivenessBroker& broker) {
    if (!broker.IsHeapObjectAlive(key)) value = nullptr;
  }
};

}  // namespace cppgc

#endif  // V8INCLUDE_CPPGC_EPHEMERON_PAIR_H_
