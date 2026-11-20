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
import * as runtime from '@midnight-ntwrk/compact-runtime';
import { createTestContract } from '@test/compact-test';
import { expect, test } from 'vitest';
import { upstreamCallbacks } from './upstream-callbacks.mjs';

async function startContract(contractCode, witnesses, privateState) {
  const { contract, ctx } = await createTestContract(contractCode.Contract, witnesses, privateState);
  return [contract, ctx];
}

async function open(compile, fixture) {
  const { build, result } = compile(`casts/${fixture}`, {}, ['ts', 'rust']);
  expect(result.status, result.stderr).toBe(0);
  return import(/* @vite-ignore */ pathToFileURL(join(build, 'contract/index.js')).href);
}

export function registerByteCastChecks(compile) {
  for (const [fixture, expected, column] of [
    ['width-encode-base', 'Secp256r1Base to type Bytes<31>', 59],
    ['width-encode-scalar', 'Secp256r1Scalar to type Bytes<33>', 61],
    ['width-decode-base', 'Bytes<33> to type Secp256r1Base', 60],
    ['width-decode-scalar', 'Bytes<31> to type Secp256r1Scalar', 62],
    ['base-scalar', 'Secp256r1Base to type Secp256r1Scalar'],
    ['scalar-k1', 'Secp256r1Scalar to type Secp256k1Scalar'],
  ]) {
    for (const targets of [['ts'], ['rust'], ['ts', 'rust']]) {
      test(`${fixture} refuses compilation for ${targets.join('+')}`, () => {
        const { build, result } = compile(`casts/${fixture}`, {}, targets);
        expect(result.status).not.toBe(0);
        expect(result.stderr).toContain(`cannot cast from type ${expected}`);
        if (column) expect(result.stderr).toContain(`line 17 char ${column}`);
        expect(existsSync(build)).toBe(false);
      });
    }
  }
  for (const [fixture, register] of Object.entries(upstreamCallbacks)) {
    test(`runs the original ${fixture} callback`, async () => {
      const contractCode = await open(compile, fixture);
      const callbacks = [];
      register((_, callback) => callbacks.push(callback), expect, startContract, contractCode, runtime);
      expect(callbacks).toHaveLength(1);
      await callbacks[0]();
    });
  }
  test('executes the original guarded save', async () => {
    const code = await open(compile, 'guarded-save');
    const [contract, context] = await startContract(code, {}, 0);
    const saved = contract.circuits.save(context, true, 256n);
    const skipped = contract.circuits.save(saved.context, false, runtime.MAX_SECP256R1_BASE);
    const expected = new Uint8Array(32);
    expected[1] = 1;
    expect(code.ledger(saved.context.currentQueryContext.state).out).toEqual(expected);
    expect(code.ledger(skipped.context.currentQueryContext.state).out).toEqual(expected);
  });
  test('runs the P-256 portions of the original constant-cast assertions', async () => {
    const contractCode = await open(compile, 'constant');
    const [C, Ctxt] = await startContract(contractCode, {}, 0);
    const L = (r) => contractCode.ledger(r.context.currentQueryContext.state);
    const V = (1n << 248n) + 5n;
    let l = L(await C.circuits.castR1(Ctxt, false, 7n, 8n));
    expect([l.r1b, l.r1s]).toEqual([V, V]);
    l = L(await C.circuits.castR1(Ctxt, true, 7n, 8n));
    expect([l.r1b, l.r1s]).toEqual([7n, 8n]);
    l = L(await C.circuits.castZero(Ctxt));
    expect([l.r1b, l.r1s]).toEqual([0n, 0n]);
  });
  test('runs the P-256 portions of the original unused-cast assertions', async () => {
    const contractCode = await open(compile, 'unused');
    const [C, Ctxt] = await startContract(contractCode, {}, 0);
    const n = (r) => contractCode.ledger(r.context.currentQueryContext.state).n;
    const v = new Uint8Array(32).fill(7);
    expect(n(await C.circuits.unusedR1(Ctxt, v))).toEqual(2n);
    expect(n(await C.circuits.unusedFieldR1(Ctxt, v, 5n))).toEqual(5n);
  });
}
