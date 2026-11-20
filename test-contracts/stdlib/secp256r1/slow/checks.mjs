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
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { createTestContract } from '@test/compact-test';
import { expect, test } from 'vitest';
import shaCompile from '../sha256_kat/compile.pass.test.ts';
import shaRuntime from '../sha256_kat/runtime.pass.test.ts';
import sequentialCompile from './verify_sequential/compile.pass.test.ts';
import { compileReleaseFixture } from '../release-fixture.mjs';
import { identity } from '../point/cases.mjs';
import { identityError, zeroError } from '../ecdsa/cases.mjs';
import { sequentialCases, witnessInputs, witnessOrder, witnesses } from './cases.mjs';

async function load(build) {
  return import(/* @vite-ignore */ pathToFileURL(join(build, 'contract/index.js')).href);
}

export function registerSignatureSuites(compile) {
  test('runs the original P-256 in-circuit SHA-256 known-answer suite', async () => {
    expect(shaCompile.result).toBe('pass');
    expect(shaRuntime.result).toBe('pass');
    const { build } = compileReleaseFixture(compile, 'sha256_kat', 'secp256r1_sha256_kat.compact',
      'signature-suites/sha256', shaCompile.options);
    const info = JSON.parse(readFileSync(join(build, 'compiler/contract-info.json'), 'utf8'));
    const lengths = [0, 3, 5, 6, 7, 8, 9, 10, 11, 12, 20];
    expect(info.circuits.map(({ name }) => name)).toEqual(lengths.map((length) => `proveEcdsa${length}`));
    expect(info.circuits.every(({ proof }) => !proof)).toBe(true);
    const { Contract, pureCircuits } = await load(build);
    await shaRuntime.run(Contract, pureCircuits);
    const runtime = createRequire(join(build, 'contract/index.js')).resolve('@midnight-ntwrk/compact-runtime');
    const noble = createRequire(runtime);
    const { p256 } = await import(/* @vite-ignore */ pathToFileURL(noble.resolve('@noble/curves/nist.js')).href);
    const { sha256 } = await import(/* @vite-ignore */ pathToFileURL(noble.resolve('@noble/hashes/sha2.js')).href);
    const key = p256.Point.Fn.toBytes(7n);
    const { x, y } = p256.Point.fromBytes(p256.getPublicKey(key)).toAffine();
    const pk = { x, y, identity: false };
    for (const message of [new Uint8Array(), Uint8Array.of(1, 2, 3)]) {
      const { r, s } = p256.Signature.fromBytes(p256.sign(sha256(message), key, { prehash: false }));
      const run = pureCircuits[`proveEcdsa${message.length}`];
      expect(run(message, { r, s }, pk)).toBe(true);
      if (message.length === 0) {
        expect(() => run(Uint8Array.of(1), { r, s }, pk)).toThrow(/Bytes<0>/);
      } else {
        expect(run(Uint8Array.of(2, 2, 3), { r, s }, pk)).toBe(false);
      }
    }
  });

  test('runs both intact P-256 sequential verification bodies and checks witness boundaries', async () => {
    expect(sequentialCompile.result).toBe('pass');
    const { build } = compileReleaseFixture(compile, 'slow/verify_sequential', 'secp256r1_verify_sequential.compact',
      'signature-suites/sequential', sequentialCompile.options);
    const module = await load(build);
    const entry = join(build, 'contract/index.js');
    const runtime = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
    const { p256 } = await import(/* @vite-ignore */ pathToFileURL(createRequire(runtime).resolve('@noble/curves/nist.js')).href);
    const cases = sequentialCases(p256);
    for (const [circuit, count] of [['two', 2], ['three', 3]]) {
      const invoke = async (inputs) => {
        const calls = [];
        const { contract, ctx } = await createTestContract(module.Contract, witnesses(inputs, calls), 0);
        const result = contract.circuits[circuit](ctx, cases.digest);
        expect(ctx.currentPrivateState).toBe(0);
        expect(result.context.currentPrivateState).toBe(2 * count);
        expect(calls).toEqual(witnessOrder(count));
        return module.ledger(result.context.currentQueryContext.state).n;
      };
      expect(await invoke(witnessInputs(cases, count))).toBe(1n);
      for (let slot = 0; slot < count; slot++) {
        const other = (slot + 1) % cases.points.length;
        const changed = witnessInputs(cases, count);
        changed.signatures[slot] = cases.signatures[other];
        expect(await invoke(changed)).toBe(0n);
        const changedKey = witnessInputs(cases, count);
        changedKey.points[slot] = cases.points[other];
        expect(await invoke(changedKey)).toBe(0n);
      }
      const zero = witnessInputs(cases, count);
      zero.signatures[0] = { ...zero.signatures[0], s: 0n };
      await expect(invoke(zero)).rejects.toThrow(zeroError);
      const atInfinity = witnessInputs(cases, count);
      atInfinity.points[0] = identity;
      await expect(invoke(atInfinity)).rejects.toThrow(identityError);
    }
  });
}
