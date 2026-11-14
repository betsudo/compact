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
import { CompactError, SECP256K1_SCALAR_MODULUS, secp256k1Add, secp256k1Mul, secp256k1MulGenerator } from '@midnight-ntwrk/compact-runtime';
import { createTestContract } from '@test/compact-test';
import { expect, test } from 'vitest';
import { baseModulus, generator, five, seven, identity, identityWithCurveCoordinates } from '../point/cases.mjs';

const maxScalar = SECP256K1_SCALAR_MODULUS - 1n;
const minusG = { ...generator, y: baseModulus - generator.y };
const two = {
  x: 0xc6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5n,
  y: 0x1ae168fea63dc339a3c58419466ceaeef7f632653266d0e1236431a950cfe52an,
  identity: false,
};

async function load(compile, file) {
  const { build, result } = compile('group', { options: {} }, ['ts', 'rust'], file);
  expect(result.status, result.stderr).toBe(0);
  const { Contract, ledger, pureCircuits } = await import(/* @vite-ignore */ pathToFileURL(join(build, 'contract/index.js')).href);
  return { ...await createTestContract(Contract, {}, 0), ledger, pureCircuits };
}

export function registerGroupTests(compile) {
  test('preserves the original computed-identity refusals and finite coordinates', async () => {
    const { contract, ctx, ledger } = await load(compile, 'upstream-computed-identity.compact');
    const { storePoint, storeXOfGenerator, storeXOfMul, storeXOfSum } = contract.circuits;
    const G = secp256k1MulGenerator(1n);
    const negG = secp256k1Mul(G, maxScalar);
    const twoG = secp256k1MulGenerator(2n);
    expect(G).toEqual(generator);
    expect(negG).toEqual(minusG);
    expect(twoG).toEqual(two);
    expect(secp256k1Add(G, negG)).toEqual(identity);
    const stored = storePoint(ctx, G);
    for (const call of [
      () => storeXOfGenerator(ctx, 0n),
      () => storeXOfSum(ctx, G, negG),
      () => storeXOfMul(stored.context, 0n),
    ]) {
      expect(call).toThrow(CompactError);
      expect(call).toThrow('cannot extract the x-coordinate of the secp256k1 identity point');
    }
    const read = (result) => ledger(result.context.currentQueryContext.state).x;
    expect(read(storeXOfGenerator(ctx, 1n))).toEqual(G.x);
    expect(read(storeXOfSum(ctx, G, G))).toEqual(twoG.x);
    expect(read(storeXOfMul(stored.context, 2n))).toEqual(twoG.x);
  });

  test('writes canonical group points and preserves the Jubjub overload', async () => {
    const { contract, ctx, ledger, pureCircuits } = await load(compile, 'group-results.compact');
    const cases = [
      ['writeGenerator', 'generator', [0n], identity],
      ['writeGenerator', 'generator', [1n], generator],
      ['writeGenerator', 'generator', [5n], five],
      ['writeGenerator', 'generator', [7n], seven],
      ['writeGenerator', 'generator', [maxScalar], minusG],
      ['writeMul', 'multiply', [generator, 0n], identity],
      ['writeMul', 'multiply', [generator, 2n], two],
      ['writeMul', 'multiply', [generator, maxScalar], minusG],
      ['writeMul', 'multiply', [identityWithCurveCoordinates, 5n], identity],
      ['writeAdd', 'add', [generator, minusG], identity],
      ['writeAdd', 'add', [generator, identityWithCurveCoordinates], generator],
      ['writeAdd', 'add', [identityWithCurveCoordinates, identity], identity],
      ['writeAdd', 'add', [two, five], seven],
    ];
    for (const [write, pure, args, point] of cases) {
      expect(pureCircuits[pure](...args)).toStrictEqual(point);
      const call = contract.circuits[write](ctx, ...args);
      expect(ledger(call.context.currentQueryContext.state).result).toStrictEqual(point);
    }
    expect(pureCircuits.jubjubZero()).toStrictEqual({ x: 0n, y: 1n });
    expect(() => contract.circuits.writeGenerator(ctx, maxScalar + 1n)).toThrow(/expected value of type Secp256k1Scalar/);
  });

  for (const [file, name] of [
    ['refuse-mixed-add.compact', 'ecAdd'],
    ['refuse-base-mul.compact', 'ecMul'],
    ['refuse-base-generator.compact', 'ecMulGenerator'],
  ]) {
    for (const targets of [['ts'], ['rust'], ['ts', 'rust']]) {
      test(`${name} refuses mixed carriers for ${targets.join('+')}`, () => {
        const { build, result } = compile('group', { options: {} }, targets, file);
        expect(result.status).not.toBe(0);
        expect(result.stderr).toMatch(new RegExp(`no compatible function named ${name}`));
        expect(existsSync(build)).toBe(false);
      });
    }
  }
}
