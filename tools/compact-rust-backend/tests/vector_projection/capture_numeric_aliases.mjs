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

// Execute the genuine generated TS wrapper against an explicitly selected runtime.
import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';
import {writeFileSync} from 'node:fs';

const {pureCircuits} = await import(pathToFileURL(process.argv[2]));
const result = {zero: [], checked: {}, ordered: {}, indices: {}};
for (const value of [0n, 1n, 5n, 255n]) {
  const zero = pureCircuits.zero(value);
  assert.equal(zero, 0n);
  result.zero.push(String(zero));
  let actual, failure;
  try { actual = pureCircuits.checked(value); } catch (error) { failure = String(error); }
  if (value === 0n) {
    assert.equal(failure, undefined);
    assert.equal(actual, 0n);
    result.checked[String(value)] = {value: String(actual)};
  } else {
    assert.equal(typeof failure, 'string');
    result.checked[String(value)] = {error: failure};
  }
}
for (const value of [0n, 1n]) {
  let observed;
  try { pureCircuits.ordered(value); } catch (error) { observed = String(error); }
  assert.equal(typeof observed, 'string');
  assert.equal(observed.includes('second failure'), value === 0n);
  result.ordered[String(value)] = {error: observed};
}
for (const [name, invoke, expected] of [
  ['index_guard_left', () => pureCircuits.index_guard_left([17n, 29n]),
    'CompactError: failed assert: result of subtraction would be negative'],
  ['index_guard_right', () => pureCircuits.index_guard_right([17n, 29n]),
    'CompactError: failed assert: result of subtraction would be negative'],
  ['index_guard_wide', () => pureCircuits.index_guard_wide([17n, 29n]),
    'CompactError: failed assert: result of subtraction would be negative'],
  ['index_order_true', () => pureCircuits.index_order([17n, 29n], true),
    'CompactError: failed assert: Vector operand before index'],
  ['index_order_false', () => pureCircuits.index_order([17n, 29n], false),
    'CompactError: failed assert: result of subtraction would be negative'],
]) {
  let failure;
  try { invoke(); } catch (error) { failure = String(error); }
  assert.equal(failure, expected);
  result.indices[name] = {error: failure};
}
const selected = pureCircuits.index_guard_success([17n, 29n]);
assert.equal(selected, 17n);
result.indices.index_guard_success = {value: String(selected)};
const wideSelected = pureCircuits.index_total_wide([17n, 29n]);
assert.equal(wideSelected, 17n);
result.indices.index_total_wide = {value: String(wideSelected)};
writeFileSync(process.argv[3], JSON.stringify(result, null, 2) + '\n');
