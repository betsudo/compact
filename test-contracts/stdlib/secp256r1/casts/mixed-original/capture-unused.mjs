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
const contract = new code.Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const context = () => runtime.createCircuitContext(runtime.dummyContractAddress(),
  initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
const bytes = new Uint8Array(32).fill(7);
const wide = new Uint8Array(64).fill(9);
const cases = [
  { circuit: 'unusedK1', args: [bytes], expected: 1n },
  { circuit: 'unusedR1', args: [bytes], expected: 2n },
  { circuit: 'unusedC25519', args: [bytes, wide], expected: 3n },
  { circuit: 'unusedFieldK1', args: [bytes, 4n], expected: 4n },
  { circuit: 'unusedFieldR1', args: [bytes, 5n], expected: 5n },
];
const sha = (data) => createHash('sha256').update(data).digest('hex');

await mkdir(output);
const receipts = {};
for (const spec of cases) {
  const rawPath = resolve(dirname(entry), `../zkir/${spec.circuit}.zkir`);
  const raw = await readFile(rawPath, 'utf8');
  const ir = jsonIrToBinary(raw);
  assert.deepEqual(Buffer.from(ir), await readFile(rawPath.replace(/\.zkir$/, '.bzkir')));
  const materials = {
    lookupKey: async () => ({ proverKey: new Uint8Array(), verifierKey: new Uint8Array(), ir }),
    getParams: async () => { throw Error('unexpected parameter request'); },
  };
  const execution = contract.circuits[spec.circuit](context(), ...spec.args);
  assert.deepEqual(execution.result, []);
  assert.equal(code.ledger(execution.context.currentQueryContext.state).n, spec.expected);
  const proof = execution.proofData;
  assert.deepEqual(proof.privateTranscriptOutputs, []);
  const serialize = (transcript) => proofDataIntoSerializedPreimage(
    proof.input, proof.output, transcript, proof.privateTranscriptOutputs, spec.circuit);
  const preimage = serialize(proof.publicTranscript);
  await check(preimage, materials);
  const changed = structuredClone(proof.publicTranscript);
  const cell = changed.find((operation) => operation.push?.storage === true && operation.push.value?.tag === 'cell');
  assert(cell, `${spec.circuit} must write a cell`);
  assert(cell.push.value.content.value[0] instanceof Uint8Array);
  cell.push.value.content.value[0][0] += 1;
  const altered = serialize(changed);
  await assert.rejects(() => check(altered, materials), /transcript input mismatch/i);
  const dir = join(output, spec.circuit);
  await mkdir(dir);
  await writeFile(join(dir, 'honest.preimage'), preimage, { flag: 'wx' });
  await writeFile(join(dir, 'changed-write.preimage'), altered, { flag: 'wx' });
  receipts[spec.circuit] = {
    circuit: spec.circuit,
    args: spec.args.map((arg) => arg instanceof Uint8Array ? Buffer.from(arg).toString('hex') : String(arg)),
    expected_ledger: String(spec.expected), raw: rawPath,
    raw_sha256: sha(raw), binary_sha256: sha(ir),
    preimage_sha256: sha(preimage), changed_write_sha256: sha(altered),
    honest_check: 'passed', changed_write_check: 'refused',
  };
}
await writeFile(join(output, 'capture.json'), JSON.stringify(receipts, null, 2) + '\n', { flag: 'wx' });
console.log('Original mixed unused casts: five honest preimages and changed-write refusals passed');
