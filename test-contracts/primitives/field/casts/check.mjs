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
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';

const [entry, name] = process.argv.slice(2);
assert.ok(entry && name && process.argv.length === 4);
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry).href);
const { pureCircuits } = await import(pathToFileURL(entry).href);
const circuit = `field_casts_${name}`;
const invoke = (...args) => pureCircuits[circuit](...args);

switch (name) {
  case 'to_boolean':
    assert.equal(invoke(), true);
    break;
  case 'to_bytes':
    assert.deepEqual(Array.from(invoke()), [1, 0, 0, 0]);
    break;
  case 'to_enum':
    for (let i = 0; i < 3; i++) assert.equal(invoke(BigInt(i)), i);
    break;
  case 'to_enum_out_of_range':
    assert.throws(() => invoke(3n),
      /cast from Field or Uint value to enum Color failed: 3 is greater than maximum enum value 2/);
    break;
  case 'to_uint':
    for (const value of [0n, 1n, 2n ** 64n - 1n]) assert.equal(invoke(value), value);
    break;
  case 'to_uint_max_field':
    assert.throws(() => invoke(runtime.MAX_FIELD), new RegExp(
      `cast from Field or Uint value to smaller Uint value failed: ${runtime.MAX_FIELD} is greater than ${2n ** 248n - 1n}`));
    break;
  case 'to_uint_out_of_range':
    assert.throws(() => invoke(2n ** 64n),
      /cast from Field or Uint value to smaller Uint value failed: 18446744073709551616 is greater than 18446744073709551615/);
    break;
  case 'bytes4_boundary': {
    for (const [value, expected] of [
      [0n, [0, 0, 0, 0]],
      [0x01020304n, [4, 3, 2, 1]],
      [0xffffffffn, [255, 255, 255, 255]],
    ]) assert.deepEqual(Array.from(invoke(value)), expected);
    for (const value of [0x100000000n, (2n ** 40n) + 0x01020304n]) {
      assert.throws(() => invoke(value), new RegExp(
        `Field or Uint value ${value} does not fit into 4 bytes`));
    }
    break;
  }
  default:
    throw Error(`unexpected Field cast case: ${name}`);
}
console.log(`${name}: original or boundary assertions passed`);
