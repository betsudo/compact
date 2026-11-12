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

import { describe, expect, test } from 'vitest';
import * as runtime from '../src/index.js';

// The secp256r1 (NIST P-256) generator and the group identity (point at
// infinity, encoded as the affine pair (0, 0)).
const G: runtime.Secp256r1Point = {
  x: 48439561293906451759052585252797914202762949526041747995844080717082404635286n,
  y: 36134250956749795798585127919587881956611106672985015071877198253568414405109n,
  identity: false,
};
const IDENTITY: runtime.Secp256r1Point = { x: 0n, y: 0n, identity: true };

describe('secp256r1 group operations', () => {
  test('mulGenerator matches the generator and the identity', () => {
    expect(runtime.secp256r1MulGenerator(1n)).toEqual(G);
    expect(runtime.secp256r1MulGenerator(0n)).toEqual(IDENTITY);
  });

  test('add, mul and mulGenerator agree on doubling', () => {
    const twoG = runtime.secp256r1MulGenerator(2n);
    expect(runtime.secp256r1Add(G, G)).toEqual(twoG);
    expect(runtime.secp256r1Mul(G, 2n)).toEqual(twoG);
  });

  test('the identity is an additive unit and a zero scalar annihilates', () => {
    expect(runtime.secp256r1Add(G, IDENTITY)).toEqual(G);
    expect(runtime.secp256r1Mul(G, 0n)).toEqual(IDENTITY);
  });

  test('a point and its negation add to the identity', () => {
    const negG: runtime.Secp256r1Point = { x: G.x, y: runtime.secp256r1BaseNeg(G.y), identity: false };
    expect(runtime.secp256r1Add(G, negG)).toEqual(IDENTITY);
  });

  test('scalar multiplication is additive in the scalar', () => {
    const fiveG = runtime.secp256r1MulGenerator(5n);
    const threeG = runtime.secp256r1MulGenerator(3n);
    expect(runtime.secp256r1Add(runtime.secp256r1MulGenerator(2n), threeG)).toEqual(fiveG);
    expect(runtime.secp256r1Mul(G, 5n)).toEqual(fiveG);
  });

  test('the largest valid scalar gives the negated generator', () => {
    const negG: runtime.Secp256r1Point = { x: G.x, y: runtime.secp256r1BaseNeg(G.y), identity: false };
    expect(runtime.secp256r1MulGenerator(runtime.SECP256R1_SCALAR_MODULUS - 1n)).toEqual(negG);
    expect(runtime.secp256r1Mul(G, runtime.SECP256R1_SCALAR_MODULUS - 1n)).toEqual(negG);
  });

  test('rejects a scalar outside [0, n)', () => {
    // The group order itself is out of range, so it cannot be used to reach
    // the identity.
    const N = runtime.SECP256R1_SCALAR_MODULUS;
    expect(() => runtime.secp256r1MulGenerator(N)).toThrow();
    expect(() => runtime.secp256r1MulGenerator(-1n)).toThrow();
    expect(() => runtime.secp256r1Mul(G, N)).toThrow();
    expect(() => runtime.secp256r1Mul(G, -1n)).toThrow();
  });
});

describe('secp256r1 point validation', () => {
  const P = runtime.SECP256R1_BASE_MODULUS;

  test('accepts the generator and computed points', () => {
    expect(runtime.isValidSecp256r1Point(G)).toBe(true);
    expect(runtime.isValidSecp256r1Point(runtime.secp256r1MulGenerator(7n))).toBe(true);
  });

  test('the identity is accepted, it has no curve equation to satisfy', () => {
    expect(runtime.isValidSecp256r1Point(IDENTITY)).toBe(true);
    expect(runtime.secp256r1Add(IDENTITY, IDENTITY)).toEqual(IDENTITY);
    expect(runtime.secp256r1Mul(IDENTITY, 5n)).toEqual(IDENTITY);
  });

  test('the identity flag wins over in-range coordinates', () => {
    expect(runtime.isValidSecp256r1Point({ x: 3n, y: 4n, identity: true })).toBe(true);
    expect(runtime.isValidSecp256r1Point({ ...G, identity: true })).toBe(true);
  });

  test('rejects (0, 0) when it is not flagged as the identity', () => {
    expect(runtime.isValidSecp256r1Point({ x: 0n, y: 0n, identity: false })).toBe(false);
  });

  test('rejects a point that is not on the curve', () => {
    // (1, 1) satisfies neither curve equation.
    expect(runtime.isValidSecp256r1Point({ x: 1n, y: 1n, identity: false })).toBe(false);
  });

  test('rejects a point from the other curve', () => {
    expect(runtime.isValidSecp256r1Point(runtime.secp256k1MulGenerator(1n))).toBe(false);
  });

  test('rejects a coordinate that is not reduced', () => {
    expect(runtime.isValidSecp256r1Point({ x: P, y: G.y, identity: false })).toBe(false);
    expect(runtime.isValidSecp256r1Point({ x: G.x, y: P, identity: false })).toBe(false);
    // The identity's coordinates are range-checked too.
    expect(runtime.isValidSecp256r1Point({ x: P, y: 0n, identity: true })).toBe(false);
  });

  test('rejects a negative coordinate', () => {
    expect(runtime.isValidSecp256r1Point({ x: -G.x, y: G.y, identity: false })).toBe(false);
    expect(runtime.isValidSecp256r1Point({ x: 0n, y: -1n, identity: true })).toBe(false);
  });

  test('rejects a value that is not a point object', () => {
    expect(runtime.isValidSecp256r1Point(null)).toBe(false);
    expect(runtime.isValidSecp256r1Point(undefined)).toBe(false);
    expect(runtime.isValidSecp256r1Point(5n)).toBe(false);
    expect(runtime.isValidSecp256r1Point({ x: G.x, y: G.y })).toBe(false);
  });

  test('rejects fields of the wrong type', () => {
    expect(runtime.isValidSecp256r1Point({ x: 0, y: 0, identity: true })).toBe(false);
    expect(runtime.isValidSecp256r1Point({ x: G.x, y: G.y, identity: 'false' })).toBe(false);
  });
});

describe('secp256r1 point coordinates', () => {
  test('pointX and pointY extract the affine coordinates', () => {
    expect(runtime.secp256r1PointX(G)).toEqual(G.x);
    expect(runtime.secp256r1PointY(G)).toEqual(G.y);
  });

  test('the identity has no coordinates', () => {
    expect(() => runtime.secp256r1PointX(IDENTITY)).toThrow(runtime.CompactError);
    expect(() => runtime.secp256r1PointY(IDENTITY)).toThrow(runtime.CompactError);
  });
});
