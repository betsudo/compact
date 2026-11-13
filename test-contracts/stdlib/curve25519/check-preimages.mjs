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

import { spawnSync } from 'node:child_process';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const [family, defaults, wasm, output] = process.argv.slice(2).map((arg) => resolve(arg));
const baseEntry = join(family, 'base/roundtrip/ts-rust/.build/contract/index.js');
const requireFromBase = createRequire(baseEntry);
const runtimeEntry = requireFromBase.resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry));
const ocrt = await import(pathToFileURL(createRequire(runtimeEntry).resolve('@midnightntwrk/onchain-runtime-v4')));
await mkdir(output);
const manifest = JSON.parse(await readFile(join(wasm, 'package.json'), 'utf8'));
assert.equal(manifest.version, '3.2.0');
const { check, jsonIrToBinary } = await import(pathToFileURL(join(wasm, manifest.exports.node)));
const decimal = value => JSON.stringify(value, (_, entry) => typeof entry === 'bigint' ? entry.toString() : entry);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const cases = {};
const make = (Contract) => {
  const contract = new Contract({});
  const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
  const ctx = runtime.createCircuitContext(runtime.dummyContractAddress(),
    initial.currentZswapLocalState, initial.currentContractState, initial.currentPrivateState);
  return {contract, ctx};
};
for (const [kind, width, max, codec] of [
  ['base',192n,runtime.MAX_CURVE25519_BASE,runtime.CompactTypeCurve25519Base],
  ['scalar',204n,runtime.MAX_CURVE25519_SCALAR,runtime.CompactTypeCurve25519Scalar],
]) {
  const source = pathToFileURL(join(family, `${kind}/roundtrip/ts-rust/.build`) + '/');
  const name = `curve25519_${kind}_roundtrip`;
  const { Contract, ledger } = await import(new URL('contract/index.js',source));
  const { contract, ctx } = make(Contract);
  const raw = await readFile(new URL(`zkir/${name}.zkir`,source), 'utf8');
  const ir = jsonIrToBinary(raw);
  const resolver = {
    lookupKey: async () => ({proverKey:new Uint8Array(),verifierKey:new Uint8Array(),ir}),
    getParams: async () => { throw new Error('unexpected key parameter request'); },
  };
  assert.deepEqual(Buffer.from(ir),await readFile(new URL(`zkir/${name}.bzkir`,source)));
  const values = [0n, 1n, 12345678901234567890n, 1n << width, (1n << width)+1n, max];
  for (const [index, value] of values.entries()) {
    const call = contract.circuits[name](ctx,value);
    assert.equal(call.result,value);
    assert.equal(ledger(call.context.currentQueryContext.state).stored,value);
    const p = call.proofData;
    assert.deepEqual(p.input.value, codec.toValue(value));
    assert.deepEqual(p.output.value, codec.toValue(value));
    const serialize = (input, output, transcript = p.publicTranscript) => ocrt.proofDataIntoSerializedPreimage(
      input,output,transcript,p.privateTranscriptOutputs,name,
    );
    const data = serialize(p.input,p.output);
    await check(data,resolver);
    const alternate = value === max ? 0n : value+1n;
    const changed = {...p.input,value:codec.toValue(alternate)};
    await assert.rejects(check(serialize(changed,p.output),resolver));
    const changedOutput = serialize(p.input,changed);
    await assert.rejects(check(changedOutput,resolver), /communications commitment mismatch/i);
    const transcript = structuredClone(p.publicTranscript);
    const writes = transcript.filter(op => op.push?.value?.tag === 'cell'
      && decimal(op.push.value.content.value) === decimal(p.input.value));
    assert.equal(writes.length, 1);
    writes[0].push.value.content.value = changed.value;
    const changedWrite = serialize(p.input,p.output,transcript);
    await assert.rejects(check(changedWrite,resolver), /transcript input mismatch/i);
    const id = `${name}-${index}`;
    await writeFile(join(output, `${id}.preimage`),data,{flag:'wx'});
    await writeFile(join(output, `${id}.changed-output`),changedOutput,{flag:'wx'});
    await writeFile(join(output, `${id}.changed-write`),changedWrite,{flag:'wx'});
    cases[id] = {
      rawIr: sha(raw), binaryIr: sha(ir), preimage: sha(data),
      changedOutput: sha(changedOutput), changedWrite: sha(changedWrite),
    };
    console.log(decimal({name,index,value,bytes:data.length}));
  }
}
const { Contract, ledger } = await import(pathToFileURL(join(defaults, 'contract/index.js')));
const {contract,ctx} = make(Contract);
const stored = contract.circuits.store(ctx,5n,7n);
const reset = contract.circuits.reset(stored.context);
assert.deepEqual(reset.result,[0n,0n]);
assert.equal(ledger(reset.context.currentQueryContext.state).base,0n);
assert.equal(ledger(reset.context.currentQueryContext.state).scalar,0n);
await writeFile(join(output, 'summary.json'), JSON.stringify(cases,null,2) + '\n',{flag:'wx'});
console.log('Both field defaults round-trip through the ledger');
