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
const { Contract, ledger, pureCircuits } = await import(pathToFileURL(generated));
const contract = new Contract({});
const key = { bytes: new Uint8Array(32) };
const initial = contract.initialState({
  initialPrivateState: null,
  initialZswapLocalState: runtime.emptyZswapLocalState(key),
});
const state = initial.currentContractState;
const snapshot = () => Buffer.from(state.serialize()).toString('hex');
const afterInit = snapshot();
function captureRefusal(run) {
  try {
    run();
  } catch (error) {
    if (!(error instanceof runtime.CompactError)) throw error;
    return { error: error.message, compactError: true };
  }
  throw new Error('expected Compact assertion refusal');
}
const pureTrue = { ok: pureCircuits.require_true(true) };
const pureFalse = captureRefusal(() => pureCircuits.require_true(false));
const ctx = runtime.createCircuitContext(runtime.dummyContractAddress(), key, state.data, initial.currentPrivateState);
const { context } = contract.circuits.trigger_ok(ctx);
state.data = new runtime.ChargedState(context.currentQueryContext.state.state);
const afterTriggerOk = snapshot();
const flagAfterTriggerOk = ledger(context.currentQueryContext.state).flag;
const triggerFail = captureRefusal(() => contract.circuits.trigger_fail(context));
state.data = new runtime.ChargedState(context.currentQueryContext.state.state);
process.stdout.write(JSON.stringify({
  afterInit, pureTrue, pureFalse, afterTriggerOk, flagAfterTriggerOk, triggerFail,
  afterTriggerFail: snapshot(),
}, null, 2) + '\n');
