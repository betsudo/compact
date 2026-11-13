// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';

const [entry, circuit] = process.argv.slice(2);
const bytes = (rows) => rows.map((row) => Uint8Array.from(row));
const expected = new Map([
  ['bytes_map_reordering', bytes([[3, 2], [5, 4], [7, 6]])],
  ['bytes_map_slice_composition', bytes([[20], [40], [60]])],
  ['bytes_map_to_bytes', bytes([[0], [1], [2], [3], [4], [5], [6], [7]])],
  ['bytes_map_transformation', bytes([[15], [25], [35], [45], [55]])],
]);
assert.ok(entry && expected.has(circuit));
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry).href);
const { Contract } = await import(pathToFileURL(entry).href);
const contract = new Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const context = runtime.createCircuitContext(
  runtime.dummyContractAddress(),
  initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState,
  initial.currentPrivateState,
);
assert.deepEqual(contract.circuits[circuit](context).result, expected.get(circuit));
console.log(`${circuit} TypeScript passed`);
