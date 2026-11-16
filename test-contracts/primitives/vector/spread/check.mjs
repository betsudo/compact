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

// The original runtime.pass.test.ts assertions run against this branch's
// synchronous circuit context without changing any pinned upstream fixture.
import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';

const [contractPath, runtimePath, name] = process.argv.slice(2);
const expected = {
  concatenation: [1n, 2n, 3n, 4n],
  default: Array.from({ length: 10 }, () => 0n),
  empty: [1n, 2n, 3n],
  mixed_types: [true, 2n, 3n],
  nested: [1n, 2n, 3n],
  with_literals: [1n, 2n, 3n, 4n, 5n],
  with_slice: [10n, 20n, 40n, 50n],
};
if (!Object.hasOwn(expected, name)) {
  throw new Error(`unknown original vector spread fixture: ${name}`);
}

const { Contract } = await import(pathToFileURL(contractPath));
const {
  createCircuitContext,
  createConstructorContext,
  dummyContractAddress,
} = await import(pathToFileURL(`${runtimePath}/dist/index.js`));
const contract = new Contract({});
const initial = await contract.initialState(
  createConstructorContext(undefined, '0'.repeat(64)),
);
const context = createCircuitContext(
  dummyContractAddress(),
  initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState,
  initial.currentPrivateState,
);
const actual = Array.from(
  (await contract.circuits[`vector_spread_${name}`](context)).result,
);
assert.deepStrictEqual(actual, expected[name]);
console.log(`${name}: original runtime result passed`);
