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

import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { createTestContract } from '@test/compact-test';
import { expect, test } from 'vitest';

import { fieldCases, selfSubtractionCases } from './cases.mjs';

const directory = 'arithmetic';
const definitions = { options: {} };

async function loadContract(compile, file) {
  const { build, result } = compile(directory, definitions, ['ts', 'rust'], file);
  expect(result.status, result.stderr).toBe(0);
  const entry = pathToFileURL(join(build, 'contract', 'index.js')).href;
  const { Contract } = await import(/* @vite-ignore */ entry);
  return createTestContract(Contract, {}, 0);
}

export function registerArithmeticTests(compile) {
  test('runs all upstream secp256k1 field arithmetic assertions', async () => {
    const { contract, ctx } = await loadContract(compile, 'field-arithmetic.compact');
    expect(fieldCases).toHaveLength(34);
    for (const [circuit, args, expected] of fieldCases) {
      const result = await contract.circuits[circuit](ctx, ...args);
      expect(result.result, `${circuit}(${args.join(', ')})`).toEqual(expected);
    }
  });

  test('runs the secp256k1 self-subtraction assertions from upstream issue #866', async () => {
    const { contract, ctx } = await loadContract(compile, 'self-subtraction.compact');
    expect(selfSubtractionCases).toHaveLength(3);
    for (const [circuit, args, expected] of selfSubtractionCases) {
      const result = await contract.circuits[circuit](ctx, ...args);
      expect(result.result, `${circuit}(${args.join(', ')})`).toEqual(expected);
    }
  });

  test('preserves field arithmetic through aliases, nominal types, and guarded writes', async () => {
    const { contract, ctx } = await loadContract(compile, 'alias-guard.compact');
    const initial = await contract.circuits.guarded(ctx, true, 3n, 5n);
    expect(initial.result).toBe(16n);
    expect((await contract.circuits.guarded(initial.context, false, 6n, 7n)).result).toBe(16n);
    expect((await contract.circuits.guarded(initial.context, true, 6n, 7n)).result).toBe(26n);
    expect((await contract.circuits.nominal(ctx, 9n, 9n)).result).toBe(0n);
  });

  const invalidOperands = [
    ['mixed-nominal.compact', /incompatible combination of types ScalarValue and Secp256k1Scalar for binary arithmetic\s+operator -/],
    ['mixed-fields.compact', /incompatible combination of types Secp256k1Base and Secp256k1Scalar for binary arithmetic\s+operator \+/],
    ['mixed-native-field.compact', /incompatible combination of types Secp256k1Scalar and Field for binary arithmetic\s+operator -/],
    ['mixed-uint.compact', /incompatible combination of types Uint<8> and Secp256k1Base for binary arithmetic\s+operator \*/],
  ];
  for (const [file, diagnostic] of invalidOperands) {
    for (const targets of [['ts'], ['rust'], ['ts', 'rust']]) {
      test(`${file} refuses compilation for ${targets.join('+')}`, () => {
        const { build, result } = compile(directory, definitions, targets, file);
        expect(result.status).not.toBe(0);
        expect(result.stderr).toMatch(diagnostic);
        expect(existsSync(build)).toBe(false);
      });
    }
  }
}
