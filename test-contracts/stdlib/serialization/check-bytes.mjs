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
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [plain, stateful, sdk, output] = process.argv.slice(2);
if (process.argv.length !== 6) {
  throw new Error('Usage: node check-bytes.mjs <bytes-output> <stateful-output> <zkir-wasm-package> <new-output-dir>');
}

const require = createRequire(resolve(stateful, 'contract/index.js'));
const runtimePath = require.resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimePath));
const ocrt = await import(pathToFileURL(require.resolve('@midnightntwrk/onchain-runtime-v4', { paths: [runtimePath] })));
const { pureCircuits } = await import(pathToFileURL(resolve(plain, 'contract/index.js')));
const { Contract, ledger } = await import(pathToFileURL(resolve(stateful, 'contract/index.js')));
const bytes = (...values) => Uint8Array.from(values);

const original = bytes(0, 1, 127, 128, 255, 0, 16, 2, 1, 0);
assert.deepEqual(pureCircuits.serialize_bytes(original), original);
assert.deepEqual(pureCircuits.deserialize_bytes(original), original);
assert.deepEqual(pureCircuits.pad_payload(bytes(0x80, 0xff)), bytes(0x80, 0xff, 0, 0));
assert.deepEqual(pureCircuits.decode_payload(bytes(0x80, 0xff, 0x55, 0xaa)), bytes(0x80, 0xff));
assert.deepEqual(pureCircuits.decode_empty(bytes(0x80, 0xff, 0x55, 0xaa)), bytes());
assert.deepEqual(pureCircuits.pad_empty(bytes()), bytes(0, 0, 0, 0));
assert.throws(() => pureCircuits.serialize_bytes(bytes(0)), /Bytes<10>/);
assert.throws(() => pureCircuits.deserialize_bytes(Array(10).fill(0)), /Bytes<10>/);

const name = 'store_prefix';
const manifest = JSON.parse(await readFile(join(sdk, 'package.json'), 'utf8'));
assert.equal(manifest.version, '3.2.0');
const { check, jsonIrToBinary } = await import(pathToFileURL(resolve(sdk, manifest.exports.node)));
const ir = jsonIrToBinary(await readFile(join(stateful, 'zkir', name + '.zkir'), 'utf8'));
const options = {
  lookupKey: async () => ({ ir, proverKey: bytes(), verifierKey: bytes() }),
  getParams: async () => { throw Error('unexpected parameter request'); },
};
const contract = new Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const context = runtime.createCircuitContext(runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState);
const read = (ctx) => ledger(ctx.currentQueryContext.state).stored;
const aligned = (length, value) => {
  const type = new runtime.CompactTypeBytes(length);
  return { value: type.toValue(value), alignment: type.alignment() };
};
await mkdir(output);
await writeFile(join(output, name + '.bzkir'), ir, { flag: 'wx' });
for (const [index, input] of [bytes(0x80, 0xff, 0x55), bytes(0x80, 0xff, 0xaa), bytes(0, 1, 255)].entries()) {
  const expected = bytes(input[0], input[1], 0, 0);
  const outcome = contract.circuits.store_prefix(context, input);
  assert.deepEqual(outcome.result, expected);
  assert.deepEqual(read(outcome.context), expected);
  assert.deepEqual(read(context), bytes(0, 0, 0, 0));
  const data = outcome.proofData;
  assert.deepEqual(data.input, aligned(3, input));
  assert.deepEqual(data.output, aligned(4, expected));
  assert.deepEqual(data.privateTranscriptOutputs, []);
  const preimage = (argument, result) => ocrt.proofDataIntoSerializedPreimage(argument, result,
    data.publicTranscript, data.privateTranscriptOutputs, name);
  const valid = preimage(data.input, data.output);
  await check(valid, options);
  await assert.rejects(check(preimage(data.input, aligned(4, bytes(input[0], input[1], 1, 0))), options));
  await assert.rejects(check(preimage(aligned(3, bytes(input[0] ^ 1, input[1], input[2])), data.output), options));
  await writeFile(join(output, `store_prefix-${index}.preimage`), valid, { flag: 'wx' });
}
assert.throws(() => contract.circuits.store_prefix(context, bytes(0, 0)), /Bytes<3>/);
assert.deepEqual(read(context), bytes(0, 0, 0, 0));
console.log('Exact upstream Bytes exports, aliases, empty values, and three stateful prefix/padding cases passed. Wrong width, type, prefix, and padding controls were refused.');
