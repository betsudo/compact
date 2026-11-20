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
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import * as runtime from '@midnight-ntwrk/compact-runtime';
import { createTestContract } from '@test/compact-test';
import { expect, test } from 'vitest';
import { identity, pointAtoms } from '../point/cases.mjs';
import { hashCases, pointTupleDigest, witnessCases } from './cases.mjs';
import { registerHash608, registerWitness } from './upstream-callbacks.mjs';

async function load(compile, fixture) {
  const { build, result } = compile(`transport/${fixture}`, {}, ['ts', 'rust']);
  expect(result.status, result.stderr).toBe(0);
  const entry = join(build, 'contract/index.js');
  const module = await import(/* @vite-ignore */ pathToFileURL(entry).href);
  const requireRuntime = createRequire(createRequire(entry).resolve('@midnight-ntwrk/compact-runtime'));
  return { module, requireRuntime };
}

function adaptedStart(module) {
  return async (_, witnesses, state) => {
    const { contract, ctx } = await createTestContract(module.Contract, witnesses, state);
    return [{ circuits: Object.fromEntries(Object.entries(contract.circuits)
      .map(([name, invoke]) => [name, async (...args) => invoke(...args)])) }, ctx];
  };
}

async function runOriginal(register, module) {
  const callbacks = [];
  register((_, callback) => callbacks.push(callback), expect, adaptedStart(module), module, runtime);
  expect(callbacks).toHaveLength(1);
  await callbacks[0]();
}

export function registerTransportChecks(compile) {
  test('runs the intact P-256 witness fixture and checks private-state transport', async () => {
    const { module } = await load(compile, 'upstream-witness');
    await runOriginal(registerWitness, module);
    for (const { circuit, callback, value, initial, step } of witnessCases) {
      const other = callback === 'point0' ? 'point1' : 'point0';
      const { contract, ctx } = await createTestContract(module.Contract, {
        [callback]: ({ privateState }) => [privateState + step, value],
        [other]: () => { throw new Error('unexpected witness callback'); },
      }, initial);
      const result = contract.circuits[circuit](ctx);
      expect(result.result).toEqual(value);
      expect(result.context.currentPrivateState).toBe(initial + step);
      expect(ctx.currentPrivateState).toBe(initial);
      expect(result.proofData.privateTranscriptOutputs).toEqual([{
        alignment: runtime.CompactTypeSecp256r1Point.alignment(),
        value: pointAtoms(value),
      }]);
      const malformed = { ...identity, identity: false };
      contract.witnesses[callback] = () => [initial + step, malformed];
      expect(() => contract.circuits[circuit](ctx)).toThrow(/return value.*Secp256r1Point/s);
      expect(ctx.currentPrivateState).toBe(initial);
      expect(module.ledger(ctx.currentQueryContext.state).point).toEqual(identity);
    }
  });

  test('runs upstream issue #608 and checks independent ordered point-tuple digests', async () => {
    const { module, requireRuntime } = await load(compile, 'upstream-hash608');
    await runOriginal(registerHash608, module);
    const { p256 } = await import(/* @vite-ignore */ pathToFileURL(requireRuntime.resolve('@noble/curves/nist.js')).href);
    const { keccak_256 } = await import(/* @vite-ignore */ pathToFileURL(requireRuntime.resolve('@noble/hashes/sha3.js')).href);
    const { contract, ctx } = await createTestContract(module.Contract);
    const bytes = new runtime.CompactTypeBytes(32);
    const digests = hashCases(p256).map(({ args }) => {
      const expected = pointTupleDigest(keccak_256, args);
      const call = contract.circuits.test(ctx, ...args);
      const cell = call.context.currentQueryContext.state.state.asArray()[0].asCell();
      expect(bytes.fromValue(cell.value.slice())).toEqual(expected);
      return Buffer.from(expected).toString('hex');
    });
    expect(new Set(digests).size).toBe(digests.length);
  });
}
