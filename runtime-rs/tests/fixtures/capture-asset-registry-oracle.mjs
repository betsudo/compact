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

import { createRequire } from 'node:module';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [contractPath] = process.argv.slice(2);
if (!contractPath) throw new Error('expected generated contract/index.js');
const generated = resolve(contractPath);
const runtime = await import(pathToFileURL(createRequire(generated).resolve('@midnight-ntwrk/compact-runtime')));
const { Contract, ledger } = await import(pathToFileURL(generated));
const point = value => runtime.hashToCurve(runtime.CompactTypeField, value);
const contract = new Contract({
  localOperatorKey: ({ privateState }) => [privateState, point(1n)],
  localAuditorKey: ({ privateState }) => [privateState, point(2n)],
  currentTimestamp: ({ privateState }) => [privateState, 1_700_000_000n],
});
const key = { bytes: new Uint8Array(32) };
const initial = contract.initialState({
  initialPrivateState: null,
  initialZswapLocalState: runtime.emptyZswapLocalState(key),
});
const state = initial.currentContractState;
let context = runtime.createCircuitContext(runtime.dummyContractAddress(), key, state.data, initial.currentPrivateState);
function snapshot(name) {
  state.data = new runtime.ChargedState(context.currentQueryContext.state.state);
  const view = ledger(context.currentQueryContext.state);
  return {
    name,
    stateHex: Buffer.from(state.serialize()).toString('hex'),
    recordCount: String(view.recordCount),
    writeCount: String(view.writeCount),
    recordsSize: String(view.records.size()),
    watchSize: String(view.watchList.size()),
    retiredSize: String(view.retiredKeys.size()),
    tagsSize: String(view.tags.size()),
  };
}
const rows = [snapshot('initial')];
function invoke(name, ...args) {
  ({ context } = contract.circuits[name](context, ...args));
  rows.push(snapshot(name));
}
const id = 'asset-1';
const record = {
  code: new Uint8Array(32),
  note: '',
  provenance: { facility: new Uint8Array(32), registeredAt: 0n },
  kind: 1, // Instrument
  quantity: 0n,
};
invoke('tag', 7n);
invoke('setRecord', id, record, 1); // Insert
invoke('setWatch', id, 1); // Add
invoke('setWatch', id, 2); // Drop
invoke('removeRecord', id);
process.stdout.write(JSON.stringify(rows, null, 2) + '\n');
