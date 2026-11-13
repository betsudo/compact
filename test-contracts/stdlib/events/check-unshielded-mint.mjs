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
  throw Error('Usage: node check-unshielded-mint.mjs <compiled-contract> <zkir-wasm-package> <new-output-dir>');
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
const input = new Uint8Array(32).fill(0x42);
const token = new Uint8Array(32).fill(0xa5);
const descriptor = new runtime.CompactTypeBytes(80);
const payload = (amount, tokenType = input) => {
  const bytes = new Uint8Array(80);
  bytes.set(input);
  bytes.set(tokenType, 32);
  for (let i = 0; i < 16; i++) {
    bytes[64 + i] = Number((amount >> BigInt(i * 8)) & 255n);
  }
  return bytes;
};
const mint = { domainSep: input, tokenType: token, amount: (1n << 128n) - 1n };
const encoded = payload(mint.amount, token);
assert.deepEqual(pureCircuits.serialize_UnshieldedMint(mint), encoded);
assert.deepEqual(pureCircuits.deserialize_UnshieldedMint(encoded), mint);
assert.deepEqual(pureCircuits.roundtrip_UnshieldedMint(mint), mint);
assert.deepEqual(instance.circuits.no_emit(ctx, input).context.events, []);
await mkdir(output);
for (const [name, variant, inputs, expected] of [
  ['emit_one', 'one', [input], [payload(1000n)]],
  ['emit_two', 'two', [input], [payload(1n), payload(2n)]],
  ['cond_emit', 'true', [true, input], [payload(5n)]],
  ['cond_emit', 'false', [false, input], []],
]) {
  const outcome = instance.circuits[name](ctx, ...inputs);
  assert.deepEqual(outcome.result, input);
  assert.equal(outcome.context.events.length, expected.length);
  for (const [i, event] of outcome.context.events.entries()) {
    assert.equal(event.address, address);
    assert.equal(event.version, 1);
    assert.equal(event.eventType, 'unshielded-mint');
    assert.equal(event.data.tag, 'cell');
    assert.deepEqual(event.data.content.alignment, descriptor.alignment());
    assert.deepEqual(event.data.content.value, descriptor.toValue(expected[i]));
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
    const [first, second] = pushes;
    const payloadCell = transcript[first].push.value.content[2];
    const original = payloadCell.content.value;
    payloadCell.content.value = descriptor.toValue(payload(1n, token));
    await assert.rejects(check(preimage(transcript), options));
    payloadCell.content.value = original;
    [transcript[first], transcript[second]] = [transcript[second], transcript[first]];
    await assert.rejects(check(preimage(transcript), options));
  }
  console.log(name, variant, 'exact event payload and SDK statement passed');
}
