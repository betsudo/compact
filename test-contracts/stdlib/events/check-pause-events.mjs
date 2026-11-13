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
import { createRequire } from 'node:module';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [paused, transition, sdk, output] = process.argv.slice(2);
if (process.argv.length !== 6) {
  throw Error('Usage: node check-pause-events.mjs <paused> <transition> <zkir-wasm-package> <new-output-dir>');
}
const entry = (build) => pathToFileURL(resolve(build, 'contract/index.js'));
const require = createRequire(entry(paused));
const runtimePath = require.resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimePath));
const onchain = await import(pathToFileURL(require.resolve('@midnightntwrk/onchain-runtime-v4', { paths: [runtimePath] })));
const manifest = JSON.parse(await readFile(resolve(sdk, 'package.json'), 'utf8'));
assert.equal(manifest.version, '3.2.0');
const { check, jsonIrToBinary } = await import(pathToFileURL(resolve(sdk, manifest.exports.node)));

const address = runtime.dummyContractAddress();
const input = new Uint8Array(32).fill(0x42);
const empty = new Uint8Array();
const bytes = new runtime.CompactTypeBytes(0);
async function initialize(build) {
  const { Contract, pureCircuits } = await import(entry(build));
  const instance = new Contract({});
  const initial = instance.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
  const ctx = runtime.createCircuitContext(address, initial.currentZswapLocalState.coinPublicKey,
    initial.currentContractState, initial.currentPrivateState);
  return { instance, pureCircuits, ctx };
}
const pause = await initialize(paused);
const resume = await initialize(transition);
assert.deepEqual(pause.pureCircuits.serialize_Paused({}), empty);
assert.deepEqual(pause.pureCircuits.deserialize_Paused(empty), {});
assert.deepEqual(pause.pureCircuits.roundtrip_Paused({}), {});
assert.deepEqual(resume.pureCircuits.serialize_Unpaused({}), empty);
assert.deepEqual(resume.pureCircuits.deserialize_Unpaused(empty), {});
for (const [runtimeExport, codec] of [
  [runtime.emitPaused, pause.pureCircuits.deserialize_Paused],
  [runtime.emitUnpaused, resume.pureCircuits.deserialize_Unpaused],
]) {
  const malformed = Uint8Array.of(1);
  assert.throws(() => codec(malformed), runtime.CompactError);
  assert.throws(() => runtimeExport(pause.ctx, {}, malformed), runtime.CompactError);
}
assert.deepEqual(pause.instance.circuits.no_emit(pause.ctx, input).context.events, []);
await mkdir(output);
for (const [build, { instance, ctx }, name, variant, inputs, expected] of [
  [paused, pause, 'emit_one', 'one', [input], ['paused']],
  [paused, pause, 'emit_two', 'two', [input], ['paused', 'paused']],
  [paused, pause, 'cond_emit', 'true', [true, input], ['paused']],
  [paused, pause, 'cond_emit', 'false', [false, input], []],
  [transition, resume, 'transition', 'two', [input], ['paused', 'unpaused']],
]) {
  const outcome = instance.circuits[name](ctx, ...inputs);
  assert.deepEqual(outcome.result, input);
  assert.equal(outcome.context.events.length, expected.length);
  for (const [i, event] of outcome.context.events.entries()) {
    assert.equal(event.address, address);
    assert.equal(event.version, 1);
    assert.equal(event.eventType, expected[i]);
    assert.equal(event.data.tag, 'cell');
    assert.deepEqual(event.data.content.alignment, bytes.alignment());
    assert.deepEqual(event.data.content.value, bytes.toValue(empty));
  }
  assert.deepEqual(ctx.events, []);
  const ir = jsonIrToBinary(await readFile(resolve(build, 'zkir', name + '.zkir'), 'utf8'));
  const options = {
    lookupKey: async () => ({ ir, proverKey: new Uint8Array(), verifierKey: new Uint8Array() }),
    getParams: async () => { throw Error('unexpected parameter lookup'); },
  };
  const data = outcome.proofData;
  const preimage = (transcript) => onchain.proofDataIntoSerializedPreimage(
    data.input, data.output, transcript, data.privateTranscriptOutputs, name,
  );
  const honest = preimage(data.publicTranscript);
  await check(honest, options);
  await writeFile(resolve(output, name + '-' + variant + '.preimage'), honest);
  await writeFile(resolve(output, name + '.bzkir'), ir);
  if (name === 'transition') {
    const transcript = structuredClone(data.publicTranscript);
    const pushes = transcript.flatMap((op, i) => op.push && transcript[i + 1] === 'log' ? [i] : []);
    assert.equal(pushes.length, 2);
    const [first, second] = pushes;
    const tag = transcript[second].push.value.content[1].content;
    const correct = tag.value;
    tag.value = transcript[first].push.value.content[1].content.value;
    await assert.rejects(() => check(preimage(transcript), options));
    tag.value = correct;
    [transcript[first], transcript[second]] = [transcript[second], transcript[first]];
    await assert.rejects(() => check(preimage(transcript), options));
  }
  console.log(name, variant, 'exact empty payload, tags and SDK statement passed');
}
