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

import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { assertCoverage, loadAddressVectors, loadCorpus, runEcdsaKat, SECP256K1_BITCOIN } from '@test/crypto';
import { expect, test } from 'vitest';

const compileTests = import.meta.glob('../{ecdsa_verify,bitcoin_kat,ethereum_address}/compile.pass.test.ts', {
  eager: true,
  import: 'default',
});
const runtimeTests = import.meta.glob('../{ecdsa_verify,bitcoin_kat,ethereum_address}/runtime.pass.test.ts', {
  eager: true,
  import: 'default',
});

const fixtures = ['ecdsa_verify', 'bitcoin_kat', 'ethereum_address'];
const pragma = 'pragma language_version >= 0.25.0;\n';
const runtimeRoot = fileURLToPath(new URL('../../../..', import.meta.url));
const hex = (bytes) => Buffer.from(bytes).toString('hex');
const coordinate = (value) => value.toString(16).padStart(64, '0');
const point = ({ x, y, identity }) => ({ x: coordinate(x), y: coordinate(y), identity });

function ecdsaVectors(pure) {
  const suite = SECP256K1_BITCOIN;
  const corpus = runEcdsaKat(suite, 'prepare Rust vector inputs', ({ scalarsInRange, e, sig, pk }) =>
    scalarsInRange && pure.verifyEcdsa(e, sig, pk));
  assertCoverage(suite.name, corpus, suite.coverage);
  const original = loadCorpus(suite.vectorsFile).testGroups.flatMap(({ tests }) => tests.map(({ tcId }) => tcId));
  const ids = [
    ...corpus.driven.map(({ tcId }) => tcId),
    ...corpus.excluded.flatMap(({ tcIds }) => tcIds),
  ];
  expect(new Set(ids).size).toBe(ids.length);
  expect(ids.sort((a, b) => a - b)).toEqual(original.sort((a, b) => a - b));
  return {
    excluded: corpus.excluded,
    driven: corpus.driven.map(({ tcId, expectation, scalarsInRange, msg, e, sig, pk }) => ({
      tcId, expectation, scalarsInRange, message: hex(msg), digest: hex(e),
      r: coordinate(sig.r), s: coordinate(sig.s), point: point(pk),
    })),
  };
}

export function registerKnownAnswerTests(compile) {
  const probe = process.env.COMPACT_SECP256K1_KAT_MODE === '--allow-pragma-probe';
  for (const fixture of fixtures) {
    test(`runs the original ${fixture} known-answer suite`, async () => {
      const compileTest = compileTests[`../${fixture}/compile.pass.test.ts`];
      const runtimeTest = runtimeTests[`../${fixture}/runtime.pass.test.ts`];
      expect(compileTest.result).toBe('pass');
      expect(runtimeTest.result).toBe('pass');
      const definition = {
        ...compileTest,
        options: { ...compileTest.options, compilerArgs: [
          ...(compileTest.options.compilerArgs ?? []),
          '--rust-runtime-root', runtimeRoot,
        ] },
      };
      const targets = ['ts', 'rust'];
      let source;
      if (probe) {
        const intact = compile(fixture, definition, targets, undefined, { lane: 'intact' });
        expect(intact.result.status).not.toBe(0);
        expect(intact.result.stderr).toContain('language version 0.23.105 mismatch');
        expect(existsSync(intact.build)).toBe(false);
        const name = `secp256k1_${fixture}.compact`;
        const original = readFileSync(new URL(`../${fixture}/${name}`, import.meta.url), 'utf8');
        expect(original.split(pragma)).toHaveLength(2);
        source = join(dirname(intact.build), name);
        writeFileSync(source, original.replace(pragma, ''), { flag: 'wx' });
      }
      const { build, result } = compile(fixture, definition, targets, undefined, { source });
      expect(result.status, result.stderr).toBe(0);
      const info = JSON.parse(readFileSync(join(build, 'compiler/contract-info.json'), 'utf8'));
      expect(info.circuits.length).toBeGreaterThan(0);
      expect(info.circuits.every(({ proof }) => proof === false)).toBe(true);
      const entry = pathToFileURL(join(build, 'contract/index.js')).href;
      const { Contract, pureCircuits } = await import(/* @vite-ignore */ entry);
      await runtimeTest.run(Contract, pureCircuits);
      if (fixture === 'ecdsa_verify') {
        writeFileSync(join(dirname(build), 'vectors.json'), JSON.stringify(ecdsaVectors(pureCircuits)), { flag: 'wx' });
      }
      if (fixture === 'ethereum_address') {
        const vectors = loadAddressVectors().map(({ seed, point: publicKey, ethAddr }) => ({
          seed, point: point(publicKey), address: ethAddr.slice(2),
        }));
        writeFileSync(join(dirname(build), 'vectors.json'), JSON.stringify(vectors), { flag: 'wx' });
      }
    });
  }
}
