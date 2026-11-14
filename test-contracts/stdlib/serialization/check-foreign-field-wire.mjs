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
  throw Error('Usage: node check-foreign-field-wire.mjs <compiled-contract> <zkir-wasm-package> <new-output-dir>');
}
const url = pathToFileURL(resolve(build, 'contract/index.js'));
const require = createRequire(url);
const runtimePath = require.resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimePath));
const onchain = await import(pathToFileURL(require.resolve('@midnightntwrk/onchain-runtime-v4', { paths: [runtimePath] })));
const {Contract, ledger, pureCircuits} = await import(url);
const moduli = {
  k1_scalar: 0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141n,
  r1_base: 0xffffffff00000001000000000000000000000000ffffffffffffffffffffffffn,
  r1_scalar: 0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551n,
  c25519_base: (1n << 255n) - 19n,
  c25519_scalar: (1n << 252n) + 27742317777372353535851937790883648493n,
};
const wire = (n, width) => Uint8Array.from(
  {length: width}, (_, i) => Number((n >> BigInt(i * 8)) & 255n),
);
const contract = new Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const ctx = runtime.createCircuitContext(runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState);
const root = resolve(sdk);
const manifest = JSON.parse(await readFile(join(root, 'package.json'), 'utf8'));
assert.equal(manifest.version, '3.2.0');
const {check, jsonIrToBinary} = await import(pathToFileURL(join(root, manifest.exports.node)));
await mkdir(destination);

for (const [carrier, modulus] of Object.entries(moduli)) {
  const entry = 'store_' + carrier;
  const model = await readFile(resolve(build, 'zkir', entry + '.zkir'), 'utf8');
  const ir = jsonIrToBinary(model);
  const options = {
    lookupKey: async () => ({ ir, proverKey: new Uint8Array(), verifierKey: new Uint8Array() }),
    getParams: async () => { throw Error('unexpected params'); },
  };
  const serialize = (data, output) => onchain.proofDataIntoSerializedPreimage(
    data.input, output, data.publicTranscript, data.privateTranscriptOutputs, entry,
  );
  await writeFile(join(destination, entry + '.bzkir'), ir);
  for (const [index, input] of [0n, 1n, modulus - 1n, modulus, modulus + 1n, (1n << 256n) - 1n].entries()) {
    const value = input % modulus;
    assert.deepEqual(pureCircuits['encode_' + carrier](value), wire(value, 34));
    const padded = wire(input, 34);
    padded.fill(255, 32);
    assert.equal(pureCircuits['decode_' + carrier](padded), value);
    const outcome = contract.circuits[entry](ctx, wire(input, 32));
    assert.equal(outcome.result, value);
    assert.equal(ledger(outcome.context.currentQueryContext.state)['stored_' + carrier], value);
    const data = outcome.proofData;
    const preimage = serialize(data, data.output);
    await check(preimage, options);
    const substituted = contract.circuits[entry](ctx, wire((value + 1n) % modulus, 32));
    await assert.rejects(check(serialize(data, substituted.proofData.output), options));
    await writeFile(join(destination, entry + '-' + index + '.preimage'), preimage);
  }
  console.log(carrier + ': six SDK statements accepted and six changed results refused');
}

const pair = [moduli.r1_base - 1n, moduli.c25519_scalar - 1n];
const expected = new Uint8Array(65);
expected.set(wire(pair[0], 32));
expected.set(wire(pair[1], 32), 32);
assert.deepEqual(pureCircuits.encode_pair(pair), expected);
expected[64] = 255;
assert.deepEqual(pureCircuits.decode_pair(expected), pair);
expected.set(wire(moduli.r1_base + 1n, 32));
expected.set(wire(moduli.c25519_scalar, 32), 32);
assert.deepEqual(pureCircuits.decode_pair(expected), [1n, 0n]);
