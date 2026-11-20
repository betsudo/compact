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

const [contractPath, runtimePath, kind] = process.argv.slice(2);
if (!contractPath || !runtimePath || !['initial', 'set', 'map', 'list'].includes(kind)) {
  throw new Error('expected generated contract, Compact runtime entry and initial/set/map/list');
}
const runtime = await import(pathToFileURL(runtimePath).href);
const { Contract } = await import(pathToFileURL(contractPath).href);
const contract = new Contract({});
const coinPublicKey = { bytes: new Uint8Array(32) };
const initial = await contract.initialState({
  initialPrivateState: null,
  initialZswapLocalState: runtime.emptyZswapLocalState(coinPublicKey),
});
const state = initial.currentContractState;
const stateHex = () => Buffer.from(state.serialize()).toString('hex');
const output = { afterInit: { stateHex: stateHex() } };

if (kind !== 'initial') {
  let context;
  const operations = {
    set: [
      ['add', () => contract.circuits.add(context, true)],
      ['remove', () => contract.circuits.remove(context, true)],
      ['add_field', () => contract.circuits.add_field(context, 42n)],
      ['reset_fields', () => contract.circuits.reset_fields(context)],
    ],
    map: [
      ['put', () => contract.circuits.put(context, true, 42n)],
      ['remove_key', () => contract.circuits.remove_key(context, true)],
      ['put_default', () => contract.circuits.put_default(context, true)],
      ['reset_table', () => contract.circuits.reset_table(context)],
    ],
    list: [
      ['prepend', () => contract.circuits.prepend(context, 42n)],
      ['prepend', () => contract.circuits.prepend(context, 7n)],
      ['drop_first', () => contract.circuits.drop_first(context)],
      ['clear_items', () => contract.circuits.clear_items(context)],
    ],
  }[kind];
  output.steps = [];
  for (const [operation, call] of operations) {
    context = runtime.createCircuitContext({
      circuitId: operation,
      contractAddress: runtime.dummyContractAddress(),
      coinPublicKeyOrZswapState: coinPublicKey,
      contractState: state.data,
      privateState: null,
    });
    const result = await call();
    state.data = new runtime.ChargedState(result.context.callContext.currentQueryContext.state.state);
    output.steps.push({ operation, stateHex: stateHex() });
  }
}

process.stdout.write(JSON.stringify(output, null, 2) + '\n');
