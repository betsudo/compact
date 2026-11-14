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
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { identity, pointAtoms } from '../point/cases.mjs';
import { bytesHex, coordinateHex, encodePoint, signatureCases } from './cases.mjs';

assert.equal(process.argv.length, 5, 'expected family output, SDK WASM package and new output directory');
const [family, wasmPackage, output] = process.argv.slice(2).map((arg) => resolve(arg));
const folder = 'ecdsa/upstream-ordinary';
const build = join(family, folder, 'ts-rust/.build');
const circuit = 'verify';
const source = join(build, 'contract/index.js');
const runtimeEntry = createRequire(source).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry));
const requireRuntime = createRequire(runtimeEntry);
const onchainEntry = requireRuntime.resolve('@midnightntwrk/onchain-runtime-v4');
const nobleEntry = requireRuntime.resolve('@noble/curves/nist.js');
const keccakEntry = requireRuntime.resolve('@noble/hashes/sha3.js');
const { proofDataIntoSerializedPreimage } = await import(pathToFileURL(onchainEntry));
const { p256 } = await import(pathToFileURL(nobleEntry));
const { keccak_256 } = await import(pathToFileURL(keccakEntry));
const manifest = JSON.parse(await readFile(join(wasmPackage, 'package.json')));
assert.equal(manifest.version, '3.2.0');
const wasmEntry = join(wasmPackage, manifest.exports.node);
const { check, jsonIrToBinary } = await import(pathToFileURL(wasmEntry));

const sha = (data) => createHash('sha256').update(data).digest('hex');
const pin = async (path) => ({ path, sha256: sha(await readFile(path)) });
const tools = await Promise.all([runtimeEntry, onchainEntry, nobleEntry, keccakEntry, wasmEntry,
  join(wasmPackage, 'midnight_zkir_wasm_bg.wasm')].map(pin));
const rawPath = join(build, 'zkir/verify.zkir');
const binaryPath = join(build, 'zkir/verify.bzkir');
const infoPath = join(build, 'compiler/contract-info.json');
const raw = await readFile(rawPath, 'utf8');
const program = JSON.parse(raw);
const info = JSON.parse(await readFile(infoPath, 'utf8'));
assert.equal(info.circuits.find(({ name }) => name === circuit).proof, true);
assert(program.do_communications_commitment);
assert.deepEqual(program.inputs.map(({ type }) => type), [
  'Scalar<BLS12-381>', 'Scalar<BLS12-381>', 'Scalar<Secp256r1>', 'Scalar<Secp256r1>', 'Point<Secp256r1>',
]);
assert.deepEqual(program.outputs, ['Scalar<BLS12-381>']);
for (const op of ['keccak256', 'inv', 'ec_mul_generator', 'ec_mul', 'add', 'into_coordinates']) {
  assert(program.instructions.some((row) => row.op === op), op);
}
assert(!program.instructions.some(({ op }) => op === 'private_input'));
const ir = jsonIrToBinary(raw);
assert.deepEqual(Buffer.from(ir), await readFile(binaryPath));
const materials = {
  lookupKey: async () => ({ proverKey: new Uint8Array(), verifierKey: new Uint8Array(), ir }),
  getParams: async () => { throw new Error('unexpected parameter request'); },
};
const artifacts = await Promise.all([rawPath, binaryPath, infoPath, source, join(build, 'contract/lib.rs')].map(pin));
const { Contract } = await import(pathToFileURL(source));
const contract = new Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const context = () => runtime.createCircuitContext(runtime.dummyContractAddress(),
  initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
const bytes = new runtime.CompactTypeBytes(32);
const scalar = runtime.CompactTypeSecp256r1Scalar;
const point = runtime.CompactTypeSecp256r1Point;
const bool = runtime.CompactTypeBoolean;
const inputs = (message, sig, pk) => [...bytes.toValue(message), ...scalar.toValue(sig.r),
  ...scalar.toValue(sig.s), ...pointAtoms(pk)];

const cases = {};
const vectors = {};
await mkdir(output);
async function save(name, data) {
  await writeFile(join(output, name), data, { flag: 'wx' });
  return sha(data);
}
for (const { id, message, sig, pk, expected } of signatureCases(p256, keccak_256)) {
  const call = contract.circuits.verify(context(), message, sig, pk);
  const proof = call.proofData;
  assert.equal(call.result, expected, id);
  const state = call.context.currentQueryContext.state.state.asArray()[0].asCell();
  assert.deepEqual(state, { alignment: bool.alignment(), value: bool.toValue(expected) }, id);
  assert.deepEqual(proof.output, state, id);
  assert.deepEqual(proof.input.alignment, [
    ...bytes.alignment(), ...scalar.alignment(), ...scalar.alignment(), ...point.alignment(),
  ], id);
  assert.deepEqual(proof.input.value, inputs(message, sig, pk), id);
  assert.deepEqual(proof.privateTranscriptOutputs, [], id);
  const serialize = ({ input = proof.input, result = proof.output, transcript = proof.publicTranscript } = {}) =>
    proofDataIntoSerializedPreimage(input, result, transcript, [], circuit);
  const honest = serialize();
  await check(honest, materials);
  const selected = id !== 'wrong-message';
  const row = { fixture: 'upstream-ordinary', folder, circuit, selected, artifacts,
    preimage: await save(id + '.preimage', honest), refusals: {} };
  const refuse = async (kind, mutation, reason) => {
    const invalid = serialize(mutation);
    await assert.rejects(() => check(invalid, materials), reason, id);
    row.refusals[kind] = await save(id + '.' + kind, invalid);
  };
  const transcript = structuredClone(proof.publicTranscript);
  const cells = transcript.filter((op) => op.push?.storage === true && op.push.value?.tag === 'cell');
  assert.equal(cells.length, 1, id);
  assert.deepEqual(cells[0].push.value.content, state, id);
  cells[0].push.value.content.value = bool.toValue(!expected);
  await refuse('changed-write', { transcript }, /transcript input mismatch/i);
  await refuse('changed-return', { result: { ...proof.output, value: bool.toValue(!expected) } },
    /communications commitment mismatch/i);
  if (id === 'original') {
    const input = (signature, key) => ({ ...proof.input, value: inputs(message, signature, key) });
    await refuse('identity-key', { input: input(sig, identity) }, /Failed direct assertion/i);
    await refuse('zero-s', { input: input({ ...sig, s: 0n }, pk) }, /invert|inverse/i);
    await refuse('off-curve', { input: input(sig, { ...identity, identity: false }) },
      /Failed to decode .*Secp256r1Point|encoded value of type Secp256r1Point.*not in canonical form/i);
  }
  cases[id] = row;
  vectors[id] = { fixture: row.fixture, circuit, selected, expected, message: bytesHex(message),
    r: coordinateHex(sig.r), s: coordinateHex(sig.s), point: encodePoint(pk) };
}
assert.equal(Object.keys(cases).length, 4);
assert.equal(Object.values(cases).filter(({ selected }) => selected).length, 3);
assert.equal(Object.values(cases).reduce((n, { refusals }) => n + Object.keys(refusals).length, 0), 11);
await save('vectors.json', JSON.stringify(vectors, null, 2) + '\n');
await save('summary.json', JSON.stringify({ tools, cases }, null, 2) + '\n');
console.log('4 SDK ECDSA preimages admitted, 11 mutations refused');
