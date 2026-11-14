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

import { expect, test, vi } from 'vitest';
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

test('large valid interfaces are checked per declared circuit', () => {
  const names = Array.from({ length: 300 }, (_, i) => `Variant${i}`);
  const largeEnum: SignatureType = { tag: 'Enum', name: 'Large', elements: names };
  for (const count of [34, 40]) {
    const declared: InterfaceDescriptor = Object.fromEntries(
      Array.from({ length: count }, (_, i) => [`c${i}`, { argumentTypes: [largeEnum], resultType: field }]),
    );
    const implemented: CircuitSignatures = Object.fromEntries(
      Array.from({ length: count }, (_, i) => [
        `c${i}`,
        { pure: false, provable: true, argumentTypes: [largeEnum], resultType: field },
      ]),
    );
    expect(checkConformance(declared, implemented)).toEqual({ outcome: 'Conformant' });
  }
});

test('a large caller declaration permits matching work but still refuses tampering and cycles', () => {
  const names = Array.from({ length: 12_000 }, (_, i) => `Variant${i}`);
  const largeEnum: SignatureType = { tag: 'Enum', name: 'Large', elements: names };
  const declared: InterfaceDescriptor = { use: { argumentTypes: [largeEnum], resultType: field } };
  expect(checkConformance(declared, implementation(largeEnum))).toEqual({ outcome: 'Conformant' });
  expect(
    checkConformance(declared, implementation({ tag: 'Enum', name: 'Large', elements: [...names.slice(0, -1), 'Changed'] })),
  ).toEqual({ outcome: 'Violation', circuitId: 'use', check: 'ArgumentType', argumentIndex: 0 });
  expect(checkConformance(declared, implementation(malformed({ tag: 'Enum', name: 'Large', elements: [...names, 4] })))).toEqual({
    outcome: 'Unreadable',
    circuitId: 'use',
    unreadableTag: 'Enum',
    argumentIndex: 0,
  });

  const cycle: { tag: string; name: string; type?: unknown } = { tag: 'Alias', name: 'Loop' };
  cycle.type = cycle;
  expect(checkConformance(declared, implementation(malformed(cycle)))).toEqual({
    outcome: 'Unreadable',
    circuitId: 'use',
    unreadableTag: 'Alias',
    argumentIndex: 0,
  });
  let deep: unknown = field;
  for (let i = 0; i < 140; i += 1) deep = { tag: 'Alias', name: `Level${i}`, type: deep };
  expect(checkConformance(declared, implementation(malformed(deep)))).toMatchObject({
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

test('a Vector logical length does not enlarge the unrelated implementation budget', () => {
  const huge: SignatureType = { tag: 'Vector', length: Number.MAX_SAFE_INTEGER, type: field };
  const declared: InterfaceDescriptor = { use: { argumentTypes: [huge], resultType: field } };
  const nested = malformed({
    tag: 'Contract',
    name: 'Nested',
    circuits: { c: { argumentTypes: Array<SignatureType>(20_000).fill(field), resultType: field } },
  });
  expect(checkConformance(declared, implementation(nested))).toMatchObject({
    outcome: 'Unreadable',
    circuitId: 'use',
    argumentIndex: 0,
  });
});

test('a large Vector can conform to a matching explicit Tuple without lending its budget to another argument', () => {
  const length = 12_000;
  const expected: SignatureType = { tag: 'Vector', length, type: field };
  const expanded: SignatureType = { tag: 'Tuple', types: Array<SignatureType>(length).fill(field) };
  const declared: InterfaceDescriptor = { use: { argumentTypes: [expected, field], resultType: field } };
  const matching: CircuitSignatures = {
    use: { pure: false, provable: true, argumentTypes: [expanded, field], resultType: field },
  };
  expect(checkConformance(declared, matching)).toEqual({ outcome: 'Conformant' });
  const changed = [...expanded.types];
  changed[length - 1] = { tag: 'Boolean' };
  expect(
    checkConformance(declared, {
      use: { pure: false, provable: true, argumentTypes: [{ tag: 'Tuple', types: changed }, field], resultType: field },
    }),
  ).toEqual({ outcome: 'Violation', circuitId: 'use', check: 'ArgumentType', argumentIndex: 0 });

  const unrelated = malformed({
    tag: 'Contract',
    name: 'Nested',
    circuits: { c: { argumentTypes: Array<SignatureType>(20_000).fill(field), resultType: field } },
  });
  expect(
    checkConformance(declared, {
      use: { pure: false, provable: true, argumentTypes: [expanded, unrelated], resultType: field },
    }),
  ).toMatchObject({ outcome: 'Unreadable', circuitId: 'use', argumentIndex: 1 });
  expect(
    checkConformance(declared, {
      use: { pure: false, provable: true, argumentTypes: [unrelated, field], resultType: field },
    }),
  ).toMatchObject({ outcome: 'Unreadable', circuitId: 'use', argumentIndex: 0 });
  expect(
    checkConformance(declared, {
      use: {
        pure: false,
        provable: true,
        argumentTypes: [malformed({ tag: 'Tuple', types: [...expanded.types.slice(0, -1), null] }), field],
        resultType: field,
      },
    }),
  ).toMatchObject({ outcome: 'Unreadable', circuitId: 'use', argumentIndex: 0 });
  const cycle: { tag: string; name: string; type?: unknown } = { tag: 'Alias', name: 'Loop' };
  cycle.type = cycle;
  expect(
    checkConformance(declared, {
      use: {
        pure: false,
        provable: true,
        argumentTypes: [{ tag: 'Tuple', types: [...expanded.types.slice(0, -1), malformed(cycle)] }, field],
        resultType: field,
      },
    }),
  ).toMatchObject({ outcome: 'Unreadable', circuitId: 'use', unreadableTag: 'Alias', argumentIndex: 0 });
});

test('shared explicit Tuples compare once per type pair instead of expanding exponentially', () => {
  let declaredType: SignatureType = field;
  let implementedType: SignatureType = field;
  for (let i = 0; i < 25; i += 1) {
    declaredType = { tag: 'Vector', length: 2, type: declaredType };
    implementedType = { tag: 'Tuple', types: [implementedType, implementedType] };
  }
  expect(
    checkConformance({ use: { argumentTypes: [declaredType], resultType: field } }, implementation(implementedType)),
  ).toEqual({ outcome: 'Conformant' });
});

test('a shared mismatching Tuple DAG is inspected once per type pair', () => {
  let declaredType: SignatureType = field;
  let implementedType: SignatureType = { tag: 'Boolean' };
  for (let i = 0; i < 25; i += 1) {
    declaredType = { tag: 'Vector', length: 2, type: declaredType };
    const child = implementedType;
    implementedType = { tag: 'Tuple', types: [child, child] };
  }
  expect(
    checkConformance({ use: { argumentTypes: [declaredType], resultType: field } }, implementation(implementedType)),
  ).toEqual({ outcome: 'Violation', circuitId: 'use', check: 'ArgumentType', argumentIndex: 0 });
});

test('a shared Tuple DAG with a mismatching Alias leaf is inspected once per type pair', () => {
  let declaredType: SignatureType = field;
  let implementedType: SignatureType = { tag: 'Alias', name: 'Other', type: field };
  for (let i = 0; i < 25; i += 1) {
    declaredType = { tag: 'Vector', length: 2, type: declaredType };
    const child = implementedType;
    implementedType = { tag: 'Tuple', types: [child, child] };
  }
  expect(
    checkConformance({ use: { argumentTypes: [declaredType], resultType: field } }, implementation(implementedType)),
  ).toEqual({ outcome: 'Violation', circuitId: 'use', check: 'ArgumentType', argumentIndex: 0 });
});

test('a matching diamond DAG reuses inspected pairs across different parents', () => {
  let declaredType: SignatureType = field;
  let left: SignatureType = { tag: 'Field' };
  let right: SignatureType = { tag: 'Field' };
  const tuple = (a: SignatureType, b: SignatureType): SignatureType => ({ tag: 'Tuple', types: [a, b] });
  for (let i = 0; i < 25; i += 1) {
    declaredType = { tag: 'Vector', length: 2, type: declaredType };
    [left, right] = [tuple(left, right), tuple(left, right)];
  }
  expect(checkConformance({ use: { argumentTypes: [declaredType], resultType: field } }, implementation(left))).toEqual({
    outcome: 'Conformant',
  });
});

test('a composite mismatch before a matching diamond does not disable later memoization', () => {
  let declaredType: SignatureType = field;
  let left: SignatureType = { tag: 'Field' };
  let right: SignatureType = { tag: 'Field' };
  const tuple = (a: SignatureType, b: SignatureType): SignatureType => ({ tag: 'Tuple', types: [a, b] });
  for (let i = 0; i < 25; i += 1) {
    declaredType = { tag: 'Vector', length: 2, type: declaredType };
    [left, right] = [tuple(left, right), tuple(left, right)];
  }
  const declared: SignatureType = { tag: 'Tuple', types: [field, declaredType] };
  const implemented: SignatureType = { tag: 'Tuple', types: [{ tag: 'Alias', name: 'Different', type: field }, left] };
  expect(checkConformance({ use: { argumentTypes: [declared], resultType: field } }, implementation(implemented))).toEqual({
    outcome: 'Violation',
    circuitId: 'use',
    check: 'ArgumentType',
    argumentIndex: 0,
  });
});

test('a composite fallback rejects an accessor before it can mutate a cached sibling', () => {
  const alias: SignatureType = { tag: 'Alias', name: 'A', type: field };
  const declared: SignatureType = { tag: 'Tuple', types: [alias, field, alias] };
  const shared = { tag: 'Alias', name: 'A', type: field as SignatureType };
  let reads = 0;
  const elements = [{ name: 'item', type: field }];
  Object.defineProperty(elements, 0, {
    configurable: true,
    enumerable: true,
    get() {
      reads += 1;
      shared.type = malformed({ tag: 'Summer' });
      return { name: 'item', type: field };
    },
  });
  const mutatingStruct = malformed({
    tag: 'Struct',
    name: 'Other',
    elements,
  });
  const implemented = malformed({ tag: 'Tuple', types: [shared, mutatingStruct, shared] });
  expect(checkConformance({ use: { argumentTypes: [declared], resultType: field } }, implementation(implemented))).toEqual({
    outcome: 'Unreadable',
    circuitId: 'use',
    unreadableTag: 'Struct',
    argumentIndex: 0,
  });
  expect(reads).toEqual(0);
});

test('a nested Proxy get trap cannot mutate a memoized Alias into false conformance', () => {
  const alias: SignatureType = { tag: 'Alias', name: 'A', type: field };
  const shared = { tag: 'Alias', name: 'A', type: field as SignatureType };
  let reads = 0;
  const middle = malformed(
    new Proxy(
      { tag: 'Vector', length: 0, type: field },
      {
        get(target, key, receiver) {
          if (key === 'type') {
            reads += 1;
            shared.type = malformed({ tag: 'Summer' });
          }
          return Reflect.get(target, key, receiver);
        },
      },
    ),
  );
  const declared: SignatureType = { tag: 'Tuple', types: [alias, { tag: 'Tuple', types: [] }, alias] };
  const implemented: SignatureType = { tag: 'Tuple', types: [shared, middle, shared] };
  expect(checkConformance({ use: { argumentTypes: [declared], resultType: field } }, implementation(implemented))).toEqual({
    outcome: 'Unreadable',
    circuitId: 'use',
    unreadableTag: '(malformed)',
    argumentIndex: 0,
  });
  expect(reads).toEqual(0);
});

test('a nested Proxy descriptor trap cannot mutate a prior Alias into false conformance', () => {
  const alias: SignatureType = { tag: 'Alias', name: 'A', type: field };
  const shared = { tag: 'Alias', name: 'A', type: field as SignatureType };
  let reads = 0;
  const middle = malformed(
    new Proxy(
      { tag: 'Vector', length: 0, type: field },
      {
        getOwnPropertyDescriptor(target, key) {
          if (key === 'type') {
            reads += 1;
            shared.type = malformed({ tag: 'Summer' });
          }
          return Reflect.getOwnPropertyDescriptor(target, key);
        },
      },
    ),
  );
  const declared: SignatureType = { tag: 'Tuple', types: [alias, { tag: 'Tuple', types: [] }, alias] };
  const implemented: SignatureType = { tag: 'Tuple', types: [shared, middle, shared] };
  expect(checkConformance({ use: { argumentTypes: [declared], resultType: field } }, implementation(implemented))).toEqual({
    outcome: 'Unreadable',
    circuitId: 'use',
    unreadableTag: '(malformed)',
    argumentIndex: 0,
  });
  expect(reads).toBeGreaterThan(0);
});

test('an unavailable structuredClone refuses implementation metadata', () => {
  vi.stubGlobal('structuredClone', undefined);
  try {
    expect(checkConformance(declaration, implementation(field))).toEqual({
      outcome: 'Unreadable',
      circuitId: 'use',
      unreadableTag: '(malformed)',
      argumentIndex: 0,
    });
  } finally {
    vi.unstubAllGlobals();
  }
});

test('a shared cyclic Tuple still reports unreadability before using a memoized sibling', () => {
  const expected: SignatureType = { tag: 'Vector', length: 2, type: { tag: 'Vector', length: 2, type: field } };
  const cyclic = malformed({ tag: 'Tuple', types: [] as SignatureType[] });
  (cyclic as { types: SignatureType[] }).types.push(field, cyclic);
  const implemented = malformed({ tag: 'Tuple', types: [cyclic, cyclic] });
  expect(checkConformance({ use: { argumentTypes: [expected], resultType: field } }, implementation(implemented))).toEqual({
    outcome: 'Unreadable',
    circuitId: 'use',
    unreadableTag: 'Tuple',
    argumentIndex: 0,
  });
});

test('a large expanded result Vector conforms under its own validation allowance', () => {
  const length = 12_000;
  const expected: SignatureType = { tag: 'Vector', length, type: field };
  const expanded: SignatureType = { tag: 'Tuple', types: Array<SignatureType>(length).fill(field) };
  expect(
    checkConformance(
      { use: { argumentTypes: [], resultType: expected } },
      { use: { pure: false, provable: true, argumentTypes: [], resultType: expanded } },
    ),
  ).toEqual({ outcome: 'Conformant' });
});

test('large Vector/Tuple pairs remain conformant inside aliases, structs and contracts', () => {
  const length = 12_000;
  const vector: SignatureType = { tag: 'Vector', length, type: field };
  const tuple: SignatureType = { tag: 'Tuple', types: Array.from({ length }, (): SignatureType => ({ tag: 'Field' })) };
  const cases: readonly (readonly [string, SignatureType, SignatureType])[] = [
    ['Alias', { tag: 'Alias', name: 'Values', type: vector }, { tag: 'Alias', name: 'Values', type: tuple }],
    [
      'Struct',
      { tag: 'Struct', name: 'Values', elements: [{ name: 'items', type: vector }] },
      { tag: 'Struct', name: 'Values', elements: [{ name: 'items', type: tuple }] },
    ],
    [
      'Contract',
      { tag: 'Contract', name: 'Declared', circuits: { read: { argumentTypes: [vector], resultType: vector } } },
      { tag: 'Contract', name: 'Implemented', circuits: { read: { argumentTypes: [tuple], resultType: tuple } } },
    ],
  ];
  for (const [name, expected, received] of cases) {
    expect(checkConformance({ use: { argumentTypes: [expected], resultType: field } }, implementation(received)), name).toEqual({
      outcome: 'Conformant',
    });
  }
});

test('a mismatching tuple cannot force unbounded work over later untrusted elements', () => {
  const fields = Array.from({ length: 5_000 }, (_, i) => ({ name: `f${i}`, type: field }));
  let inspected = 0;
  const elements = Array.from({ length: 2_000 }, () =>
    malformed(
      new Proxy(
        { tag: 'Struct', name: 'Unrelated', elements: fields },
        {
          get(target, key, receiver) {
            if (key === 'elements') inspected += 1;
            return Reflect.get(target, key, receiver);
          },
        },
      ),
    ),
  );
  const expected: SignatureType = { tag: 'Vector', length: elements.length, type: field };
  const received: SignatureType = { tag: 'Tuple', types: elements };
  expect(checkConformance({ use: { argumentTypes: [expected], resultType: field } }, implementation(received))).toMatchObject({
    outcome: 'Unreadable',
    circuitId: 'use',
    argumentIndex: 0,
  });
  expect(inspected).toBeLessThan(30);
});

test('a matching expanded struct field cannot lend its allowance to another field', () => {
  const length = 12_000;
  const vector: SignatureType = { tag: 'Vector', length, type: field };
  const tuple: SignatureType = { tag: 'Tuple', types: Array<SignatureType>(length).fill(field) };
  const declaredType: SignatureType = {
    tag: 'Struct',
    name: 'Pair',
    elements: [
      { name: 'items', type: vector },
      { name: 'other', type: field },
    ],
  };
  const unrelated = malformed({
    tag: 'Contract',
    name: 'Nested',
    circuits: { c: { argumentTypes: Array<SignatureType>(20_000).fill(field), resultType: field } },
  });
  const implementedType: SignatureType = {
    tag: 'Struct',
    name: 'Pair',
    elements: [
      { name: 'items', type: tuple },
      { name: 'other', type: unrelated },
    ],
  };
  expect(
    checkConformance({ use: { argumentTypes: [declaredType], resultType: field } }, implementation(implementedType)),
  ).toMatchObject({ outcome: 'Unreadable', circuitId: 'use', argumentIndex: 0 });
});

test('an unreadable later tuple member takes precedence over an earlier type mismatch', () => {
  const declared: InterfaceDescriptor = { use: { argumentTypes: [{ tag: 'Tuple', types: [field, field] }], resultType: field } };
  const implemented = malformed({ tag: 'Tuple', types: [{ tag: 'Boolean' }, { tag: 'Summer' }] });
  expect(checkConformance(declared, implementation(implemented))).toEqual({
    outcome: 'Unreadable',
    circuitId: 'use',
    unreadableTag: 'Summer',
    argumentIndex: 0,
  });
});

test('a later Struct field-name mismatch does not rewalk a valid large expanded field', () => {
  const length = 12_000;
  const vector: SignatureType = { tag: 'Vector', length, type: field };
  const tuple: SignatureType = { tag: 'Tuple', types: Array<SignatureType>(length).fill(field) };
  const declaredType: SignatureType = {
    tag: 'Struct',
    name: 'Pair',
    elements: [
      { name: 'big', type: vector },
      { name: 'other', type: field },
    ],
  };
  const implementedType: SignatureType = {
    tag: 'Struct',
    name: 'Pair',
    elements: [
      { name: 'big', type: tuple },
      { name: 'wrong', type: field },
    ],
  };
  expect(
    checkConformance({ use: { argumentTypes: [declaredType], resultType: field } }, implementation(implementedType)),
  ).toEqual({ outcome: 'Violation', circuitId: 'use', check: 'ArgumentType', argumentIndex: 0 });

  let nestedDeclared: SignatureType = field;
  let nestedImplemented: SignatureType = field;
  for (let i = 0; i < 14; i += 1) {
    nestedDeclared = { tag: 'Vector', length: 2, type: nestedDeclared };
    nestedImplemented = { tag: 'Tuple', types: [nestedImplemented, nestedImplemented] };
  }
  expect(
    checkConformance(
      {
        use: {
          argumentTypes: [
            {
              tag: 'Struct',
              name: 'Pair',
              elements: [
                { name: 'big', type: nestedDeclared },
                { name: 'other', type: field },
              ],
            },
          ],
          resultType: field,
        },
      },
      implementation({
        tag: 'Struct',
        name: 'Pair',
        elements: [
          { name: 'big', type: nestedImplemented },
          { name: 'wrong', type: field },
        ],
      }),
    ),
  ).toEqual({ outcome: 'Violation', circuitId: 'use', check: 'ArgumentType', argumentIndex: 0 });
});

test('getter-backed signatures are unreadable and cannot hide mutation behind memoization', () => {
  const alias: SignatureType = { tag: 'Alias', name: 'A', type: field };
  const declared: InterfaceDescriptor = { use: { argumentTypes: [{ tag: 'Tuple', types: [alias, alias] }], resultType: field } };
  let reads = 0;
  const mutable = malformed({
    tag: 'Alias',
    get name() {
      reads += 1;
      return reads <= 2 ? 'A' : 'B';
    },
    type: field,
  });
  const implemented = malformed({ tag: 'Tuple', types: [mutable, mutable] });
  expect(checkConformance(declared, implementation(implemented))).toEqual({
    outcome: 'Unreadable',
    circuitId: 'use',
    unreadableTag: 'Alias',
    argumentIndex: 0,
  });
  expect(reads).toEqual(0);
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
