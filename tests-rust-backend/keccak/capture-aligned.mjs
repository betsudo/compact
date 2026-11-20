// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Capture the generated fixture's Ledger 9 inputs with an independent Keccak
// implementation, then cross-check Runtime 0.17.0's aligned TypeScript API.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const runtimeRoot = resolve(process.argv[2] ?? '');
if (!process.argv[2]) throw new Error('expected a built Runtime 0.17.0 package root');
const manifest = JSON.parse(readFileSync(join(runtimeRoot, 'package.json'), 'utf8'));
assert.equal(manifest.version, '0.17.0');
const runtime = await import(pathToFileURL(join(runtimeRoot, 'dist/index.js')).href);
const { keccak_256 } = await import(
  pathToFileURL(join(runtimeRoot, 'node_modules/@noble/hashes/sha3.js')).href
);

const field = (value) => Uint8Array.from([Number(value), ...Array(31).fill(0)]);
const cases = {
  field: [runtime.CompactTypeField, 42n, field(42)],
  bytes: [new runtime.CompactTypeBytes(4), Uint8Array.of(1, 2, 3, 4), Uint8Array.of(1, 2, 3, 4)],
  pair: [
    new runtime.CompactTypeVector(2, runtime.CompactTypeField),
    [3n, 5n],
    Uint8Array.from([...field(3), ...field(5)]),
  ],
  witness: [runtime.CompactTypeField, 9n, field(9)],
};
const output = { format: 'compact-keccak-aligned-oracle/v1', runtimeVersion: manifest.version };
for (const [name, [type, value, preimage]] of Object.entries(cases)) {
  const digest = Buffer.from(keccak_256(preimage)).toString('hex');
  assert.equal(Buffer.from(runtime.keccak256Aligned(type, value)).toString('hex'), digest, name);
  output[name] = { preimage: Buffer.from(preimage).toString('hex'), digest };
}
process.stdout.write(JSON.stringify(output, null, 2) + '\n');
