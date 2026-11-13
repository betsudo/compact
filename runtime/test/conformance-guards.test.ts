// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
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

import { expect, test } from 'vitest';
import { CircuitSignatures, InterfaceDescriptor, SignatureType, checkConformance, signatureTypesEqual } from '../src/index.js';

const field: SignatureType = { tag: 'Field' };
const declaration: InterfaceDescriptor = { use: { argumentTypes: [field], resultType: field } };
const malformed = (value: unknown): SignatureType => value as SignatureType;
const implementation = (argument: SignatureType, provable: unknown = true): CircuitSignatures =>
  ({ use: { pure: false, provable, argumentTypes: [argument], resultType: field } }) as CircuitSignatures;

test('a truthy nonboolean proof flag does not establish provability', () => {
  for (const provable of ['false', 1, {}, true]) {
    expect(checkConformance(declaration, implementation(field, provable))).toEqual(
      provable === true ? { outcome: 'Conformant' } : { outcome: 'Violation', circuitId: 'use', check: 'Provability' },
    );
  }
});

test('malformed payloads of known tags are unreadable', () => {
  const sparseEnum = Array<string>(1);
  const cases: readonly (readonly [string, unknown])[] = [
    ['Uint', { tag: 'Uint' }],
    ['Uint', { tag: 'Uint', maxval: 'not a decimal' }],
    ['Bytes', { tag: 'Bytes', length: -1 }],
    ['Opaque', { tag: 'Opaque', tsType: 4 }],
    ['Enum', { tag: 'Enum', name: 4, elements: ['a'] }],
    ['Enum', { tag: 'Enum', name: 'E', elements: [4] }],
    ['Enum', { tag: 'Enum', name: 'E', elements: sparseEnum }],
    ['Struct', { tag: 'Struct', name: 'S', elements: [{ name: 4, type: field }] }],
    ['Alias', { tag: 'Alias', name: 4, type: field }],
    ['Contract', { tag: 'Contract', name: 4, circuits: {} }],
  ];
  for (const [unreadableTag, value] of cases) {
    expect(checkConformance(declaration, implementation(malformed(value)))).toEqual({
      outcome: 'Unreadable',
      circuitId: 'use',
      unreadableTag,
      argumentIndex: 0,
    });
  }
});

test('a shared acyclic descriptor cannot exhaust the caller', () => {
  let shared: unknown = field;
  for (let i = 0; i < 25; i += 1) shared = { tag: 'Tuple', types: [shared, shared] };
  expect(checkConformance(declaration, implementation(malformed(shared)))).toMatchObject({
    outcome: 'Unreadable',
    circuitId: 'use',
    argumentIndex: 0,
  });
});

test('equal vector descriptors compare their element type once', () => {
  const huge: SignatureType = { tag: 'Vector', length: Number.MAX_SAFE_INTEGER, type: field };
  expect(signatureTypesEqual(huge, huge)).toEqual(true);
  expect(checkConformance({ use: { argumentTypes: [huge], resultType: field } }, implementation(huge))).toEqual({
    outcome: 'Conformant',
  });
});

test('nested contract arguments stop at the validation budget', () => {
  const argumentTypes = Array<SignatureType>(20_000).fill(field);
  const nested = malformed({ tag: 'Contract', name: 'Nested', circuits: { c: { argumentTypes, resultType: field } } });
  expect(checkConformance(declaration, implementation(nested))).toMatchObject({
    outcome: 'Unreadable',
    circuitId: 'use',
    argumentIndex: 0,
  });
});

test('nested contract circuit names stop at the validation budget', () => {
  const circuits = Object.fromEntries(
    Array.from({ length: 20_000 }, (_, i) => [`c${i}`, { argumentTypes: [], resultType: field }]),
  );
  const nested = malformed({ tag: 'Contract', name: 'Nested', circuits });
  expect(checkConformance(declaration, implementation(nested))).toMatchObject({
    outcome: 'Unreadable',
    circuitId: 'use',
    argumentIndex: 0,
  });
});

test('inherited descriptor names do not consume the own-name budget', () => {
  const prototype = Object.fromEntries(
    Array.from({ length: 20_000 }, (_, i) => [`inherited${i}`, { argumentTypes: [], resultType: field }]),
  );
  const circuits = Object.assign(Object.create(prototype) as Record<string, unknown>, {
    c: { argumentTypes: [], resultType: field },
  });
  const declaredType = malformed({ tag: 'Contract', name: 'Nested', circuits: { c: circuits.c } });
  const implementedType = malformed({ tag: 'Contract', name: 'Nested', circuits });
  expect(
    checkConformance({ use: { argumentTypes: [declaredType], resultType: field } }, implementation(implementedType)),
  ).toEqual({ outcome: 'Conformant' });
});

test('cyclic and deeply nested external types are unreadable', () => {
  const cycle: { tag: string; name: string; type?: unknown } = { tag: 'Alias', name: 'Loop' };
  cycle.type = cycle;
  expect(checkConformance(declaration, implementation(malformed(cycle)))).toEqual({
    outcome: 'Unreadable',
    circuitId: 'use',
    unreadableTag: 'Alias',
    argumentIndex: 0,
  });

  let deep: unknown = field;
  for (let i = 0; i < 140; i += 1) deep = { tag: 'Alias', name: `Level${i}`, type: deep };
  expect(checkConformance(declaration, implementation(malformed(deep)))).toMatchObject({
    outcome: 'Unreadable',
    circuitId: 'use',
    argumentIndex: 0,
  });
});

test('shared acyclic subtypes remain readable', () => {
  const shared: SignatureType = { tag: 'Alias', name: 'Value', type: field };
  const d: InterfaceDescriptor = { use: { argumentTypes: [shared, shared], resultType: field } };
  const i: CircuitSignatures = { use: { pure: false, provable: true, argumentTypes: [shared, shared], resultType: field } };
  expect(checkConformance(d, i)).toEqual({ outcome: 'Conformant' });
});
