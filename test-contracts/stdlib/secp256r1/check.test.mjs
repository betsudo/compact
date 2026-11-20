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

import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { MAX_SECP256R1_BASE, MAX_SECP256R1_SCALAR } from '@midnight-ntwrk/compact-runtime';
import { createTestContract } from '@test/compact-test';
import { expect, test } from 'vitest';

import roundtripCompile from './base/roundtrip/compile.pass.test.ts';
import roundtripRuntime from './base/roundtrip/runtime.pass.test.ts';
import { registerArithmeticChecks } from './arithmetic/checks.mjs';
import { registerAccessorChecks } from './accessors/checks.mjs';
import { registerSignatureSuites } from './slow/checks.mjs';
import { registerEcdsaChecks } from './ecdsa/checks.mjs';
import { registerTransportChecks } from './transport/checks.mjs';
import { registerGroupChecks } from './group/checks.mjs';
import { registerPointChecks } from './point/checks.mjs';
import { registerUnaryChecks } from './unary/checks.mjs';
import { registerByteCastChecks } from './casts/checks.mjs';

const root = fileURLToPath(new URL('.', import.meta.url));
const runtimeRoot = fileURLToPath(new URL('../../..', import.meta.url));
const compiler = process.env.COMPACT_SECP256R1_TEST_COMPILER;
const output = process.env.COMPACT_SECP256R1_TEST_OUTPUT;

function compile(fixture, options, targets, feature = '--feature-zkir-v3', source) {
  if (source === undefined) {
    const directory = join(root, fixture);
    const sources = readdirSync(directory).filter((name) => name.endsWith('.compact'));
    expect(sources).toHaveLength(1);
    source = join(directory, sources[0]);
  }
  const caseDirectory = join(output, fixture, feature ? targets.join('-') : 'legacy');
  const build = join(caseDirectory, '.build');
  mkdirSync(caseDirectory, { recursive: true });
  const result = spawnSync(compiler, [
    ...targets.flatMap((target) => ['--target', target]),
    ...(targets.includes('rust') ? ['--rust-runtime-root', runtimeRoot] : []),
    ...(feature ? [feature] : []), '--skip-zk', ...(options.compilerArgs ?? []),
    source, build,
  ], { encoding: 'utf8' });
  if (result.error) throw result.error;
  expect(result.signal).toBeNull();
  writeFileSync(join(caseDirectory, 'stdout.log'), result.stdout);
  writeFileSync(join(caseDirectory, 'stderr.log'), result.stderr);
  writeFileSync(join(caseDirectory, 'exit.txt'), `${result.status}\n`);
  return { build, result };
}

const refusals = import.meta.glob('./{base,scalar}/casts/*/compile.fail.test.ts', { eager: true, import: 'default' });

test('retains all six original compile refusal declarations', () => {
  expect(Object.keys(refusals)).toHaveLength(6);
});

for (const [path, definition] of Object.entries(refusals)) {
  const fixture = dirname(path);
  for (const targets of [['ts'], ['rust'], ['ts', 'rust']]) {
    test(`${fixture} refuses compilation for ${targets.join('+')}`, () => {
      expect(definition.result).toBe('fail');
      const { build, result } = compile(fixture, definition.options, targets);
      expect(result.status).not.toBe(0);
      expect(result.stderr).toMatch(definition.options.expectedError);
      expect(existsSync(build)).toBe(false);
    });
  }
}

test('runs the intact base-field roundtrip against generated TypeScript', async () => {
  expect(roundtripCompile.result).toBe('pass');
  expect(roundtripRuntime.result).toBe('pass');
  const { build, result } = compile('base/roundtrip', roundtripCompile.options, ['ts', 'rust']);
  expect(result.status, result.stderr).toBe(0);
  const entry = pathToFileURL(join(build, 'contract/index.js')).href;
  const { Contract, pureCircuits } = await import(/* @vite-ignore */ entry);
  await roundtripRuntime.run(Contract, pureCircuits);
  const { contract, ctx } = await createTestContract(Contract);
  for (const invalid of [-1n, MAX_SECP256R1_BASE + 1n]) {
    expect(() => contract.circuits.secp256r1_base_roundtrip(ctx, invalid)).toThrow(/Secp256r1Base/);
  }
});

test('preserves shifted defaults and scalar boundaries through the generated contract', async () => {
  const { build, result } = compile('controls', {}, ['ts', 'rust']);
  expect(result.status, result.stderr).toBe(0);
  const entry = pathToFileURL(join(build, 'contract/index.js')).href;
  const { Contract, ledger } = await import(/* @vite-ignore */ entry);
  const { contract, ctx } = await createTestContract(Contract);
  for (const scalar of [0n, 1n, (1n << 192n) + 17n, MAX_SECP256R1_SCALAR]) {
    const call = contract.circuits.scalarRoundtrip(ctx, scalar);
    expect(call.result).toBe(scalar);
    expect(ledger(call.context.currentQueryContext.state).scalarCell).toBe(scalar);
  }
  expect(() => contract.circuits.scalarRoundtrip(ctx, MAX_SECP256R1_SCALAR + 1n)).toThrow(/Secp256r1Scalar/);
  const written = contract.circuits.scalarRoundtrip(ctx, MAX_SECP256R1_SCALAR);
  const reset = contract.circuits.resetFields(written.context);
  expect(reset.result).toEqual([0n, 0n]);
  const state = ledger(reset.context.currentQueryContext.state);
  expect([state.baseCell, state.scalarCell]).toEqual([0n, 0n]);
});

test('refuses the original P-256 roundtrip in the legacy proof profile', () => {
  const { build, result } = compile('base/roundtrip', roundtripCompile.options, ['ts'], '');
  expect(result.status).not.toBe(0);
  expect(result.stderr).toContain('Secp256r1Base requires ZKIR v3');
  expect(existsSync(build)).toBe(false);
});

registerByteCastChecks(compile);
registerArithmeticChecks(compile);
registerUnaryChecks(compile);
registerPointChecks(compile);
registerAccessorChecks(compile);
registerGroupChecks(compile);
registerTransportChecks(compile);
registerEcdsaChecks(compile);
registerSignatureSuites(compile);
