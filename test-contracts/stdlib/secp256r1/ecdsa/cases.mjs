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

export const identityError =
  'Secp256r1Point identity is not a permitted secp256r1EcdsaVerify verification key';
export const zeroError = 'secp256r1 scalar field has no inverse for 0';
export const coordinateHex = (value) => value.toString(16).padStart(64, '0');
export const bytesHex = (value) => Buffer.from(value).toString('hex');
export const encodePoint = ({ x, y, identity }) => ({ x: coordinateHex(x), y: coordinateHex(y), identity });

export function signatureCases(p256, keccak) {
  const { generator } = pointCases(p256);
  const secret = p256.Point.Fn.toBytes(7n);
  const message = Uint8Array.from({ length: 32 }, (_, index) => index + 1);
  const changed = Uint8Array.from(message, (byte, index) => index === 0 ? byte ^ 1 : byte);
  const digest = keccak(message);
  const { r, s } = p256.Signature.fromBytes(p256.sign(digest, secret, { prehash: false }));
  const sig = { r, s };
  const twin = { r, s: p256.Point.Fn.ORDER - s };
  const pk = generator(7n);
  const other = generator(8n);
  const cases = [
    { id: 'original', message, sig, pk, expected: true },
    { id: 'twin', message, sig: twin, pk, expected: true },
    { id: 'wrong-key', message, sig, pk: other, expected: false },
    { id: 'wrong-message', message: changed, sig, pk, expected: false },
  ];
  for (const row of cases) {
    const signature = new p256.Signature(row.sig.r, row.sig.s).toBytes('compact');
    const key = p256.Point.fromAffine({ x: row.pk.x, y: row.pk.y }).toBytes(false);
    assert.equal(p256.verify(signature, keccak(row.message), key, { prehash: false, lowS: false }),
      row.expected, row.id);
  }
  return cases;
}
