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
  throw new Error('Usage: node check.mjs <boolean-output> <stateful-output> <zkir-wasm-package> <new-output-dir>');
}
const require = createRequire(resolve(stateful, 'contract/index.js'));
const runtimePath = require.resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimePath));
const ocrt = await import(pathToFileURL(require.resolve('@midnightntwrk/onchain-runtime-v4', { paths: [runtimePath] })));
const { pureCircuits } = await import(pathToFileURL(resolve(plain, 'contract/index.js')));
const { Contract } = await import(pathToFileURL(resolve(stateful, 'contract/index.js')));
const manifest = JSON.parse(await readFile(join(sdk, 'package.json'), 'utf8'));
assert.equal(manifest.version, '3.2.0');
const { check, jsonIrToBinary } = await import(pathToFileURL(resolve(sdk, manifest.exports.node)));
const bytes = (...values) => Uint8Array.from(values);
const name = 'd1';
const ir = jsonIrToBinary(await readFile(join(stateful, 'zkir/d1.zkir'), 'utf8'));
assert.deepEqual(Buffer.from(ir), await readFile(join(stateful, 'zkir/d1.bzkir')));
const options = {
  lookupKey: async () => ({ ir, proverKey: bytes(), verifierKey: bytes() }),
  getParams: async () => { throw Error('unexpected parameter request'); },
};
const contract = new Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const context = runtime.createCircuitContext(runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState);
const read = (ctx) => runtime.CompactTypeField.fromValue(ctx.currentQueryContext.state.state.asArray()[0].asCell().value);
const byteType = new runtime.CompactTypeBytes(1);
const encode = (value) => ({ value: byteType.toValue(value), alignment: byteType.alignment() });
const preimage = (data, input = data.input, result = data.output) =>
  ocrt.proofDataIntoSerializedPreimage(input, result, data.publicTranscript, data.privateTranscriptOutputs, name);
await mkdir(output);
const proofData = [];
for (const value of [0, 1]) {
  const expected = value === 1;
  assert.deepEqual(pureCircuits.serialize_boolean(expected), bytes(value));
  assert.equal(pureCircuits.deserialize_boolean(bytes(value)), expected);
  const outcome = contract.circuits.d1(context, bytes(value));
  assert.equal(outcome.result, expected);
  assert.equal(read(outcome.context), 7n);
  assert.equal(read(context), 0n);
  const data = outcome.proofData;
  proofData.push(data);
  assert.deepEqual(data.input, encode(bytes(value)));
  assert.deepEqual(data.privateTranscriptOutputs, []);
  const model = preimage(data);
  await check(model, options);
  await writeFile(join(output, `d1-${value}.preimage`), model, { flag: 'wx' });
}
for (let value = 2; value <= 255; value += 1) {
  assert.throws(() => pureCircuits.deserialize_boolean(bytes(value)), /cast from Field or Uint.*smaller Uint/i);
  assert.throws(() => contract.circuits.d1(context, bytes(value)), /cast from Field or Uint.*smaller Uint/i);
  assert.equal(read(context), 0n);
  await assert.rejects(check(preimage(proofData[0], encode(bytes(value))), options));
}
assert.throws(() => pureCircuits.deserialize_boolean(bytes(1, 0)), /Bytes<1>/);
assert.throws(() => pureCircuits.serialize_boolean(1), /Boolean/);
await assert.rejects(check(preimage(proofData[0], proofData[0].input, proofData[1].output), options));
console.log('Both Boolean encodings pass. All 254 other byte values fail in TypeScript and ZKIR; wrong width, type and output also fail.');
