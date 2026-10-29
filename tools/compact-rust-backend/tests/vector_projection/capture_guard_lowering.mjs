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

// Real stateful/constructor wrapper execution; no proof of a failing path.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {realpathSync, writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';

const [stateful, known, unknown, runtime, output] = process.argv.slice(2);
const entry = realpathSync(resolve(runtime, 'dist/index.js'));
for (const wrapper of [stateful, known, unknown]) {
  assert.equal(realpathSync(createRequire(wrapper).resolve('@midnight-ntwrk/compact-runtime')), entry);
}
const r = await import(pathToFileURL(entry));
const c = await import(pathToFileURL(stateful));
const key = {bytes: new Uint8Array(32)}, rows = {};
for (const name of ['guarded', 'guarded_unknown']) {
  for (const active of [false, true]) {
    let count = 0;
    const contract = new c.Contract({values(ctx) { count++; return [ctx.privateState, [17n, 29n]]; }});
    const initial = contract.initialState({initialPrivateState: 7, initialZswapLocalState: r.emptyZswapLocalState(key)});
    const context = r.createCircuitContext(r.dummyContractAddress(), key, initial.currentContractState.data, 7,
      undefined, undefined, 0);
    let value, error;
    try { value = String(contract.circuits[name](context, active).result); } catch (e) { error = String(e); }
    if (active) assert.equal(error, 'CompactError: failed assert: result of subtraction would be negative');
    else { assert.equal(value, '0'); assert.equal(error, undefined); }
    assert.equal(count, active && name === 'guarded' ? 1 : 0);
    rows[name + '_' + active] = {value, error, witnesses: count};
  }
}
for (const [name, wrapper] of [['constructor_known', known], ['constructor_unknown', unknown]]) {
  const module = await import(pathToFileURL(wrapper));
  const contract = new module.Contract({});
  let error;
  try {
    contract.initialState({initialPrivateState: 7, initialZswapLocalState: r.emptyZswapLocalState(key)}, [17n, 29n]);
  } catch (e) { error = String(e); }
  assert.equal(error, 'CompactError: failed assert: result of subtraction would be negative');
  rows[name] = {error};
}
writeFileSync(output, JSON.stringify(rows, null, 2) + '\n');
