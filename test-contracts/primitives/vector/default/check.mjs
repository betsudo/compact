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

const [entry, expectedArg] = process.argv.slice(2);
assert.ok(entry && /^\d+(,\d+){3}$/.test(expectedArg));
const expected = expectedArg.split(',').map((part) => BigInt(part));
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry).href);
const { Contract, ledger } = await import(pathToFileURL(entry).href);
const contract = new Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const context = runtime.createCircuitContext(
  runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState,
);
const execution = contract.circuits.vector_default_semantics(context);
assert.deepStrictEqual(Array.from(execution.result), expected);
assert.deepStrictEqual(
  Array.from(ledger(execution.context.currentQueryContext.state).default_result), expected,
);
assert.ok(execution.proofData);
console.log('default: original Vector result and declared state passed');
