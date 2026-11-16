// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';

const [entry, name] = process.argv.slice(2);
assert.ok(entry && name && process.argv.length === 4);
const { pureCircuits } = await import(pathToFileURL(entry).href);
const expected = new Map([
  ['casts/to_bytes', [2, 1, 0, 0]],
  ['casts/to_uint_different_width', 1n],
  ['ranges/addition', 8n],
  ['ranges/multiplication', 15n],
]);
assert.ok(expected.has(name), `unexpected Uint case: ${name}`);
const circuit = `uint_${name.replace('/', '_')}`;
const value = pureCircuits[circuit]();
if (Array.isArray(expected.get(name))) {
  assert.deepEqual(Array.from(value), expected.get(name));
} else {
  assert.equal(value, expected.get(name));
}
console.log(`${name}: original assertion passed`);
