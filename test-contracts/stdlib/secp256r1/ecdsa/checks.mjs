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
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import * as runtime from '@midnight-ntwrk/compact-runtime';
import { createTestContract } from '@test/compact-test';
import { assertCoverage, loadCorpus, runEcdsaKat, SECP256R1 } from '@test/crypto';
import { expect, test } from 'vitest';
import compileTest from '../ecdsa_verify/compile.pass.test.ts';
import runtimeTest from '../ecdsa_verify/runtime.pass.test.ts';
import { identity } from '../point/cases.mjs';
import { bytesHex, coordinateHex, encodePoint, identityError, signatureCases, zeroError } from './cases.mjs';
import { registerOrdinary } from './upstream-callback.mjs';

async function load(build) {
  return import(/* @vite-ignore */ pathToFileURL(join(build, 'contract/index.js')).href);
}

export function registerEcdsaChecks(compile) {
  test('runs the original P-256 ECDSA assertions with message binding', async () => {
    const { build, result } = compile('ecdsa/upstream-ordinary', {}, ['ts', 'rust']);
    expect(result.status, result.stderr).toBe(0);
    const module = await load(build);
    const entry = join(build, 'contract/index.js');
    const requireRuntime = createRequire(createRequire(entry).resolve('@midnight-ntwrk/compact-runtime'));
    const { p256 } = await import(/* @vite-ignore */ pathToFileURL(requireRuntime.resolve('@noble/curves/nist.js')).href);
    const { keccak_256 } = await import(/* @vite-ignore */ pathToFileURL(requireRuntime.resolve('@noble/hashes/sha3.js')).href);
    const start = async (_, witnesses, state) => {
      const { contract, ctx } = await createTestContract(module.Contract, witnesses, state);
      return [{ circuits: Object.fromEntries(Object.entries(contract.circuits)
        .map(([name, invoke]) => [name, async (...args) => invoke(...args)])) }, ctx];
    };
    const callbacks = [];
    registerOrdinary((_, callback) => callbacks.push(callback), expect, start, module, runtime, p256);
    expect(callbacks).toHaveLength(1);
    await callbacks[0]();
    const { contract, ctx } = await createTestContract(module.Contract);
    const cases = signatureCases(p256, keccak_256);
    for (const { message, sig, pk, expected } of cases) {
      const call = contract.circuits.verify(ctx, message, sig, pk);
      expect(call.result).toBe(expected);
      const cell = call.context.currentQueryContext.state.state.asArray()[0].asCell();
      expect(runtime.CompactTypeBoolean.fromValue(cell.value.slice())).toBe(expected);
    }
    const { message, sig, pk } = cases[0];
    expect(() => contract.circuits.verify(ctx, message, sig, identity)).toThrow(identityError);
    expect(() => contract.circuits.verify(ctx, message, { ...sig, s: 0n }, pk)).toThrow(zeroError);
    expect(() => contract.circuits.verify(ctx, message, sig, { ...identity, identity: false }))
      .toThrow(/expected value of type Secp256r1Point/);
  });

  test('runs the original P-256 Wycheproof runtime suite', async () => {
    expect(compileTest.result).toBe('pass');
    expect(runtimeTest.result).toBe('pass');
    const targets = ['ts', 'rust'];
    const intact = compile('ecdsa_verify', compileTest.options, targets);
    const name = 'secp256r1_ecdsa_verify.compact';
    const originalPath = new URL('../ecdsa_verify/' + name, import.meta.url);
    let source = fileURLToPath(originalPath);
    if (intact.result.status !== 0) {
      expect(process.env.COMPACT_SECP256R1_KAT_MODE).toBe('--allow-pragma-probe');
      expect(intact.result.stderr).toContain('language version 0.23.105 mismatch');
      expect(existsSync(intact.build)).toBe(false);
      const original = readFileSync(originalPath, 'utf8');
      const pragma = 'pragma language_version >= 0.25.0;\n';
      expect(original.split(pragma)).toHaveLength(2);
      source = join(dirname(intact.build), name);
      writeFileSync(source, original.replace(pragma, ''), { flag: 'wx' });
    }
    const built = compile('ecdsa/wycheproof', compileTest.options, targets, '--feature-zkir-v3', source);
    expect(built.result.status, built.result.stderr).toBe(0);
    const info = JSON.parse(readFileSync(join(built.build, 'compiler/contract-info.json'), 'utf8'));
    expect(info.circuits).toHaveLength(1);
    expect(info.circuits[0].proof).toBe(false);
    const { Contract, pureCircuits } = await load(built.build);
    await runtimeTest.run(Contract, pureCircuits);
    const corpus = runEcdsaKat(SECP256R1, 'prepare Rust P-256 vector inputs',
      ({ scalarsInRange, e, sig, pk }) => scalarsInRange && pureCircuits.verifyEcdsa(e, sig, pk));
    assertCoverage(SECP256R1.name, corpus, SECP256R1.coverage);
    const original = loadCorpus(SECP256R1.vectorsFile).testGroups.flatMap(({ tests }) => tests.map(({ tcId }) => tcId));
    const ids = [...corpus.driven.map(({ tcId }) => tcId), ...corpus.excluded.flatMap(({ tcIds }) => tcIds)];
    expect(new Set(ids).size).toBe(original.length);
    expect(ids.sort((a, b) => a - b)).toEqual(original.sort((a, b) => a - b));
    const driven = corpus.driven.map(({ tcId, expectation, scalarsInRange, msg, e, sig, pk }) => ({
      tcId, expectation, scalarsInRange, message: bytesHex(msg), digest: bytesHex(e),
      r: coordinateHex(sig.r), s: coordinateHex(sig.s), point: encodePoint(pk),
    }));
    writeFileSync(join(dirname(built.build), 'vectors.json'), JSON.stringify({ driven, excluded: corpus.excluded }),
      { flag: 'wx' });
    console.log(`P-256 Wycheproof: ${driven.length} driven, ${original.length - driven.length} excluded, ${original.length} total`);
  });
}
