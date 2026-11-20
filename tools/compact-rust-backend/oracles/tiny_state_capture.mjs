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

// Capture Tiny's full ContractState envelope at each step of its public API.
import { createRequire } from 'node:module';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

if (process.argv.length !== 3) throw new Error('usage: node tiny_state_capture.mjs <compiled-contract-dir>');
const contractIndex = resolve(process.argv[2], 'index.js');
const requireFromContract = createRequire(contractIndex);
const runtime = await import(pathToFileURL(requireFromContract.resolve('@midnight-ntwrk/compact-runtime')));
const { Contract, ledger } = await import(pathToFileURL(contractIndex));

const contract = new Contract({
  private$secret_key: ({ privateState }) => [privateState, new Uint8Array(32).fill(7)],
});
const coinPublicKey = { bytes: new Uint8Array(32) };
const initial = contract.initialState({
  initialPrivateState: null,
  initialZswapLocalState: runtime.emptyZswapLocalState(coinPublicKey),
}, 42n);
const state = initial.currentContractState;
function snapshot() {
  return {
    stateHex: Buffer.from(state.serialize()).toString('hex'),
    ledger: { value: ledger(state.data).value.toString() },
  };
}
const afterInit = snapshot();
let context = runtime.createCircuitContext(
  runtime.dummyContractAddress(), coinPublicKey, state.data, initial.currentPrivateState,
);
context = contract.circuits.clear(context).context;
state.data = new runtime.ChargedState(context.currentQueryContext.state.state);
const afterClear = snapshot();
context = contract.circuits.set(context, 99n).context;
state.data = new runtime.ChargedState(context.currentQueryContext.state.state);
const afterSet99 = snapshot();
const { result } = contract.circuits.get(context);
const getResult = { isSome: result.is_some, value: result.value.toString() };

process.stdout.write(JSON.stringify({
  ...afterInit, afterInit, afterClear, afterSet99, getResult,
}, null, 2) + '\n');
