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

// Test-local actual JS execution and official preimage producer. This does not prove.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

if (process.argv.length !== 4) {
  throw new Error('usage: node capture_js_counter.mjs COMPILED_CONTRACT_DIR NEW_OUTPUT_DIR');
}
const index = resolve(process.argv[2], 'index.js');
const output = resolve(process.argv[3]);
const requireFromContract = createRequire(index);
const runtimeEntry = requireFromContract.resolve('@midnight-ntwrk/compact-runtime');
const requireFromRuntime = createRequire(runtimeEntry);
const runtimePackage = requireFromContract.resolve('@midnight-ntwrk/compact-runtime/package.json');
const onchainEntry = requireFromRuntime.resolve('@midnight-ntwrk/onchain-runtime-v3');
const onchainPackage = resolve(dirname(onchainEntry), 'package.json');
const onchainWasm = resolve(dirname(onchainEntry), 'midnight_onchain_runtime_wasm_bg.wasm');
const onchainMetadata = JSON.parse(readFileSync(onchainPackage, 'utf8'));
// This pinned package selects the same Node entry for ESM and require. Refuse
// a future split until its actual loaded implementation is separately qualified.
assert.equal(onchainMetadata.exports.import, undefined);
assert.equal(onchainMetadata.exports.require, undefined);
assert.equal(resolve(dirname(onchainEntry), onchainMetadata.exports.node), onchainEntry);
const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
function runtimeFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) return runtimeFiles(path);
    return entry.name.endsWith('.js') ? [path] : [];
  });
}
const identities = Object.fromEntries([
  process.execPath, resolve(process.argv[1]), index, runtimePackage,
  ...runtimeFiles(dirname(runtimeEntry)), onchainPackage, onchainEntry, onchainWasm,
  resolve(dirname(onchainEntry), 'midnight_onchain_runtime_wasm_bg.js'),
].map(path => [path, hash(path)]));
const before = identities[index];
// Freeze selected implementation bytes before loading either generated code or
// runtime. Versions alone do not establish a producer or serializer identity.
const runtime = await import(pathToFileURL(runtimeEntry));
const { Contract, ledger } = await import(pathToFileURL(index));
const coinPublicKey = { bytes: new Uint8Array(32) };
mkdirSync(output); // Refuse an existing destination; no cached execution verdicts.
const cases = [];
for (const [branch, amount, remove, expected] of [[0, 3n, 2n, 1n], [1, 10n, 65535n, 0n]]) {
  const contract = new Contract({});
  const initial = contract.initialState({
    initialPrivateState: 7n,
    initialZswapLocalState: runtime.emptyZswapLocalState(coinPublicKey),
  });
  assert.equal(ledger(initial.currentContractState.data).round, 0n);
  assert.equal(ledger(initial.currentContractState.data).peer, 10n);
  const initialState = initial.currentContractState.data.state.encode();
  const context = runtime.createCircuitContext(runtime.dummyContractAddress(), coinPublicKey,
    initial.currentContractState.data, initial.currentPrivateState);
  const actual = contract.circuits.adjust(context, amount, remove);
  const state = actual.context.currentQueryContext.state.state;
  assert.equal(actual.result, expected);
  assert.equal(ledger(state).round, expected);
  assert.equal(ledger(state).peer, 10n);
  assert.equal(actual.context.currentPrivateState, 7n);
  assert.deepEqual(actual.proofData.privateTranscriptOutputs, []);
  const proof = actual.proofData;
  const preimage = runtime.proofDataIntoSerializedPreimage(proof.input, proof.output,
    proof.publicTranscript, proof.privateTranscriptOutputs, 'adjust');
  writeFileSync(resolve(output, `branch-${branch}.preimage`), preimage, { flag: 'wx' });
  const transport = JSON.stringify({ branch, amount: amount.toString(), remove: remove.toString(),
    expected: expected.toString(), input: proof.input, output: proof.output,
    publicTranscript: proof.publicTranscript, privateTranscriptOutputs: proof.privateTranscriptOutputs,
    initialState, finalState: state.encode() },
  (_, value) => typeof value === 'bigint' ? value.toString()
    : value instanceof Uint8Array ? Array.from(value) : value, 2);
  writeFileSync(resolve(output, `branch-${branch}.json`), transport + '\n', { flag: 'wx' });
  // The generated argument guard must execute; a Rust recomputation cannot supply this result.
  assert.throws(() => contract.circuits.adjust(context, -1n, 0n), error =>
    error instanceof runtime.CompactError && /type error: adjust argument 1/.test(error.message)
      && error.message.includes('Uint<0..65536>'));
  cases.push({ branch, expected: expected.toString(), preimage_bytes: preimage.length,
    transcript_operations: proof.publicTranscript.length, negative_argument_refused: true });
}
for (const [path, expected] of Object.entries(identities)) assert.equal(hash(path), expected);
writeFileSync(resolve(output, 'execution.json'), JSON.stringify({
  scope: 'actual generated JS execution and official preimage serialization; no proof or ledger apply',
  node: process.version, generated_index_sha256: before,
  runtime_entry: runtimeEntry, runtime_entry_sha256: hash(runtimeEntry), cases,
  runtime_package: JSON.parse(readFileSync(runtimePackage, 'utf8')).version,
  onchain_package: onchainMetadata.version,
  onchain_entry: onchainEntry, onchain_entry_sha256: hash(onchainEntry),
  onchain_wasm: onchainWasm, onchain_wasm_sha256: hash(onchainWasm),
  node_executable: process.execPath, node_executable_sha256: identities[process.execPath],
  producer_files_sha256: identities,
}, null, 2) + '\n', { flag: 'wx' });
console.log('actual JS counter branches and argument refusals executed; official preimages emitted');
