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

import { pathToFileURL } from 'node:url';

const [contractPath, runtimePath] = process.argv.slice(2);
if (!contractPath || !runtimePath) {
  throw new Error('expected generated contract and Compact runtime entry paths');
}
const runtime = await import(pathToFileURL(runtimePath).href);
const { Contract, ledger } = await import(pathToFileURL(contractPath).href);

const contract = new Contract({});
const coinPublicKey = { bytes: new Uint8Array(32) };
const initial = await contract.initialState({
  initialPrivateState: null,
  initialZswapLocalState: runtime.emptyZswapLocalState(coinPublicKey),
});
const state = initial.currentContractState;
const serialize = () => Buffer.from(state.serialize()).toString('hex');
const before = serialize();
const context = runtime.createCircuitContext({
  circuitId: 'increment',
  contractAddress: runtime.dummyContractAddress(),
  coinPublicKeyOrZswapState: coinPublicKey,
  contractState: state.data,
  privateState: null,
});
const outcome = await contract.circuits.increment(context);
state.data = new runtime.ChargedState(outcome.context.callContext.currentQueryContext.state.state);
const value = ledger(state.data).round;
if (value !== 1n || outcome.result.length !== 0) throw new Error('Counter behavior differs');
console.log(JSON.stringify({ before, after: serialize(), value: value.toString(), result: outcome.result }, null, 2));
