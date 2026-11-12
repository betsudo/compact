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
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import * as runtime from '@midnight-ntwrk/compact-runtime';
import { createTestContract } from '@test/compact-test';
import { expect, test } from 'vitest';
import { pointCases, identity } from '../point/cases.mjs';
import { generator, variedIdentity, xZero } from './cases.mjs';
import { registerRoundtrip, registerDirect, registerLedger, registerAsserted } from './upstream-callbacks.mjs';

async function load(compile, fixture) {
  const { build, result } = compile(fixture, {}, ['ts', 'rust']);
  expect(result.status, result.stderr).toBe(0);
  const entry = join(build, 'contract/index.js');
  const module = await import(/* @vite-ignore */ pathToFileURL(entry).href);
  const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
  const noble = createRequire(runtimeEntry).resolve('@noble/curves/nist.js');
  const { p256 } = await import(/* @vite-ignore */ pathToFileURL(noble).href);
  return { module, cases: pointCases(p256) };
}

async function startContract(module, witnesses, state) {
  const { contract, ctx } = await createTestContract(module.Contract, witnesses, state);
  // Upstream callbacks expect asynchronous circuit calls. Preserve their
  // assertions and adapt only the call boundary.
  return [{ circuits: Object.fromEntries(Object.entries(contract.circuits)
    .map(([name, invoke]) => [name, async (...args) => invoke(...args)])) }, ctx];
}

export function registerAccessorChecks(compile) {
  for (const [fixture, register] of [
    ['point/upstream-roundtrip', registerRoundtrip],
    ['accessors/upstream-direct', registerDirect],
    ['accessors/upstream-ledger', registerLedger],
    ['accessors/upstream-asserted', registerAsserted],
  ]) {
    test(`${fixture} runs its original P-256 assertions`, async () => {
      const { module, cases } = await load(compile, fixture);
      const callbacks = [];
      register((_, callback) => callbacks.push(callback), expect, startContract, module,
        { ...runtime, secp256r1MulGenerator: cases.generator });
      expect(callbacks).toHaveLength(1);
      await callbacks[0]();
    });
  }

  test('guarded P-256 accessors preserve identity errors and ledger state', async () => {
    const { module } = await load(compile, 'accessors/guarded-writes');
    const { contract, ctx } = await createTestContract(module.Contract);
    for (const [name, slot] of [['guardedX', 'x'], ['guardedY', 'y']]) {
      const finite = contract.circuits[name](ctx, true, generator);
      expect(finite.result).toBe(generator[slot]);
      expect(module.ledger(finite.context.currentQueryContext.state)[slot]).toBe(generator[slot]);
      expect(contract.circuits[name](ctx, true, xZero).result).toBe(xZero[slot]);
      for (const point of [identity, variedIdentity]) {
        const inactive = contract.circuits[name](ctx, false, point);
        expect(inactive.result).toBe(0n);
        expect(module.ledger(inactive.context.currentQueryContext.state)[slot]).toBe(0n);
        expect(() => contract.circuits[name](finite.context, true, point)).toThrow(
          `cannot extract the ${slot}-coordinate of the secp256r1 identity point`);
        expect(module.ledger(finite.context.currentQueryContext.state)[slot]).toBe(generator[slot]);
      }
    }
  });

  for (const targets of [['ts'], ['rust'], ['ts', 'rust']]) {
    test(`P-256 access refuses the K1 point type for ${targets.join('+')}`, () => {
      const { build, result } = compile('accessors/wrong-curve', {}, targets);
      expect(result.status).not.toBe(0);
      expect(result.stderr).toMatch(/no compatible function named secp256r1PointX[\s\S]*supplied argument types:\s+\(Secp256k1Point\)/);
      expect(existsSync(build)).toBe(false);
    });
  }
}
