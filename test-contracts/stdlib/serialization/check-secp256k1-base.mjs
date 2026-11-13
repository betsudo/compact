// This file is part of Compact.
// Copyright (C) 2025 Midnight Foundation
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
import { pathToFileURL } from 'node:url';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';

const [build, sdk, destination] = process.argv.slice(2);
if (process.argv.length !== 5) {
  throw Error('Usage: node check-secp256k1-base.mjs <compiled-contract> <zkir-wasm-package> <new-output-dir>');
}
const dir = resolve(build);
const url = pathToFileURL(dir + '/contract/index.js');
const require = createRequire(url);
const runtimePath = require.resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimePath));
const onchain = await import(pathToFileURL(require.resolve('@midnightntwrk/onchain-runtime-v4', { paths: [runtimePath] })));
const {Contract, ledger, pureCircuits: p} = await import(url);
const modulus = (1n << 256n) - (1n << 32n) - 977n;
const wire = (n, width) => Uint8Array.from({length:width}, (_, i) => Number((n >> BigInt(i * 8)) & 255n));
const contract = new Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const ctx = runtime.createCircuitContext(runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
const root = resolve(sdk);
const manifest = JSON.parse(await readFile(join(root, 'package.json'), 'utf8'));
assert.equal(manifest.version, '3.2.0');
const {check, jsonIrToBinary} = await import(pathToFileURL(join(root, manifest.exports.node)));
const ir = jsonIrToBinary(await readFile(dir + '/zkir/store_word.zkir', 'utf8'));
const opts = {
  lookupKey: async () => ({ ir, proverKey: new Uint8Array(), verifierKey: new Uint8Array() }),
  getParams: async () => { throw Error('unexpected params'); },
};
await mkdir(destination);
await writeFile(join(destination, 'store_word.bzkir'), ir);
for (const [i, input] of [0n, 1n, modulus - 1n, modulus, modulus + 1n, (1n << 256n) - 1n].entries()) {
  const value = input % modulus;
  assert.deepEqual(p.encode(value), wire(value, 34));
  const bytes = wire(input, 34);
  bytes.fill(0xff, 32);
  assert.equal(p.decode(bytes), value);
  const nested = new Uint8Array(34);
  nested[0] = 1;
  nested.set(wire(value, 32), 1);
  assert.deepEqual(p.nested([true, value]), nested);
  const outcome = contract.circuits.store_word(ctx, wire(input, 32));
  assert.equal(outcome.result, value);
  assert.equal(ledger(outcome.context.currentQueryContext.state).stored, value);
  const data = outcome.proofData;
  const preimage = onchain.proofDataIntoSerializedPreimage(data.input, data.output, data.publicTranscript, data.privateTranscriptOutputs, 'store_word');
  await check(preimage, opts);
  const changedOutput = structuredClone(data.output);
  changedOutput.value[0] = onchain.bigIntToValue((value + 1n) % modulus)[0];
  const changedPreimage = onchain.proofDataIntoSerializedPreimage(data.input, changedOutput, data.publicTranscript, data.privateTranscriptOutputs, 'store_word');
  await assert.rejects(check(changedPreimage, opts));
  await writeFile(join(destination, 'store_word-' + i + '.preimage'), preimage);
}
console.log('TypeScript and SDK accepted six exact Secp256k1Base wire vectors, including modular reduction and padding.');
