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
const results = new Map([
  ['casts/empty', []],
  ['casts/round_trip', [5n, 6n, 7n, 8n]],
  ['casts/to_field', 42n],
  ['casts/to_tuple', [10n, 20n, 30n, 40n]],
  ['casts/to_uint', 67305985n],
  ['literal/escape_sequences', [13, 10]],
  ['literal/non_ascii', [10, 195, 132]],
  ['literal/string', [97, 98, 99, 100, 101, 102, 103, 104]],
]);
assert.ok(results.has(name), `unexpected Bytes case: ${name}`);
const circuit = `bytes_${name.replace('/', '_')}`;
const value = pureCircuits[circuit]();
const expected = results.get(name);
if (Array.isArray(expected)) {
  assert.deepEqual(Array.from(value), expected);
} else {
  assert.equal(value, expected);
}
console.log(`${name}: original assertion passed`);
