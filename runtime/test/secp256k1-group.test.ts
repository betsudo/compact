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

// The secp256k1 generator and the group identity (point at infinity, encoded
// as the affine pair (0, 0)).
const G: runtime.Secp256k1Point = {
  x: 55066263022277343669578718895168534326250603453777594175500187360389116729240n,
  y: 32670510020758816978083085130507043184471273380659243275938904335757337482424n,
  identity: false,
};
const IDENTITY: runtime.Secp256k1Point = { x: 0n, y: 0n, identity: true };

describe('secp256k1 group operations', () => {
  test('mulGenerator matches the generator and the identity', () => {
    expect(runtime.secp256k1MulGenerator(1n)).toEqual(G);
    expect(runtime.secp256k1MulGenerator(0n)).toEqual(IDENTITY);
  });

  test('add, mul and mulGenerator agree on doubling', () => {
    const twoG = runtime.secp256k1MulGenerator(2n);
    expect(runtime.secp256k1Add(G, G)).toEqual(twoG);
    expect(runtime.secp256k1Mul(G, 2n)).toEqual(twoG);
  });

  test('the identity is an additive unit and a zero scalar annihilates', () => {
    expect(runtime.secp256k1Add(G, IDENTITY)).toEqual(G);
    expect(runtime.secp256k1Mul(G, 0n)).toEqual(IDENTITY);
  });
});

describe('secp256k1 point validation', () => {
  const P = runtime.SECP256K1_BASE_MODULUS;

  test('accepts the generator and computed points', () => {
    expect(runtime.isValidSecp256k1Point(G)).toBe(true);
    expect(runtime.isValidSecp256k1Point(runtime.secp256k1MulGenerator(7n))).toBe(true);
  });

  test('the identity is accepted, it has no curve equation to satisfy', () => {
    expect(runtime.isValidSecp256k1Point(IDENTITY)).toBe(true);
    expect(runtime.secp256k1Add(IDENTITY, IDENTITY)).toEqual(IDENTITY);
    expect(runtime.secp256k1Mul(IDENTITY, 5n)).toEqual(IDENTITY);
  });

  test('the identity flag wins over in-range coordinates', () => {
    expect(runtime.isValidSecp256k1Point({ x: 3n, y: 4n, identity: true })).toBe(true);
    expect(runtime.isValidSecp256k1Point({ ...G, identity: true })).toBe(true);
  });

  test('rejects (0, 0) when it is not flagged as the identity', () => {
    expect(runtime.isValidSecp256k1Point({ x: 0n, y: 0n, identity: false })).toBe(false);
  });

  test('rejects a point that is not on the curve', () => {
    // (1, 1) satisfies neither curve equation.
    expect(runtime.isValidSecp256k1Point({ x: 1n, y: 1n, identity: false })).toBe(false);
  });

  test('rejects a point from the other curve', () => {
    // The upstream call produces this standard P-256 generator.
    expect(
      runtime.isValidSecp256k1Point({
        x: 48439561293906451759052585252797914202762949526041747995844080717082404635286n,
        y: 36134250956749795798585127919587881956611106672985015071877198253568414405109n,
        identity: false,
      }),
    ).toBe(false);
  });

  test('rejects a coordinate that is not reduced', () => {
    expect(runtime.isValidSecp256k1Point({ x: P, y: G.y, identity: false })).toBe(false);
    expect(runtime.isValidSecp256k1Point({ x: G.x, y: P, identity: false })).toBe(false);
    // The identity's coordinates are range-checked too.
    expect(runtime.isValidSecp256k1Point({ x: P, y: 0n, identity: true })).toBe(false);
  });

  test('rejects a negative coordinate', () => {
    expect(runtime.isValidSecp256k1Point({ x: -G.x, y: G.y, identity: false })).toBe(false);
    expect(runtime.isValidSecp256k1Point({ x: 0n, y: -1n, identity: true })).toBe(false);
  });

  test('rejects a value that is not a point object', () => {
    expect(runtime.isValidSecp256k1Point(null)).toBe(false);
    expect(runtime.isValidSecp256k1Point(undefined)).toBe(false);
    expect(runtime.isValidSecp256k1Point(5n)).toBe(false);
    expect(runtime.isValidSecp256k1Point({ x: G.x, y: G.y })).toBe(false);
  });

  test('rejects fields of the wrong type', () => {
    expect(runtime.isValidSecp256k1Point({ x: 0, y: 0, identity: true })).toBe(false);
    expect(runtime.isValidSecp256k1Point({ x: G.x, y: G.y, identity: 'false' })).toBe(false);
  });
});

describe('secp256k1 scalar field operations', () => {
  const N = runtime.SECP256K1_SCALAR_MODULUS;
  const a = 123456789n;
  const b = N - 7n;

  test('add reduces modulo the scalar modulus', () => {
    expect(runtime.secp256k1ScalarAdd(a, b)).toEqual((a + b) % N);
    expect(runtime.secp256k1ScalarAdd(a, N - a)).toEqual(0n);
  });

  test('neg is the additive inverse', () => {
    expect(runtime.secp256k1ScalarAdd(a, runtime.secp256k1ScalarNeg(a))).toEqual(0n);
    expect(runtime.secp256k1ScalarNeg(0n)).toEqual(0n);
  });

  test('mul reduces modulo the scalar modulus', () => {
    expect(runtime.secp256k1ScalarMul(a, b)).toEqual((a * b) % N);
  });

  test('inv is the multiplicative inverse', () => {
    expect(runtime.secp256k1ScalarMul(a, runtime.secp256k1ScalarInv(a))).toEqual(1n);
  });
});

describe('secp256k1 base field operations', () => {
  const P = runtime.SECP256K1_BASE_MODULUS;
  const a = 987654321n;
  const b = P - 11n;

  test('add reduces modulo the base modulus', () => {
    expect(runtime.secp256k1BaseAdd(a, b)).toEqual((a + b) % P);
    expect(runtime.secp256k1BaseAdd(a, P - a)).toEqual(0n);
  });

  test('neg is the additive inverse', () => {
    expect(runtime.secp256k1BaseAdd(a, runtime.secp256k1BaseNeg(a))).toEqual(0n);
    expect(runtime.secp256k1BaseNeg(0n)).toEqual(0n);
  });

  test('mul reduces modulo the base modulus', () => {
    expect(runtime.secp256k1BaseMul(a, b)).toEqual((a * b) % P);
  });

  test('inv is the multiplicative inverse', () => {
    expect(runtime.secp256k1BaseMul(a, runtime.secp256k1BaseInv(a))).toEqual(1n);
  });
});
