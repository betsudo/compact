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
import { mkdirSync, readFileSync, symlinkSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const [compilerArg, runtimeArg, outputArg, mode, ...extra] = process.argv.slice(2);
if (!outputArg || extra.length || (mode && !['--upstream-kat', '--allow-pragma-probe'].includes(mode))) {
  throw new Error('Usage: node check.mjs <compiler> <built-ts-runtime-dir> <new-output-dir> [--upstream-kat | --allow-pragma-probe]');
}
const [compiler, runtime, output] = [compilerArg, runtimeArg, outputArg].map((argument) => resolve(argument));
const root = fileURLToPath(new URL('.', import.meta.url));
const readJson = (path) => JSON.parse(readFileSync(path, 'utf8'));

const runtimeManifest = join(runtime, 'package.json');
const runtimePackage = readJson(runtimeManifest);
const runtimeName = '@midnight-ntwrk/compact-runtime';
if (runtimePackage.name !== runtimeName || typeof runtimePackage.main !== 'string') {
  throw new Error('Expected the built Compact TypeScript runtime package');
}
const requireFromRuntime = createRequire(runtimeManifest);
const vitestManifest = requireFromRuntime.resolve('vitest/package.json');
const vitestRoot = dirname(vitestManifest);
const vitestPackage = readJson(vitestManifest);

mkdirSync(output);
const namespace = join(output, 'node_modules', '@midnight-ntwrk');
mkdirSync(namespace, { recursive: true });
symlinkSync(runtime, join(namespace, 'compact-runtime'), 'dir');

const config = join(output, 'vitest.config.mjs');
writeFileSync(
  config,
  `export default ${JSON.stringify({
    root,
    cacheDir: join(output, 'vite-cache'),
    resolve: {
      alias: {
        '@test/compact-test': join(root, 'test-bridge.mjs'),
        '@test/crypto': join(root, '../../support/crypto/index.ts'),
        '@noble/curves': dirname(requireFromRuntime.resolve('@noble/curves/secp256k1.js')),
        '@noble/hashes': dirname(requireFromRuntime.resolve('@noble/hashes/utils.js')),
        [runtimeName]: join(runtime, runtimePackage.main),
        vitest: join(vitestRoot, vitestPackage.exports['.'].import.default),
      },
    },
    test: { include: ['check.test.mjs'], testTimeout: 60_000 },
  })};\n`,
);

const filter = mode ? ['--testNamePattern', 'runs the original .* known-answer suite'] : [];
const result = spawnSync(process.execPath, [join(vitestRoot, vitestPackage.bin.vitest), 'run', '--config', config, ...filter], {
  env: {
    ...process.env,
    COMPACT_SECP256K1_TEST_COMPILER: compiler,
    COMPACT_SECP256K1_TEST_OUTPUT: output,
    COMPACT_SECP256K1_KAT_MODE: mode ?? '',
  },
  stdio: 'inherit',
});
if (result.error) {
  throw result.error;
}
process.exitCode = result.status ?? 1;
