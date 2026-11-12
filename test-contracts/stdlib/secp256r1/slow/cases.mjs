// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import assert from 'node:assert/strict';
import { pointCases } from '../point/cases.mjs';

export function sequentialCases(p256) {
  const { generator } = pointCases(p256);
  const digest = Uint8Array.from({ length: 32 }, (_, index) => index + 1);
  const keys = [5n, 7n, 11n];
  const keyBytes = keys.map((key) => p256.Point.Fn.toBytes(key));
  const points = keys.map(generator);
  const signatures = keyBytes.map((key, index) => {
    const raw = p256.sign(digest, key, { prehash: false, format: 'compact' });
    assert(p256.verify(raw, digest, p256.getPublicKey(key), { prehash: false }));
    const alternate = keyBytes[(index + 1) % keys.length];
    assert(!p256.verify(raw, digest, p256.getPublicKey(alternate), { prehash: false }));
    const { r, s } = p256.Signature.fromBytes(raw, 'compact');
    return { r, s };
  });
  return { digest, points, signatures };
}

export function witnessInputs(cases, count) {
  return {
    signatures: cases.signatures.slice(0, count),
    points: cases.points.slice(0, count),
  };
}

export function witnesses(inputs, calls) {
  const get = (kind, values) => ({ privateState }, index) => {
    const slot = Number(index);
    assert.equal(privateState, calls.length, kind);
    assert(slot < values.length, kind);
    calls.push([kind, slot]);
    return [privateState + 1, values[slot]];
  };
  return { sigOf: get('signature', inputs.signatures), keyOf: get('key', inputs.points) };
}

export function witnessOrder(count) {
  return Array.from({ length: count }, (_, slot) => [['signature', slot], ['key', slot]]).flat();
}
