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

const [build, sdk, output] = process.argv.slice(2);
if (process.argv.length !== 5) {
  throw Error('Usage: node check-misc.mjs <compiled-contract> <zkir-wasm-package> <new-output-dir>');
}
const entry = pathToFileURL(resolve(build, 'contract/index.js'));
const require = createRequire(entry);
const runtimePath = require.resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimePath));
const onchain = await import(pathToFileURL(require.resolve('@midnightntwrk/onchain-runtime-v4', { paths: [runtimePath] })));
const { Contract, pureCircuits } = await import(entry);
const manifest = JSON.parse(await readFile(resolve(sdk, 'package.json'), 'utf8'));
assert.equal(manifest.version, '3.2.0');
const { check, jsonIrToBinary } = await import(pathToFileURL(resolve(sdk, manifest.exports.node)));
const instance = new Contract({});
const initial = instance.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const address = runtime.dummyContractAddress();
const ctx = runtime.createCircuitContext(address, initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState);
const nameBytes = new Uint8Array(32).fill(0x42);
const payloadBytes = new Uint8Array(256).fill(0x77);
payloadBytes[255] = 0;
const descriptor = new runtime.CompactTypeBytes(288);
const encoded = new Uint8Array(288);
encoded.set(nameBytes);
encoded.set(payloadBytes, 32);
const event = { name: nameBytes, payload: payloadBytes };
assert.deepEqual(pureCircuits.serialize_Misc(event), encoded);
assert.deepEqual(pureCircuits.deserialize_Misc(encoded), event);
assert.deepEqual(pureCircuits.roundtrip_Misc(event), event);
const wrongSize = encoded.slice(1);
assert.throws(() => pureCircuits.deserialize_Misc(wrongSize), runtime.CompactError);
assert.throws(() => runtime.emitMisc(ctx, {}, wrongSize), runtime.CompactError);
assert.deepEqual(instance.circuits.no_emit(ctx, nameBytes).context.events, []);
await mkdir(output);
for (const [name, variant, inputs, count] of [
  ['emit_one', 'one', [nameBytes, payloadBytes], 1],
  ['emit_two', 'two', [nameBytes, payloadBytes], 2],
  ['cond_emit', 'true', [true, nameBytes, payloadBytes], 1],
  ['cond_emit', 'false', [false, nameBytes, payloadBytes], 0],
]) {
  const outcome = instance.circuits[name](ctx, ...inputs);
  assert.deepEqual(outcome.result, nameBytes);
  assert.equal(outcome.context.events.length, count);
  for (const event of outcome.context.events) {
    assert.equal(event.address, address);
    assert.equal(event.version, 1);
    assert.equal(event.eventType, 'misc');
    assert.equal(event.data.tag, 'cell');
    assert.deepEqual(event.data.content.alignment, descriptor.alignment());
    assert.deepEqual(event.data.content.value, descriptor.toValue(encoded));
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
  if (variant === 'two') {
    const transcript = structuredClone(data.publicTranscript);
    const pushes = transcript.flatMap((op, i) => op.push && transcript[i + 1] === 'log' ? [i] : []);
    assert.equal(pushes.length, 2);
    const payload = transcript[pushes[1]].push.value.content[2].content;
    for (const index of [0, 32, 287]) {
      const altered = encoded.slice();
      altered[index] ^= 1;
      payload.value = descriptor.toValue(altered);
      await assert.rejects(check(preimage(transcript), options));
    }
    payload.value = descriptor.toValue(encoded);
    const tag = transcript[pushes[1]].push.value.content[1].content;
    tag.value = onchain.bigIntToValue(9n);
    await assert.rejects(check(preimage(transcript), options));
  }
  console.log(name, variant, 'exact event payload and SDK statement passed');
}
