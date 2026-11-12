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
import {
  CompactTypeOpaqueString,
  CompactTypeSecp256k1Base,
  CompactTypeSecp256k1Scalar,
  CompactTypeSecp256r1Base,
  CompactTypeSecp256r1Scalar,
  MAX_SECP256K1_BASE,
  MAX_SECP256K1_SCALAR,
  MAX_SECP256R1_BASE,
  MAX_SECP256R1_SCALAR,
} from '../src/index.js';

describe('opaque strings', () => {
  test.each(['hello', '', '\uFEFFhello'])('round trips Opaque<"string"> %j', (text) => {
    expect(CompactTypeOpaqueString.fromValue(CompactTypeOpaqueString.toValue(text))).toBe(text);
  });

  test.each([
    ['a leading U+FEFF', [0xef, 0xbb, 0xbf, 0x68, 0x69]],
    ['two leading U+FEFFs', [0xef, 0xbb, 0xbf, 0xef, 0xbb, 0xbf, 0x68, 0x69]],
    ['nothing but U+FEFF', [0xef, 0xbb, 0xbf]],
  ])('Opaque<"string"> decoded from bytes with %s encodes back to those bytes', (_label, bytes) => {
    const stored = new Uint8Array(bytes);
    const decoded = CompactTypeOpaqueString.fromValue([stored]);
    expect(CompactTypeOpaqueString.toValue(decoded)).toStrictEqual([stored]);
  });
});

describe.each([
  ['Secp256k1Base', CompactTypeSecp256k1Base, MAX_SECP256K1_BASE],
  ['Secp256k1Scalar', CompactTypeSecp256k1Scalar, MAX_SECP256K1_SCALAR],
  ['Secp256r1Base', CompactTypeSecp256r1Base, MAX_SECP256R1_BASE],
  ['Secp256r1Scalar', CompactTypeSecp256r1Scalar, MAX_SECP256R1_SCALAR],
] as const)('%s', (name, descriptor, max) => {
  test('uses two little-endian atoms in low then high order', () => {
    const low = Uint8Array.from({ length: 24 }, (_, i) => i + 1);
    const high = Uint8Array.from({ length: 8 }, (_, i) => 0x21 + i);
    const value = 0x28272625242322211817161514131211100f0e0d0c0b0a090807060504030202n;
    expect(descriptor.alignment()).toStrictEqual([
      { tag: 'atom', value: { tag: 'bytes', length: 24 } },
      { tag: 'atom', value: { tag: 'bytes', length: 8 } },
    ]);
    expect(descriptor.toValue(value)).toStrictEqual([low, high]);
    const tail = Uint8Array.of(42);
    const atoms = [low, high, tail];
    expect(descriptor.fromValue(atoms)).toBe(value);
    expect(atoms).toStrictEqual([tail]);
  });

  test.each([0n, 1n, 987654321n, max])('round trips canonical value %s', (value) => {
    expect(descriptor.fromValue(descriptor.toValue(value))).toBe(value);
  });

  test('encodes one as two empty atoms', () => {
    expect(descriptor.toValue(1n)).toStrictEqual([new Uint8Array(), new Uint8Array()]);
  });

  test('refuses noncanonical values and malformed limbs', () => {
    const empty = new Uint8Array();
    const oversizedLow = new Uint8Array(25);
    oversizedLow[24] = 1;
    const oversizedHigh = new Uint8Array(9);
    oversizedHigh[8] = 1;
    expect(() => descriptor.toValue(-1n)).toThrow(`expected ${name}`);
    expect(() => descriptor.toValue(max + 1n)).toThrow(`expected ${name}`);
    expect(() => descriptor.fromValue([empty])).toThrow(`expected ${name}`);
    expect(() => descriptor.fromValue([oversizedLow, empty])).toThrow(`expected ${name}`);
    expect(() => descriptor.fromValue([empty, oversizedHigh])).toThrow(`expected ${name}`);
    expect(() => descriptor.fromValue([new Uint8Array(24).fill(0xff), new Uint8Array(8).fill(0xff)])).toThrow(`expected ${name}`);
  });
});
