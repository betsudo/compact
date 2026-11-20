// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
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

/** Capture one fresh Runtime 0.18.0 Set insertion proof preimage from generated TypeScript. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const [entryArg, rawArg, wasmArg, outputArg] = process.argv.slice(2);
assert.ok(
  entryArg && rawArg && wasmArg && outputArg,
  "usage: capture-ledger9-set-add-proof-v018.mjs <generated-index.js> <add.zkir> <zkir-wasm-package> <new-output-dir>",
);
const entry = resolve(entryArg);
const rawPath = resolve(rawArg);
const wasmPackage = resolve(wasmArg);
const output = resolve(outputArg);
const hash = (value) => createHash("sha256").update(value).digest("hex");
const wasmManifest = JSON.parse(
  await readFile(join(wasmPackage, "package.json")),
);
assert.equal(wasmManifest.version, "3.2.0");
const { check, jsonIrToBinary } = await import(
  pathToFileURL(join(wasmPackage, wasmManifest.exports.node))
);
const runtimeEntry = createRequire(entry).resolve(
  "@midnight-ntwrk/compact-runtime",
);
const runtime = await import(pathToFileURL(runtimeEntry));
assert.equal(
  JSON.parse(
    await readFile(
      createRequire(entry).resolve(
        "@midnight-ntwrk/compact-runtime/package.json",
      ),
    ),
  ).version,
  "0.18.0",
);
const onchainEntry = createRequire(runtimeEntry).resolve(
  "@midnightntwrk/onchain-runtime-v4",
);
const { proofDataIntoSerializedPreimage } = await import(
  pathToFileURL(onchainEntry)
);
assert.equal(
  JSON.parse(await readFile(resolve(dirname(onchainEntry), "package.json")))
    .version,
  "4.0.1",
);
const code = await import(pathToFileURL(entry));
const contract = new code.Contract({});
const coinPublicKey = { bytes: new Uint8Array(32) };
const initial = contract.initialState({
  initialPrivateState: null,
  initialZswapLocalState: runtime.emptyZswapLocalState(coinPublicKey),
});
const context = runtime.createCircuitContext(
  runtime.dummyContractAddress(),
  coinPublicKey,
  initial.currentContractState.data,
  null,
);
const execution = contract.circuits.add(context, true);
assert.deepEqual(execution.result, []);
const proof = execution.proofData;
assert.deepEqual(proof.privateTranscriptOutputs, []);
const raw = await readFile(rawPath, "utf8");
const ir = jsonIrToBinary(raw);
const materials = {
  lookupKey: async () => ({
    proverKey: new Uint8Array(),
    verifierKey: new Uint8Array(),
    ir,
  }),
  getParams: async () => {
    throw Error("unexpected parameter request");
  },
};
const serialize = (transcript) =>
  proofDataIntoSerializedPreimage(
    proof.input,
    proof.output,
    transcript,
    proof.privateTranscriptOutputs,
    "add",
  );
const preimage = serialize(proof.publicTranscript);
await check(preimage, materials);
const changed = structuredClone(proof.publicTranscript);
const insert = changed.find((operation) => operation.ins?.cached === false);
assert(insert && insert.ins.n === 1, "Set.add must insert one element");
insert.ins.n = 2;
const altered = serialize(changed);
await assert.rejects(
  () => check(altered, materials),
  /public transcript input mismatch/i,
);

await mkdir(output);
await writeFile(join(output, "honest.preimage"), preimage, { flag: "wx" });
await writeFile(join(output, "changed-insert.preimage"), altered, {
  flag: "wx",
});
const receipt = {
  circuit: "add",
  argument: true,
  generated_index_sha256: hash(await readFile(entry)),
  raw_zkir_sha256: hash(raw),
  binary_zkir_sha256: hash(ir),
  preimage_sha256: hash(preimage),
  changed_insert_sha256: hash(altered),
  runtime_version: "0.18.0",
  onchain_version: "4.0.1",
  zkir_version: "3.2.0",
  honest_check: "passed",
  changed_insert_check: "refused",
  transcript_operations: proof.publicTranscript.map((operation) =>
    Object.keys(operation),
  ),
};
await writeFile(
  join(output, "capture.json"),
  JSON.stringify(receipt, null, 2) + "\n",
  { flag: "wx" },
);
console.log("Ledger 9 Set.add preimage passed independent ZKIR wasm check");
