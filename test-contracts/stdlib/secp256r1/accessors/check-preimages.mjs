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
import { identity } from '../point/cases.mjs';
import { accessors } from './cases.mjs';

assert.equal(process.argv.length, 5, 'expected family output, SDK WASM package and new output directory');
const [family, wasmPackage, output] = process.argv.slice(2).map((arg) => resolve(arg));
const build = (folder) => join(family, folder, 'ts-rust/.build');
const runtimeEntry = createRequire(join(build(accessors[0].folder), 'contract/index.js'))
  .resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry));
const onchainEntry = createRequire(runtimeEntry).resolve('@midnightntwrk/onchain-runtime-v4');
const { proofDataIntoSerializedPreimage } = await import(pathToFileURL(onchainEntry));
const manifest = JSON.parse(await readFile(join(wasmPackage, 'package.json')));
assert.equal(manifest.version, '3.2.0');
const wasmEntry = join(wasmPackage, manifest.exports.node);
const { check, jsonIrToBinary } = await import(pathToFileURL(wasmEntry));

const base = runtime.CompactTypeSecp256r1Base;
const point = runtime.CompactTypeSecp256r1Point;
const bool = runtime.CompactTypeBoolean;
const descriptor = (value) => typeof value === 'boolean' ? bool : typeof value === 'bigint' ? base : point;
const encode = (value) => {
  if (typeof value === 'bigint') return value.toString(16).padStart(64, '0');
  if (Array.isArray(value)) return value.map(encode);
  if (typeof value === 'object') return { x: encode(value.x), y: encode(value.y), identity: value.identity };
  return value;
};
const sha = (data) => createHash('sha256').update(data).digest('hex');
const pin = async (path) => ({ path, sha256: sha(await readFile(path)) });
const tools = await Promise.all([runtimeEntry, onchainEntry, wasmEntry,
  join(wasmPackage, 'midnight_zkir_wasm_bg.wasm')].map(pin));
const cases = {};
const vectors = {};
const counts = { honest: 0, changedWrite: 0, changedReturn: 0, activeIdentity: 0, selected: 0 };
await mkdir(output);
async function save(name, data) {
  await writeFile(join(output, name), data, { flag: 'wx' });
  return sha(data);
}

for (const { fixture, folder, calls } of accessors) {
  const dir = build(folder);
  const source = join(dir, 'contract/index.js');
  assert.equal(createRequire(source).resolve('@midnight-ntwrk/compact-runtime'), runtimeEntry);
  const infoPath = join(dir, 'compiler/contract-info.json');
  const info = JSON.parse(await readFile(infoPath));
  const { Contract, ledger } = await import(pathToFileURL(source));
  const contract = new Contract({});
  const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
  const context = () => runtime.createCircuitContext(runtime.dummyContractAddress(),
    initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
  const programs = new Map();
  for (const { name, proof } of info.circuits) {
    assert.equal(proof, true, name);
    const raw = join(dir, 'zkir', name + '.zkir');
    const binary = raw.replace(/\.zkir$/, '.bzkir');
    const text = await readFile(raw, 'utf8');
    const zkir = JSON.parse(text);
    assert(zkir.do_communications_commitment, name);
    if (name.startsWith('guarded')) {
      assert.deepEqual(zkir.inputs.map(({ type }) => type), ['Scalar<BLS12-381>', 'Point<Secp256r1>']);
      assert.deepEqual(zkir.outputs, ['Base<Secp256r1>']);
      const ops = zkir.instructions;
      const extraction = ops.find(({ op }) => op === 'into_coordinates');
      const active = ops.find(({ op, output }) => op === 'cond_select' && output === extraction.point);
      assert.equal(active.bit, zkir.inputs[0].name, name);
      assert.equal(active.a, zkir.inputs[1].name, name);
      const fallback = ops.find(({ op, output }) => op === 'from_coordinates' && output === active.b);
      assert.equal(fallback.inputs.length, 2, name);
      for (const input of fallback.inputs) {
        const coordinate = ops.find(({ output }) => output === input);
        assert.equal(coordinate.op, 'from_bytes', name);
        assert.equal(coordinate.type, 'Base<Secp256r1>', name);
      }
      assert(!ops.some(({ type }) => type?.includes('Secp256k1')), name);
    }
    const ir = jsonIrToBinary(text);
    assert.deepEqual(Buffer.from(ir), await readFile(binary));
    programs.set(name, {
      artifacts: await Promise.all([raw, binary, infoPath, source, join(dir, 'contract/lib.rs')].map(pin)),
      materials: {
        lookupKey: async () => ({ proverKey: new Uint8Array(), verifierKey: new Uint8Array(), ir }),
        getParams: async () => { throw new Error('unexpected parameter request'); },
      },
    });
  }

  const indices = new Map();
  for (const { circuit, args, expected, seed, slot, written, selected = false } of calls) {
    const index = indices.get(circuit) ?? 0;
    indices.set(circuit, index + 1);
    const id = fixture + '-' + circuit + '-' + index;
    const seedCall = fixture === 'upstream-roundtrip' ? 'test0' : 'storePoint';
    const ctx = seed ? contract.circuits[seedCall](context(), seed).context : context();
    const call = contract.circuits[circuit](ctx, ...args);
    assert.deepEqual(call.result, expected, id);
    const proof = call.proofData;
    assert.deepEqual(proof.privateTranscriptOutputs, [], id);
    const outputs = Array.isArray(expected) ? expected : [expected];
    const alignment = (values) => values.flatMap((value) => descriptor(value).alignment());
    const atoms = (values) => values.flatMap((value) => descriptor(value).toValue(value));
    assert.deepEqual(proof.input.alignment, alignment(args), id);
    assert.deepEqual(proof.input.value, atoms(args), id);
    assert.deepEqual(proof.output.alignment, alignment(outputs), id);
    assert.deepEqual(proof.output.value, atoms(outputs), id);
    const { artifacts, materials } = programs.get(circuit);
    const serialize = (transcript = proof.publicTranscript, output = proof.output, input = proof.input) =>
      proofDataIntoSerializedPreimage(input, output, transcript, proof.privateTranscriptOutputs, circuit);
    const honest = serialize();
    await check(honest, materials);
    const record = { fixture, folder, circuit, selected, artifacts, preimage: await save(id + '.preimage', honest) };
    counts.honest++;
    if (selected) counts.selected++;
    if (slot) {
      const value = written ?? (circuit === 'storePoint' ? (args[0].identity ? identity : args[0]) : expected);
      assert.deepEqual(ledger(call.context.currentQueryContext.state)[slot], value, id);
      const transcript = structuredClone(proof.publicTranscript);
      const cells = transcript.filter((op) => op.push?.storage === true && op.push.value?.tag === 'cell');
      assert.equal(cells.length, 1, id);
      const cell = cells[0].push.value.content;
      assert.deepEqual(cell.alignment, descriptor(value).alignment(), id);
      assert.deepEqual(cell.value, atoms([value]), id);
      const wrong = typeof value === 'bigint' ? value === 0n ? 1n : 0n
        : value.identity ? accessors[0].calls[0].args[0] : identity;
      cell.value = atoms([wrong]);
      const changed = serialize(transcript);
      await assert.rejects(() => check(changed, materials), /transcript input mismatch/i, id);
      record.changedWrite = await save(id + '.changed-write', changed);
      counts.changedWrite++;
    }
    if (outputs.length > 0) {
      const changed = [...outputs];
      const value = changed[0];
      changed[0] = typeof value === 'bigint' ? value === 0n ? 1n : 0n : identity;
      const invalid = serialize(proof.publicTranscript, { ...proof.output, value: atoms(changed) });
      await assert.rejects(() => check(invalid, materials), /communications commitment mismatch/i, id);
      record.changedReturn = await save(id + '.changed-return', invalid);
      counts.changedReturn++;
    }
    if (seed || (circuit.startsWith('guarded') && args[0] === true)) {
      let invalid;
      if (seed) {
        const transcript = structuredClone(proof.publicTranscript);
        const reads = transcript.filter((op) => op.popeq?.result
          && JSON.stringify(op.popeq.result.alignment) === JSON.stringify(point.alignment()));
        assert(reads.length > 0, id);
        for (const read of reads) {
          assert.equal(point.fromValue(read.popeq.result.value.slice()).identity, false);
          read.popeq.result.value = point.toValue(identity);
        }
        invalid = serialize(transcript);
      } else {
        const input = structuredClone(proof.input);
        input.value.splice(-5, 5, ...point.toValue(identity));
        invalid = serialize(proof.publicTranscript, proof.output, input);
      }
      await assert.rejects(() => check(invalid, materials),
        circuit === 'storeXChecked' ? { message: 'Failed direct assertion' } : /cannot extract coordinates of the Secp256r1 identity/i, id);
      record.activeIdentity = await save(id + '.active-identity', invalid);
      counts.activeIdentity++;
    }
    cases[id] = record;
    vectors[id] = { fixture, folder, circuit, selected, inputs: args.map(encode), expected: encode(expected) };
    if (seed) vectors[id].seed = encode(seed);
    if (written !== undefined) vectors[id].written = encode(written);
  }
}
assert.deepEqual(counts, { honest: 18, changedWrite: 16, changedReturn: 12, activeIdentity: 9, selected: 11 });
await save('vectors.json', JSON.stringify(vectors, null, 2) + '\n');
await save('summary.json', JSON.stringify({ tools, cases, counts }, null, 2) + '\n');
console.log(JSON.stringify(counts));
