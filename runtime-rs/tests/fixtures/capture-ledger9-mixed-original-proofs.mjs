// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/** Capture the intact mixed Jubjub/K1 and Jubjub/P-256 proof statements. */
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [caseName, entryArg, rawArg, wasmArg, outputArg] = process.argv.slice(2);
assert.ok(['k1', 'r1'].includes(caseName) && entryArg && rawArg && wasmArg && outputArg);
const entry = resolve(entryArg);
const rawPath = resolve(rawArg);
const wasmPath = resolve(wasmArg);
const output = resolve(outputArg);
const sha = (bytes) => createHash('sha256').update(bytes).digest('hex');

const wasmManifest = JSON.parse(await readFile(join(wasmPath, 'package.json')));
assert.equal(wasmManifest.version, '3.2.0');
const { check, jsonIrToBinary } = await import(pathToFileURL(join(wasmPath, wasmManifest.exports.node)));
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtimeManifest = JSON.parse(await readFile(createRequire(entry).resolve('@midnight-ntwrk/compact-runtime/package.json')));
assert.equal(runtimeManifest.version, '0.18.0');
const runtime = await import(pathToFileURL(runtimeEntry));
const onchainEntry = createRequire(runtimeEntry).resolve('@midnightntwrk/onchain-runtime-v4');
const onchainManifest = JSON.parse(await readFile(join(dirname(onchainEntry), 'package.json')));
assert.equal(onchainManifest.version, '4.0.1');
const { proofDataIntoSerializedPreimage } = await import(pathToFileURL(onchainEntry));
const { Contract, ledger } = await import(pathToFileURL(entry));

const contract = new Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const context = runtime.createCircuitContext(runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
const call = contract.circuits.test(context, 3n, 4n);
assert.deepEqual(call.result, []);
const state = ledger(call.context.currentQueryContext.state);
const multiply = caseName === 'k1' ? runtime.secp256k1MulGenerator : runtime.secp256r1MulGenerator;
const tuple = caseName === 'k1' ? state.secp256k1Tuple : state.secp256r1Tuple;
assert.deepEqual(state.jubjubPoints.lookup(1n), runtime.ecMulGenerator(3n));
assert.deepEqual(tuple, [
  { x: 0n, y: 0n, identity: true }, multiply(4n), multiply(8n),
]);
assert.deepEqual(state.pointStruct, {
  jp: runtime.ecMulGenerator(3n), color: [204n, 85n, 0n], sp: multiply(12n),
});
assert.deepEqual(state.jubjubTuples.lookup(0n), [
  runtime.ecMulGenerator(0n), runtime.ecMulGenerator(3n), runtime.ecMulGenerator(3n),
]);
assert.deepEqual(state.jubjubMap.lookup(0n).lookup(1n), runtime.ecMulGenerator(3n));
const proof = call.proofData;
assert.ok(proof);
assert.deepEqual(proof.privateTranscriptOutputs, []);
const raw = await readFile(rawPath, 'utf8');
const ir = jsonIrToBinary(raw);
const options = {
  lookupKey: async () => ({ proverKey: new Uint8Array(), verifierKey: new Uint8Array(), ir }),
  getParams: async () => { throw new Error('parameters requested during capture'); },
};
const serialize = transcript => proofDataIntoSerializedPreimage(proof.input, proof.output, transcript, proof.privateTranscriptOutputs, 'test');
const honest = serialize(proof.publicTranscript);
await check(honest, options);
const changed = structuredClone(proof.publicTranscript);
const nested = changed.findLast(op => op.idx?.path.length === 2);
assert.ok(nested, 'original nested Map lookup missing');
// The zero Uint<32> key has an empty canonical payload. Replace it with 1.
assert.equal(nested.idx.path[1].value.value[0].length, 0);
nested.idx.path[1].value.value[0] = new Uint8Array([1]);
const invalid = serialize(changed);
await assert.rejects(() => check(invalid, options), /transcript input mismatch/i);

await mkdir(output);
await writeFile(join(output, 'honest.preimage'), honest);
await writeFile(join(output, 'changed-key.preimage'), invalid);
await writeFile(join(output, 'test.bzkir'), ir);
const receipt = {
  case: caseName,
  circuit: 'test',
  runtime: runtimeManifest.version,
  onchainRuntime: onchainManifest.version,
  zkirWasm: wasmManifest.version,
  sourceModuleSha256: sha(await readFile(entry)),
  rawZkirSha256: sha(raw),
  binaryZkirSha256: sha(ir),
  honestPreimageSha256: sha(honest),
  changedKeyPreimageSha256: sha(invalid),
  publicTranscriptOperations: proof.publicTranscript.length,
  checks: ['all original callback assertions and nested Map point', 'honest ZKIR accepted', 'changed nested Map key refused'],
};
await writeFile(join(output, 'capture.json'), JSON.stringify(receipt, null, 2)+'\n');
console.log(`${caseName} original mixed collection capture passed`);
