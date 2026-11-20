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

import { existsSync, readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import * as runtime from '@midnight-ntwrk/compact-runtime';
import { createTestContract } from '@test/compact-test';
import { expect, test } from 'vitest';
import { identity, pointCases } from './cases.mjs';
import { registerEquality, registerIdentity } from './upstream-callbacks.mjs';

async function load(compile, fixture) {
  const { build, result } = compile(`point/${fixture}`, {}, ['ts', 'rust']);
  expect(result.status, result.stderr).toBe(0);
  const entry = join(build, 'contract/index.js');
  const module = await import(/* @vite-ignore */ pathToFileURL(entry).href);
  const runtimePath = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
  const noble = createRequire(runtimePath).resolve('@noble/curves/nist.js');
  const { p256 } = await import(/* @vite-ignore */ pathToFileURL(noble).href);
  return { build, module, cases: pointCases(p256) };
}

async function startContract(code, witnesses, state) {
  const { contract, ctx } = await createTestContract(code.Contract, witnesses, state);
  return [contract, ctx];
}

export function registerPointChecks(compile) {
  for (const [fixture, register] of [
    ['upstream-equality', registerEquality],
    ['upstream-identity', registerIdentity],
  ]) {
    test(`${fixture} runs its original P-256 assertions`, async () => {
      const { module, cases } = await load(compile, fixture);
      const callbacks = [];
      register((_, callback) => callbacks.push(callback), expect, startContract, module,
        { secp256r1MulGenerator: cases.generator });
      expect(callbacks).toHaveLength(1);
      await callbacks[0]();
      const [contract, ctx] = await startContract(module, {}, 0);
      if (fixture === 'upstream-equality') {
        for (const bad of cases.malformed) {
          expect(runtime.isValidSecp256r1Point(bad)).toBe(false);
          expect(() => contract.circuits.pointsEqual(ctx, bad, cases.five)).toThrow(/expected value of type Secp256r1Point/);
          expect(() => contract.circuits.pointsEqual(ctx, cases.five, bad)).toThrow(/expected value of type Secp256r1Point/);
        }
      } else {
        const finite = contract.circuits.storePoint(ctx, cases.five);
        expect(finite.result).toEqual(cases.five);
        expect(module.ledger(finite.context.currentQueryContext.state).point).toEqual(cases.five);
        expect(contract.circuits.hashPoint(ctx, cases.five).result)
          .not.toEqual(contract.circuits.hashPoint(ctx, identity).result);
      }
    });
  }

  test('the P-256 point descriptor preserves the five-atom prefix', async () => {
    const { build, module, cases } = await load(compile, 'upstream-tags');
    const descriptor = runtime.CompactTypeSecp256r1Point;
    expect(descriptor.alignment()).toEqual([
      ...[24, 8, 24, 8].map((length) => ({ tag: 'atom', value: { tag: 'bytes', length } })),
      { tag: 'atom', value: { tag: 'field' } },
    ]);
    const suffix = new Uint8Array([9]);
    for (const value of [cases.five, cases.seven, ...cases.variedIdentities]) {
      const encoded = descriptor.toValue(value);
      expect(encoded).toHaveLength(5);
      const joined = encoded.concat([suffix]);
      expect(descriptor.fromValue(joined)).toEqual(value.identity ? identity : value);
      expect(joined).toEqual([suffix]);
      expect(() => descriptor.fromValue(encoded.slice(0, 4))).toThrow(runtime.CompactError);
      expect(() => descriptor.fromValue(encoded.slice(0, 4).concat([new Uint8Array([2])])))
        .toThrow(/expected Secp256r1Point/);
    }
    const info = JSON.parse(readFileSync(join(build, 'compiler/contract-info.json')));
    const circuit = info.circuits.find(({ name }) => name === 'curves');
    expect(circuit.arguments.map(({ type }) => type['type-name']))
      .toEqual(['Secp256r1Base', 'Secp256r1Scalar', 'Secp256r1Point']);
    const ir = JSON.parse(readFileSync(join(build, 'contract/compact-rust-ir.json')));
    expect(ir.circuits.find(({ name }) => name === 'curves').parameters.map(({ ty }) => ty.kind))
      .toEqual(['secp256r1_base', 'secp256r1_scalar', 'secp256r1_point']);
    // Original pinned upstream JavaScript assertion from upstream.ss.txt.
    expect(module.circuitSignatures.curves.argumentTypes).toEqual([
      { tag: 'Secp256r1Base' },
      { tag: 'Secp256r1Scalar' },
      { tag: 'Secp256r1Point' },
    ]);
  });

  for (const [fixture, reason] of [
    ['cross-curve', 'mismatch between actual return type Secp256k1Point and declared return type Secp256r1Point'],
    ['opaque-spoof', 'Secp256r1Point'],
  ]) {
    for (const targets of [['ts'], ['rust'], ['ts', 'rust']]) {
      test(`${fixture} refuses compilation for ${targets.join('+')}`, () => {
        const { build, result } = compile(`point/${fixture}`, {}, targets);
        expect(result.status).not.toBe(0);
        expect(result.stderr.replace(/\s+/g, ' ')).toContain(reason);
        expect(existsSync(build)).toBe(false);
      });
    }
  }
}
