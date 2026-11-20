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

const [emitted, wasmPackage, output] = process.argv.slice(2).map((arg) => resolve(arg));
const manifest = JSON.parse(await readFile(join(wasmPackage, 'package.json')));
assert.equal(manifest.version, '3.2.0');
const wasmEntry = join(wasmPackage, manifest.exports.node);
const { check, jsonIrToBinary } = await import(pathToFileURL(wasmEntry));
const digestOf = (bytes) => createHash('sha256').update(bytes).digest('hex');
const pin = async (path) => ({ path, sha256: digestOf(await readFile(path)) });
const fromPath = (path) => import(pathToFileURL(path));
const entry = (family) => join(emitted, family, '.build/contract/index.js');
const runtimeEntry = createRequire(entry('verify_sequential')).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await fromPath(runtimeEntry);
const requireFromRuntime = createRequire(runtimeEntry);
const onchainEntry = requireFromRuntime.resolve('@midnightntwrk/onchain-runtime-v4');
const { proofDataIntoSerializedPreimage } = await fromPath(onchainEntry);
const nobleEntry = requireFromRuntime.resolve('@noble/curves/secp256k1.js');
const { secp256k1 } = await fromPath(nobleEntry);
const pins = await Promise.all([runtimeEntry, onchainEntry, nobleEntry, wasmEntry,
  join(wasmPackage, 'midnight_zkir_wasm_bg.wasm')].map(pin));

const pointType = runtime.CompactTypeSecp256k1Point;
const scalarType = runtime.CompactTypeSecp256k1Scalar;
const bytes20 = new runtime.CompactTypeBytes(20);
const uint64 = new runtime.CompactTypeUnsignedInteger((1n << 64n) - 1n, 8);
const digest = Uint8Array.from({ length: 32 }, (_, index) => index + 1);
const keyBytes = [5n, 7n, 11n].map((value) => secp256k1.Point.Fn.toBytes(value));
const points = keyBytes.map((key) => {
  const point = secp256k1.Point.fromBytes(secp256k1.getPublicKey(key));
  return { x: point.x, y: point.y, identity: false };
});
const signatures = keyBytes.map((key, index) => {
  const raw = secp256k1.sign(digest, key, { prehash: false, format: 'compact' });
  assert(secp256k1.verify(raw, digest, secp256k1.getPublicKey(key), { prehash: false }));
  assert(!secp256k1.verify(raw, digest, secp256k1.getPublicKey(keyBytes[(index + 1) % 3]), { prehash: false }));
  const { r, s } = secp256k1.Signature.fromBytes(raw, 'compact');
  return { r, s };
});
const publicAddress = (point) => runtime.keccak256(
  new runtime.CompactTypeBytes(64), point.toBytes(false).subarray(1),
).slice(12);
const address = publicAddress(secp256k1.Point.fromBytes(secp256k1.getPublicKey(keyBytes[0])));
const otherAddress = publicAddress(secp256k1.Point.fromBytes(secp256k1.getPublicKey(keyBytes[1])));
assert.notDeepEqual(address, otherAddress);

const sameAtoms = (a, b) => a.length === b.length && a.every((atom, i) => Buffer.from(atom).equals(Buffer.from(b[i])));
const signatureAtoms = ({ r, s }) => [...scalarType.toValue(r), ...scalarType.toValue(s)];
const cases = {};
await mkdir(output);

async function persist(name, value) {
  await writeFile(join(output, name), value, { flag: 'wx' });
  return digestOf(value);
}

async function checkedContract(family, name, witnesses = {}) {
  const source = entry(family);
  assert.equal(createRequire(source).resolve('@midnight-ntwrk/compact-runtime'), runtimeEntry);
  const module = await fromPath(source);
  const contract = new module.Contract(witnesses);
  const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
  const context = runtime.createCircuitContext(runtime.dummyContractAddress(),
    initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
  const build = resolve(dirname(source), '..');
  const raw = join(build, 'zkir', name + '.zkir');
  const binary = join(build, 'zkir', name + '.bzkir');
  const ir = jsonIrToBinary(await readFile(raw, 'utf8'));
  assert.deepEqual(Buffer.from(ir), await readFile(binary));
  const materials = {
    lookupKey: async () => ({ proverKey: new Uint8Array(), verifierKey: new Uint8Array(), ir }),
    getParams: async () => { throw new Error('unexpected parameters request'); },
  };
  const artifacts = await Promise.all([source, join(build, 'compiler/contract-info.json'), raw, binary].map(pin));
  return { contract, context, ledger: module.ledger, materials, artifacts };
}

function serialize(name, proof, { transcript = proof.publicTranscript,
  output = proof.output, privateOutputs = proof.privateTranscriptOutputs } = {}) {
  return proofDataIntoSerializedPreimage(proof.input, output, transcript, privateOutputs, name);
}

async function admit(name, proof, artifacts, materials) {
  const preimage = serialize(name, proof);
  await check(preimage, materials);
  const record = { artifacts, honest: await persist(name + '.preimage', preimage), refusals: {} };
  cases[name] = record;
  return async (kind, mutation, reason) => {
    const invalid = serialize(name, proof, mutation);
    await assert.rejects(() => check(invalid, materials), reason);
    record.refusals[kind] = await persist(name + '.' + kind, invalid);
  };
}

function changeWrite(proof, descriptor, expected, replacement) {
  const transcript = structuredClone(proof.publicTranscript);
  const writes = transcript.filter((op) => op.push?.storage === true && op.push.value?.tag === 'cell');
  assert.equal(writes.length, 1);
  const cell = writes[0].push.value.content;
  assert.deepEqual(cell.alignment, descriptor.alignment());
  assert(sameAtoms(cell.value, descriptor.toValue(expected)));
  cell.value = descriptor.toValue(replacement);
  return transcript;
}

{
  const name = 'storeEthereumAddress';
  const { contract, context, ledger, materials, artifacts } = await checkedContract('prove_ethereum_address', name);
  const call = contract.circuits[name](context, points[0]);
  assert.deepEqual(call.result, address);
  assert.deepEqual(ledger(call.context.currentQueryContext.state).lastAddr, address);
  const proof = call.proofData;
  assert.equal(proof.privateTranscriptOutputs.length, 0);
  const refuse = await admit(name, proof, artifacts, materials);
  await refuse('changed-write', { transcript: changeWrite(proof, bytes20, address, otherAddress) }, /transcript input mismatch/i);
  const output = structuredClone(proof.output);
  assert(sameAtoms(output.value, bytes20.toValue(address)));
  output.value = bytes20.toValue(otherAddress);
  await refuse('changed-return', { output }, /communications commitment mismatch/i);
}

for (const [name, count] of [['two', 2], ['three', 3]]) {
  const calls = [];
  let substitution;
  const witnessIndex = (kind, i) => substitution?.kind === kind && substitution.index === i ? (i + 1) % 3 : i;
  const witnesses = {
    sigOf: ({ privateState }, i) => {
      calls.push(['sig', Number(i)]);
      return [privateState, signatures[witnessIndex('signature', Number(i))]];
    },
    keyOf: ({ privateState }, i) => {
      calls.push(['key', Number(i)]);
      return [privateState, points[witnessIndex('key', Number(i))]];
    },
  };
  const { contract, context, ledger, materials, artifacts } = await checkedContract('verify_sequential', name, witnesses);
  const expectedOrder = Array.from({ length: count }, (_, i) => [['sig', i], ['key', i]]).flat();
  const call = contract.circuits[name](context, digest);
  assert.deepEqual(calls, expectedOrder);
  assert.deepEqual(call.result, []);
  assert.equal(ledger(call.context.currentQueryContext.state).n, 1n);
  const proof = call.proofData;
  assert.equal(proof.privateTranscriptOutputs.length, count * 2);
  const refuse = await admit(name, proof, artifacts, materials);
  await refuse('changed-write', { transcript: changeWrite(proof, uint64, 1n, 0n) }, /transcript input mismatch/i);
  for (let i = 0; i < count; i++) {
    const alternate = (i + 1) % 3;
    const outputs = proof.privateTranscriptOutputs;
    assert(sameAtoms(outputs[i * 2].value, signatureAtoms(signatures[i])));
    assert(sameAtoms(outputs[i * 2 + 1].value, pointType.toValue(points[i])));
    for (const [kind, slot, value] of [
      ['signature', i * 2, signatureAtoms(signatures[alternate])],
      ['key', i * 2 + 1, pointType.toValue(points[alternate])],
    ]) {
      const privateOutputs = structuredClone(outputs);
      privateOutputs[slot].value = value;
      await refuse(`changed-${kind}-${i}`, { privateOutputs }, /transcript input mismatch/i);
      calls.length = 0;
      substitution = { kind, index: i };
      const changed = contract.circuits[name](context, digest);
      assert.deepEqual(calls, expectedOrder);
      assert.equal(ledger(changed.context.currentQueryContext.state).n, 0n);
      await check(serialize(name, changed.proofData), materials);
    }
  }
}

const word = (value) => value.toString(16).padStart(64, '0');
const hex = (bytes) => Buffer.from(bytes).toString('hex');
const vectors = {
  digest: hex(digest), address: hex(address),
  points: points.map(({ x, y }) => ({ x: word(x), y: word(y) })),
  signatures: signatures.map(({ r, s }) => ({ r: word(r), s: word(s) })),
};
await persist('vectors.json', JSON.stringify(vectors, null, 2) + '\n');
await persist('summary.json', JSON.stringify({ pins, cases }, null, 2) + '\n');
console.log('Three honest preimages, ten zero-fold alternatives and fourteen statement or witness refusals');
