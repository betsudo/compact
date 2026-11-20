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
  throw new Error('Usage: node check-uint.mjs <uint-output> <stateful-output> <zkir-wasm-package> <new-output-dir>');
}
const require = createRequire(resolve(stateful, 'contract/index.js'));
const runtimePath = require.resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimePath));
const ocrt = await import(pathToFileURL(require.resolve('@midnightntwrk/onchain-runtime-v4', { paths: [runtimePath] })));
const { pureCircuits } = await import(pathToFileURL(resolve(plain, 'contract/index.js')));
const { Contract, ledger } = await import(pathToFileURL(resolve(stateful, 'contract/index.js')));
const bytes = (...values) => Uint8Array.from(values);
for (const width of [8, 64, 128]) {
  const max = (1n << BigInt(width)) - 1n;
  for (const value of [0n, 1n, 128n, max - 1n, max]) {
    const expected = Uint8Array.from({ length: width / 8 }, (_, i) => Number((value >> BigInt(i * 8)) & 255n));
    assert.deepEqual(pureCircuits['serialize_uint' + width](value), expected);
    assert.equal(pureCircuits['deserialize_uint' + width](expected), value);
  }
}
assert.deepEqual(pureCircuits.encode_word(511n), bytes(255, 1, 0, 0));
assert.equal(pureCircuits.decode_word(bytes(255, 1, 99, 255)), 511n);
assert.throws(() => pureCircuits.decode_word(bytes(0, 2, 0, 0)), runtime.CompactError);
assert.deepEqual(pureCircuits.zero_encoding(0n), bytes(0, 0));
assert.equal(pureCircuits.zero_decoding(bytes(255, 255)), 0n);
assert.deepEqual(pureCircuits.narrow_cast(255n), bytes(255));
assert.throws(() => pureCircuits.narrow_cast(256n), runtime.CompactError);
const extended = new Uint8Array(17).fill(255, 0, 16);
assert.equal(pureCircuits.extended_cast(extended), (1n << 128n) - 1n);
extended[16] = 1;
assert.throws(() => pureCircuits.extended_cast(extended), runtime.CompactError);

const name = 'store_word';
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
const inputType = new runtime.CompactTypeBytes(2);
const alignInput = (input) => ({ value: inputType.toValue(input), alignment: inputType.alignment() });
await mkdir(output);
await writeFile(join(output, name + '.bzkir'), ir, { flag: 'wx' });
for (const [index, [input, expected]] of [[bytes(0, 0), 0n], [bytes(255, 1), 511n]].entries()) {
  const result = contract.circuits.store_word(context, input);
  assert.equal(result.result, expected);
  assert.equal(read(result.context), expected);
  assert.equal(read(context), 0n);
  const data = result.proofData;
  assert.deepEqual(data.input, alignInput(input));
  assert.deepEqual(data.privateTranscriptOutputs, []);
  const preimage = (argument, outputValue) => ocrt.proofDataIntoSerializedPreimage(argument, outputValue,
    data.publicTranscript, data.privateTranscriptOutputs, name);
  const valid = preimage(data.input, data.output);
  await check(valid, options);
  await assert.rejects(check(preimage(alignInput(bytes(0, 2)), data.output), options));
  const wrong = structuredClone(data.output);
  wrong.value = ocrt.bigIntToValue(expected === 0n ? 1n : 0n);
  await assert.rejects(check(preimage(data.input, wrong), options));
  await writeFile(join(output, name + '-' + index + '.preimage'), valid, { flag: 'wx' });
}
assert.throws(() => contract.circuits.store_word(context, bytes(0, 2)), runtime.CompactError);
assert.throws(() => contract.circuits.store_word(context, bytes(0)), /Bytes<2>/);
assert.equal(read(context), 0n);
console.log('Uint<8/64/128> wire pairs, nominal Uint<9>, zero width and checked casts passed; overflow and changed output refused in the stateful SDK model.');
