// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [entry, wasmPackage, output] = process.argv.slice(2).map((path) => resolve(path));
assert(entry && wasmPackage && output);
const packageJson = JSON.parse(await readFile(join(wasmPackage, 'package.json')));
assert.equal(packageJson.version, '3.2.0');
const { check, jsonIrToBinary } = await import(pathToFileURL(join(wasmPackage, packageJson.exports.node)));
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry));
const onchainEntry = createRequire(runtimeEntry).resolve('@midnightntwrk/onchain-runtime-v4');
const { proofDataIntoSerializedPreimage } = await import(pathToFileURL(onchainEntry));
const code = await import(pathToFileURL(entry));
const rawPath = resolve(dirname(entry), '../zkir/castNative.zkir');
const raw = await readFile(rawPath, 'utf8');
const ir = jsonIrToBinary(raw);
assert.deepEqual(Buffer.from(ir), await readFile(rawPath.replace(/\.zkir$/, '.bzkir')));
const materials = {
  lookupKey: async () => ({ proverKey: new Uint8Array(), verifierKey: new Uint8Array(), ir }),
  getParams: async () => { throw Error('unexpected parameter request'); },
};

const contract = new code.Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const context = () => runtime.createCircuitContext(runtime.dummyContractAddress(),
  initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
const execution = contract.circuits.castNative(context(), new Uint8Array(32), 1n, 2n);
assert.deepEqual(execution.result, []);
assert.equal(code.ledger(execution.context.currentQueryContext.state).f, (1n << 248n) + 5n);
const proof = execution.proofData;
assert.deepEqual(proof.privateTranscriptOutputs, []);
const serialize = (transcript) => proofDataIntoSerializedPreimage(
  proof.input, proof.output, transcript, proof.privateTranscriptOutputs, 'castNative');
const preimage = serialize(proof.publicTranscript);
await check(preimage, materials);
const changed = structuredClone(proof.publicTranscript);
const cell = changed.find((operation) => operation.push?.storage === true && operation.push.value?.tag === 'cell');
assert(cell, 'castNative must write a cell');
cell.push.value.content.value = runtime.CompactTypeField.toValue(0n);
const altered = serialize(changed);
await assert.rejects(() => check(altered, materials), /transcript input mismatch/i);

await mkdir(output);
await writeFile(join(output, 'castNative.preimage'), preimage, { flag: 'wx' });
await writeFile(join(output, 'changed-write.preimage'), altered, { flag: 'wx' });
const sha = (bytes) => createHash('sha256').update(bytes).digest('hex');
const receipt = {
  source: entry,
  raw_zkir: rawPath,
  raw_sha256: sha(raw),
  binary_sha256: sha(ir),
  preimage_sha256: sha(preimage),
  changed_write_sha256: sha(altered),
  honest_check: 'passed',
  changed_write_check: 'refused',
};
await writeFile(join(output, 'capture.json'), JSON.stringify(receipt, null, 2) + '\n', { flag: 'wx' });
console.log(receipt);
