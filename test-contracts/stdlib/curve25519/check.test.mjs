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
import { MAX_CURVE25519_BASE, MAX_CURVE25519_SCALAR } from '@midnight-ntwrk/compact-runtime';
import { createTestContract } from '@test/compact-test';
import { expect, test } from 'vitest';

import baseCompile from './base/roundtrip/compile.pass.test.ts';
import baseRuntime from './base/roundtrip/runtime.pass.test.ts';
import scalarCompile from './scalar/roundtrip/compile.pass.test.ts';
import scalarRuntime from './scalar/roundtrip/runtime.pass.test.ts';

const root = fileURLToPath(new URL('.', import.meta.url));
const runtimeRoot = fileURLToPath(new URL('../../..', import.meta.url));
const compiler = process.env.COMPACT_CURVE25519_TEST_COMPILER;
const output = process.env.COMPACT_CURVE25519_TEST_OUTPUT;

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

for (const [name, definition, runtimeTest, maximum] of [
  ['base', baseCompile, baseRuntime, MAX_CURVE25519_BASE],
  ['scalar', scalarCompile, scalarRuntime, MAX_CURVE25519_SCALAR],
]) {
  test(`runs the intact ${name} roundtrip for generated TypeScript`, async () => {
    expect(definition.result).toBe('pass');
    expect(runtimeTest.result).toBe('pass');
    const { build, result } = compile(`${name}/roundtrip`, definition.options, ['ts', 'rust']);
    expect(result.status, result.stderr).toBe(0);
    const { Contract, pureCircuits } = await import(/* @vite-ignore */ pathToFileURL(join(build, 'contract/index.js')).href);
    await runtimeTest.run(Contract, pureCircuits);
    const { contract, ctx } = await createTestContract(Contract);
    for (const invalid of [-1n, maximum + 1n]) {
      expect(() => contract.circuits[`curve25519_${name}_roundtrip`](ctx, invalid)).toThrow(/Curve25519/);
    }
  });

  test(`refuses the ${name} roundtrip in the legacy proof profile`, () => {
    const { build, result } = compile(`${name}/roundtrip`, definition.options, ['ts'], '');
    expect(result.status).not.toBe(0);
    expect(result.stderr).toMatch(/Curve25519(?:Base|Scalar) requires ZKIR v3/);
    expect(existsSync(build)).toBe(false);
  });
}

test('emits field arithmetic, checked 32-byte casts and Scalar hashes for both targets', async () => {
  const { build, result } = compile('operations', {}, ['ts', 'rust']);
  expect(result.status, result.stderr).toBe(0);
  const { Contract } = await import(/* @vite-ignore */ pathToFileURL(join(build, 'contract/index.js')).href);
  const { contract, ctx } = await createTestContract(Contract);
  expect(contract.circuits.arithmetic(ctx, 3n, 5n).result).toEqual([5n, 7n]);
  expect(() => contract.circuits.arithmetic(ctx, 0n, 1n)).toThrow(/Curve25519 base field has no inverse/);
  expect(() => contract.circuits.arithmetic(ctx, 1n, 0n)).toThrow(/Curve25519 scalar field has no inverse/);
  const bytes = new Uint8Array(32);
  bytes[0] = 17;
  expect(contract.circuits.casts(ctx, bytes).result).toEqual([bytes, bytes]);
  expect(contract.circuits.hashScalar(ctx, 1n).result).toHaveLength(32);
});
