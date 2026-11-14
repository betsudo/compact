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

import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { generator, five, seven } from '../point/cases.mjs';

export const message = new Uint8Array(32).fill(0xab);
export const originalInvalid = { r: 17n, s: 31n };
export const identityError =
  'Secp256k1Point identity is not a permitted secp256k1EcdsaVerify verification key';

export function witnesses(first, second) {
  const result = (value) => ({ privateState }) => [privateState, value];
  return {
    pk0: result(first.pk),
    sig0: result(first.sig),
    pk1: result(second.pk),
    sig1: result(second.sig),
  };
}

export async function signatureCases(runtimeEntry) {
  const require = createRequire(runtimeEntry);
  const { secp256k1 } = await import(/* @vite-ignore */ pathToFileURL(require.resolve('@noble/curves/secp256k1.js')).href);
  const { keccak_256 } = await import(/* @vite-ignore */ pathToFileURL(require.resolve('@noble/hashes/sha3.js')).href);
  const digest = keccak_256(message);
  const sign = (secret, pk) => {
    const key = secp256k1.Point.Fn.toBytes(secret);
    const signature = secp256k1.sign(digest, key, { prehash: false, lowS: true, format: 'compact' });
    const { r, s } = secp256k1.Signature.fromBytes(signature, 'compact');
    return { pk, sig: { r, s }, key, signature };
  };
  const first = sign(5n, five);
  const second = sign(7n, seven);
  const high = { ...first.sig, s: secp256k1.Point.Fn.ORDER - first.sig.s };
  const changed = Uint8Array.from(message);
  changed[0] ^= 1;
  const verify = (bytes, digest, key) =>
    secp256k1.verify(bytes, digest, secp256k1.getPublicKey(key), { prehash: false, lowS: false });
  if (!verify(first.signature, digest, first.key) ||
      !verify(second.signature, digest, second.key) ||
      !verify(new secp256k1.Signature(high.r, high.s).toBytes('compact'), digest, first.key) ||
      verify(first.signature, digest, second.key) ||
      verify(first.signature, keccak_256(changed), first.key)) {
    throw new Error('independent ECDSA fixture checks failed');
  }
  return { digest, first, second, high, changed, generator };
}
