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

const args = process.argv.slice(2);
if (args.length !== 3) {
  throw new Error('Usage: node check.mjs <compiler> <built-ts-runtime-dir> <new-output-dir>');
}
const [compiler, runtime, output] = args.slice(0, 3).map((argument) => resolve(argument));
const root = fileURLToPath(new URL('.', import.meta.url));
const runtimeManifest = join(runtime, 'package.json');
const runtimePackage = JSON.parse(readFileSync(runtimeManifest, 'utf8'));
const runtimeName = '@midnight-ntwrk/compact-runtime';
if (runtimePackage.name !== runtimeName || typeof runtimePackage.main !== 'string') {
  throw new Error('Expected the built Compact TypeScript runtime package');
}
const requireFromRuntime = createRequire(runtimeManifest);
const vitestManifest = requireFromRuntime.resolve('vitest/package.json');
const vitestRoot = dirname(vitestManifest);
const vitestPackage = JSON.parse(readFileSync(vitestManifest, 'utf8'));

mkdirSync(output);
const namespace = join(output, 'node_modules', '@midnight-ntwrk');
mkdirSync(namespace, { recursive: true });
symlinkSync(runtime, join(namespace, 'compact-runtime'), 'dir');
const config = join(output, 'vitest.config.mjs');
writeFileSync(config, `export default ${JSON.stringify({
  root,
  cacheDir: join(output, 'vite-cache'),
  resolve: {
    alias: {
      '@test/compact-test': join(root, '../secp256k1/test-bridge.mjs'),
      [runtimeName]: join(runtime, runtimePackage.main),
      vitest: join(vitestRoot, vitestPackage.exports['.'].import.default),
    },
  },
  test: { include: ['check.test.mjs'], testTimeout: 60_000 },
})};\n`);

const result = spawnSync(process.execPath, [join(vitestRoot, vitestPackage.bin.vitest), 'run', '--config', config], {
  env: { ...process.env, COMPACT_CURVE25519_TEST_COMPILER: compiler, COMPACT_CURVE25519_TEST_OUTPUT: output },
  stdio: 'inherit',
});
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
