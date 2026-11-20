// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import assert from 'node:assert/strict';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [build, control, wasm, output] = process.argv.slice(2);
if (process.argv.length !== 6) {
  throw new Error('Usage: node check.mjs <inner-output> <ordered-output> <zkir-wasm-package> <new-output-dir>');
}
const require = createRequire(resolve(build, 'contract/index.js'));
const runtime = await import(pathToFileURL(require.resolve('@midnight-ntwrk/compact-runtime')));
const ocrt = await import(pathToFileURL(require.resolve('@midnightntwrk/onchain-runtime-v4', {
  paths: [require.resolve('@midnight-ntwrk/compact-runtime')],
})));
const manifest = JSON.parse(await readFile(join(wasm, 'package.json'), 'utf8'));
assert.equal(manifest.version, '3.2.0');
const { check, jsonIrToBinary } = await import(pathToFileURL(resolve(wasm, manifest.exports.node)));
const { Contract, ledger } = await import(pathToFileURL(resolve(build, 'contract/index.js')));
const contract = new Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const address = runtime.dummyContractAddress();
const start = runtime.createCircuitContext(address, initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState);
const nullifier = Uint8Array.from({length: 32}, (_, i) => i + 1);
const changed = Uint8Array.from(nullifier);
changed[31] ^= 128;
const first = contract.circuits.innerEmit(start, nullifier);
const second = contract.circuits.innerEmit(first.context, changed);
assert.equal(first.result, 1n);
assert.equal(second.result, 2n);
assert.equal(ledger(second.context.currentQueryContext.state).v, 2n);
assert.deepEqual(start.events, []);
assert.equal(first.context.events.length, 1);
assert.equal(second.context.events.length, 2);
for (const [i, bytes] of [nullifier, changed].entries()) {
  const event = second.context.events[i];
  assert.equal(event.version, 1);
  assert.equal(event.eventType, 'shielded-spend');
  assert.equal(event.address, address);
  assert.equal(event.data.tag, 'cell');
  assert.deepEqual(event.data.content.alignment, new runtime.CompactTypeBytes(32).alignment());
  assert.deepEqual(event.data.content.value, new runtime.CompactTypeBytes(32).toValue(bytes));
}
assert.throws(() => contract.circuits.innerEmit(first.context, new Uint8Array(31)), /Bytes<32>/);
assert.equal(first.context.events.length, 1);

const name = 'innerEmit';
const ir = jsonIrToBinary(await readFile(join(build, 'zkir', name + '.zkir'), 'utf8'));
const bin = await readFile(join(build, 'zkir', name + '.bzkir'));
assert.deepEqual(Buffer.from(ir), bin);
const opts = {
  lookupKey: async () => ({ ir, proverKey: new Uint8Array(), verifierKey: new Uint8Array() }),
  getParams: async () => { throw Error('unexpected parameters request'); },
};
const data = first.proofData;
const serialize = (t) => ocrt.proofDataIntoSerializedPreimage(data.input, data.output, t,
  data.privateTranscriptOutputs, name);
const eventIndex = data.publicTranscript.indexOf('log');
assert.ok(eventIndex > 0);
const pushIndex = eventIndex - 1;
const push = data.publicTranscript[pushIndex];
assert.equal(push.push.value.tag, 'array');
assert.equal(push.push.storage, false);
assert.deepEqual(data.privateTranscriptOutputs, []);
const preimage = serialize(data.publicTranscript);
await check(preimage, opts);
await mkdir(output);
await writeFile(join(output, name + '.preimage'), preimage, { flag: 'wx' });
for (const [kind, program] of [
  ['missing-event', data.publicTranscript.filter((_, i) => i !== eventIndex && i !== pushIndex)],
  ...[0, 1, 2].map((cell) => {
    const program = structuredClone(data.publicTranscript);
    program[pushIndex].push.value.content[cell].content.value = cell === 2
      ? new runtime.CompactTypeBytes(32).toValue(changed)
      : ocrt.bigIntToValue(2n);
    return [['changed-version', 'changed-tag', 'changed-payload'][cell], program];
  }),
]) {
  const bad = serialize(program);
  assert.notDeepEqual(bad, preimage);
  await assert.rejects(check(bad, opts));
  await writeFile(join(output, kind + '.preimage'), bad, { flag: 'wx' });
}
const { Contract: Ordered } = await import(pathToFileURL(resolve(control, 'contract/index.js')));
const ordered = new Ordered({});
const orderedInitial = ordered.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const orderedContext = runtime.createCircuitContext(address, orderedInitial.currentZswapLocalState.coinPublicKey,
  orderedInitial.currentContractState, orderedInitial.currentPrivateState);
const orderedIr = jsonIrToBinary(await readFile(join(control, 'zkir/ordered.zkir'), 'utf8'));
assert.deepEqual(Buffer.from(orderedIr), await readFile(join(control, 'zkir/ordered.bzkir')));
const orderedOpts = { ...opts, lookupKey: async () => ({ ir: orderedIr, proverKey: new Uint8Array(), verifierKey: new Uint8Array() }) };
const orderedData = (proof) => ocrt.proofDataIntoSerializedPreimage(proof.input, proof.output,
  proof.publicTranscript, proof.privateTranscriptOutputs, 'ordered');
for (const active of [false, true]) {
  const reached = ordered.circuits.ordered(orderedContext, active, nullifier, changed);
  assert.equal(reached.result, 1n);
  assert.equal(reached.context.events.length, active ? 2 : 1);
  assert.deepEqual(reached.context.events.map(event => event.data.content.value),
    (active ? [nullifier, changed] : [changed]).map(bytes => new runtime.CompactTypeBytes(32).toValue(bytes)));
  const preimage = orderedData(reached.proofData);
  await check(preimage, orderedOpts);
  await writeFile(join(output, `ordered-${active}.preimage`), preimage, { flag: 'wx' });
  if (active) {
    const transcript = reached.proofData.publicTranscript;
    assert.equal(transcript.length, 4);
    assert.equal(transcript[1], 'log');
    assert.equal(transcript[3], 'log');
    await assert.rejects(check(orderedData({ ...reached.proofData,
      publicTranscript: [...transcript.slice(2), ...transcript.slice(0, 2)],
    }), orderedOpts));
  }
}
assert.equal(Object.keys(ordered.provableCircuits).length, 2);
assert.throws(() => ordered.circuits.rollback(orderedContext, false, nullifier), /rollback after event/);
assert.deepEqual(orderedContext.events, []);
const accepted = ordered.circuits.rollback(orderedContext, true, nullifier);
assert.equal(accepted.context.events.length, 1);
assert.deepEqual(orderedContext.events, []);
console.log(JSON.stringify({ gas: Object.fromEntries(Object.entries(first.gasCost).map(([k, v]) => [k, String(v)])) }));
console.log('guarded event circuits stay provable and failed assertions roll back their events');
console.log('two ordered VM logs and state updates match upstream fixture; wrong width and four transcript mutations refused');
