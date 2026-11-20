// This file is part of Compact.
// Copyright (C) 2025 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { describe, expect, test } from 'vitest';
import * as runtime from '../src/index.js';
import { p256 } from '@noble/curves/nist.js';
import { keccak_256 } from '@noble/hashes/sha3.js';
import { utf8ToBytes } from '@noble/hashes/utils.js';

describe('secp256r1 ECDSA public key recovery', () => {
  // Signing is RFC 6979 deterministic and low-s normalised, so every run
  // produces the same signature and a failure is reproducible.
  const SK = 7n;
  const MESSAGE = 'compact ecrecover test vector';

  const digest = keccak_256(utf8ToBytes(MESSAGE));
  // `recovered` format is the recovery byte followed by r and s.
  const sigBytes = p256.sign(digest, p256.Point.Fn.toBytes(SK), {
    prehash: false,
    format: 'recovered',
  });
  const recoveryId = sigBytes[0];
  const parsed = p256.Signature.fromBytes(sigBytes, 'recovered');
  const sig = { r: parsed.r, s: parsed.s };
  // The signer's public key, SK*G.
  const pk = runtime.secp256r1MulGenerator(SK);

  test('recovers the signing public key from a known signature', () => {
    expect(runtime.secp256r1EcdsaRecover(digest, sig, recoveryId)).toEqual(pk);
  });

  test('the recovered key agrees with scalar multiplication of the generator', () => {
    expect(runtime.secp256r1EcdsaRecover(digest, sig, recoveryId)).toEqual(runtime.secp256r1MulGenerator(SK));
  });

  test('the other recovery id selects a different candidate key', () => {
    const other = runtime.secp256r1EcdsaRecover(digest, sig, recoveryId ^ 1);
    expect(other).not.toEqual(pk);
  });

  const N = runtime.SECP256R1_SCALAR_MODULUS;
  const highS = { r: sig.r, s: N - sig.s };

  test('the signature under test is the low-s representative', () => {
    expect(sig.s <= N / 2n).toBe(true);
    expect(highS.s > N / 2n).toBe(true);
  });

  test('accepts high-s and recovers the same key when the id is flipped', () => {
    expect(runtime.secp256r1EcdsaRecover(digest, highS, recoveryId ^ 1)).toEqual(pk);
  });

  test('negating s without flipping the id recovers a different key', () => {
    expect(runtime.secp256r1EcdsaRecover(digest, highS, recoveryId)).not.toEqual(pk);
  });

  test('rejects a message hash that is not 32 bytes', () => {
    expect(() => runtime.secp256r1EcdsaRecover(digest.slice(0, 31), sig, recoveryId)).toThrow(runtime.CompactError);
    expect(() => runtime.secp256r1EcdsaRecover(new Uint8Array(33), sig, recoveryId)).toThrow(runtime.CompactError);
  });

  test('rejects a recovery id outside [0, 3]', () => {
    expect(() => runtime.secp256r1EcdsaRecover(digest, sig, -1)).toThrow(runtime.CompactError);
    expect(() => runtime.secp256r1EcdsaRecover(digest, sig, 4)).toThrow(runtime.CompactError);
    expect(() => runtime.secp256r1EcdsaRecover(digest, sig, 1.5)).toThrow(runtime.CompactError);
  });

  test('rejects r and s outside the scalar field', () => {
    expect(() => runtime.secp256r1EcdsaRecover(digest, { r: 0n, s: sig.s }, recoveryId)).toThrow();
    expect(() => runtime.secp256r1EcdsaRecover(digest, { r: sig.r, s: 0n }, recoveryId)).toThrow();
    expect(() => runtime.secp256r1EcdsaRecover(digest, { r: N, s: sig.s }, recoveryId)).toThrow();
    expect(() => runtime.secp256r1EcdsaRecover(digest, { r: sig.r, s: N }, recoveryId)).toThrow();
  });
});

describe('secp256r1 recovery edge cases', () => {
  const digest = new Uint8Array(32);
  digest[31] = 1;

  test('recovers both parities when the nonce x-coordinate is r plus the order', () => {
    const sig = { r: 3n, s: 1n };
    expect(runtime.secp256r1EcdsaRecover(digest, sig, 2)).toEqual({
      x: 0xabefd53035325dcb20b6967a2c284a9bfc679b8446682523b32baf41120d76e4n,
      y: 0xb4d5ee3eb39ab3dc059bc7cb8c8fcf1f842cee7d729602a7f15eaa653d81872dn,
      identity: false,
    });
    expect(runtime.secp256r1EcdsaRecover(digest, sig, 3)).toEqual({
      x: 0xab835d9808d0b3e93199f38d0c1c9a5ab8c9bf62516ffbf37b037ea17f2fbd41n,
      y: 0xe99c2933ec5b6de96409c9c5ebe6a33842583f16805b96074e9a977b125d29cdn,
      identity: false,
    });
  });

  test('rejects a wrapped nonce coordinate outside the base field', () => {
    const sig = { r: 0x4319055358e8617b0c46353d039cdaaen, s: 1n };
    expect(() => runtime.secp256r1EcdsaRecover(digest, sig, 2)).toThrow();
  });

  test('rejects a signature and digest that recover the identity', () => {
    const generator = runtime.secp256r1MulGenerator(1n);
    const sig = { r: generator.x, s: 1n };
    expect(() => runtime.secp256r1EcdsaRecover(digest, sig, Number(generator.y & 1n))).toThrow();
  });
});
