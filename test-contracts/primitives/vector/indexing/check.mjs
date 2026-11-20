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

// Run the unchanged upstream result assertions through the generated API.
import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';

const [contractPath, runtimePath, name] = process.argv.slice(2);
if (!['basic', 'nested'].includes(name)) throw new Error(`unknown indexing case: ${name}`);
const { Contract } = await import(pathToFileURL(contractPath));
const { createCircuitContext, createConstructorContext, dummyContractAddress } =
  await import(pathToFileURL(`${runtimePath}/dist/index.js`));
const contract = new Contract({});
const initial = await contract.initialState(createConstructorContext(undefined, '0'.repeat(64)));
const context = createCircuitContext(
  dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState,
);
const result = (await contract.circuits[`vector_indexing_${name}`](context)).result;
if (name === 'basic') assert.deepStrictEqual(Array.from(result), [10n, 30n, 40n]);
else assert.strictEqual(result, 60n);
console.log(`${name}: original indexing assertion passed`);
