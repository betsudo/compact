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

// Execute upstream runtime assertions and the local Field-to-Uint index case.
import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';

const [contractPath, runtimePath, name] = process.argv.slice(2);
const expected = {
  computed_index: [20n, 30n],
  default: [0n, 0n],
  empty: [],
  field_to_uint: [20n],
  from_parameter: [42n],
  mixed_types: [4n, true],
  nested: [1200n, 1200n],
  radix_bounds: [30n, 40n],
  semantics: [20n, 30n],
};
if (!Object.hasOwn(expected, name)) throw new Error(`unknown vector slice fixture: ${name}`);

const { Contract } = await import(pathToFileURL(contractPath));
const { createCircuitContext, createConstructorContext, dummyContractAddress } =
  await import(pathToFileURL(`${runtimePath}/dist/index.js`));
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
const args = name === 'from_parameter' ? [[false, true, 42n]] : [];
const actual = Array.from(
  (await contract.circuits[`vector_slice_${name}`](context, ...args)).result,
);
assert.deepStrictEqual(actual, expected[name]);
console.log(`${name}: ${name === 'field_to_uint' ? 'local' : 'original'} runtime result passed`);
