// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';

const [entry, family] = process.argv.slice(2);
const {pureCircuits: pure} = await import(pathToFileURL(entry).href);
if (family === 'same_range_arithmetic') {
  const rows = [
    ['mul_bit', 1n, 1n, 1n],
    ['mul_bit', 1n, 0n, 0n],
    ['mul_bit', 0n, 0n, 0n],
    ['mul_bit_range', 1n, 1n, 1n],
    ['mul_bit_range', 0n, 1n, 0n],
    ['add_zero', 0n, 0n, 0n],
    ['mul_zero', 0n, 0n, 0n],
    ['sub_bit', 1n, 1n, 0n],
    ['sub_bit', 1n, 0n, 1n],
  ];
  for (const [name, x, y, expected] of rows) assert.equal(pure[name](x, y), expected);
  assert.throws(() => pure.sub_bit(0n, 1n), /result of subtraction would be negative/);
  console.log('nine original same-range assertions and subtraction refusal passed');
} else if (family === 'addition_overflow') {
  assert.equal(pure.add_byte(200n, 55n), 255n);
  assert.throws(() => pure.add_byte(255n, 1n), /256 is greater than 255/);
  console.log('original byte boundary and overflow refusal passed');
} else {
  throw new Error(`unknown nominal Uint family: ${family}`);
}
