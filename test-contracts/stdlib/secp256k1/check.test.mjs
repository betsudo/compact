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
import { expect, test } from 'vitest';

import roundtripCompile from './base/roundtrip/compile.pass.test.ts';
import roundtripRuntime from './base/roundtrip/runtime.pass.test.ts';
import { registerArithmeticTests } from './arithmetic/checks.mjs';
import { registerPositiveCastTests } from './positive-casts/checks.mjs';
import { registerUnaryTests } from './unary/checks.mjs';

const root = fileURLToPath(new URL('.', import.meta.url));
const compiler = process.env.COMPACT_SECP256K1_TEST_COMPILER;
const output = process.env.COMPACT_SECP256K1_TEST_OUTPUT;

function compile(fixture, definition, targets, file) {
  const sourceDirectory = join(root, fixture);
  const sources = readdirSync(sourceDirectory).filter((name) => file ? name === file : name.endsWith('.compact'));
  expect(sources).toHaveLength(1);
  const caseDirectory = join(output, fixture, file ? file.replace(/\.compact$/, '') : '', targets.join('-'));
  const build = join(caseDirectory, '.build');
  mkdirSync(caseDirectory, { recursive: true });
  const targetArgs = targets.flatMap((target) => ['--target', target]);
  const result = spawnSync(
    compiler,
    [
      ...targetArgs,
      '--feature-zkir-v3',
      '--skip-zk',
      ...(definition.options.compilerArgs ?? []),
      join(sourceDirectory, sources[0]),
      build,
    ],
    { encoding: 'utf8' },
  );
  if (result.error) {
    throw result.error;
  }
  if (result.signal) {
    throw new Error(`Compiler stopped by ${result.signal}`);
  }
  writeFileSync(join(caseDirectory, 'stdout.log'), result.stdout);
  writeFileSync(join(caseDirectory, 'stderr.log'), result.stderr);
  writeFileSync(join(caseDirectory, 'exit.txt'), `${result.status}\n`);
  return { build, result };
}

const refusals = import.meta.glob('./{base,scalar}/casts/*/compile.fail.test.ts', { eager: true, import: 'default' });

test('retains all six upstream compile refusal declarations', () => {
  expect(Object.keys(refusals)).toHaveLength(6);
});

for (const [path, definition] of Object.entries(refusals)) {
  const fixture = dirname(path);
  for (const targets of [['ts'], ['rust'], ['ts', 'rust']]) {
    test(`${fixture} refuses compilation for ${targets.join('+')}`, () => {
      expect(definition.result).toBe('fail');
      expect(definition.options.expectedError).toBeInstanceOf(RegExp);
      const { build, result } = compile(fixture, definition, targets);
      expect(result.status).not.toBe(0);
      expect(result.stderr).toMatch(definition.options.expectedError);
      expect(existsSync(build)).toBe(false);
    });
  }
}

test('runs the original base field roundtrip assertion on generated TypeScript', async () => {
  expect(roundtripCompile.result).toBe('pass');
  expect(roundtripRuntime.result).toBe('pass');
  const { build, result } = compile('base/roundtrip', roundtripCompile, ['ts']);
  expect(result.status, result.stderr).toBe(0);
  const entry = pathToFileURL(join(build, 'contract', 'index.js')).href;
  const { Contract, pureCircuits } = await import(/* @vite-ignore */ entry);
  await roundtripRuntime.run(Contract, pureCircuits);
});

registerPositiveCastTests(compile);
registerArithmeticTests(compile);
registerUnaryTests(compile);
