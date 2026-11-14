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

// Keep the 0.17.0 capture and its exact historical bindings intact. This
// runner creates a separate capture from the current 0.18.0 source pair.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const [compilerArg, runtimeArg, outputArg] = process.argv.slice(2);
assert.ok(
  compilerArg && runtimeArg && outputArg,
  "usage: node capture-ledger9-paired-counter-v018.mjs <compactc> <built-runtime-dir> <new-output-dir>",
);
const compiler = resolve(compilerArg);
const runtimeDir = resolve(runtimeArg);
const output = resolve(outputArg);
const repo = resolve(fileURLToPath(new URL("../../..", import.meta.url)));
const source = join(repo, "examples/rust_backend/counter.compact");
const oraclePath = fileURLToPath(
  new URL("ledger9-counter-oracle.json", import.meta.url),
);
const runtimeManifest = JSON.parse(
  readFileSync(join(runtimeDir, "package.json"), "utf8"),
);
assert.equal(runtimeManifest.name, "@midnight-ntwrk/compact-runtime");
assert.equal(runtimeManifest.version, "0.18.0");
const hash = (path) =>
  createHash("sha256").update(readFileSync(path)).digest("hex");
const hashDirectory = (directory) => {
  const digest = createHash("sha256");
  const walk = (base, relative) => {
    for (const entry of readdirSync(base, { withFileTypes: true }).sort(
      (a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0),
    )) {
      const path = join(base, entry.name);
      const name = relative ? `${relative}/${entry.name}` : entry.name;
      if (entry.isDirectory()) {
        walk(path, name);
      } else {
        digest.update(name).update("\0").update(hash(path)).update("\n");
      }
    }
  };
  walk(directory, "");
  return digest.digest("hex");
};
const sourceSha = hash(source);
const compilerSha = hash(compiler);
const runtimePackageSha = hash(join(runtimeDir, "package.json"));
const runtimeDistPath = join(runtimeDir, "dist");
const runtimeDistSha = hashDirectory(runtimeDistPath);
const runtimeBuiltinsPath = join(runtimeDir, "dist/built-ins.js");
const runtimeBuiltinsSha = hash(runtimeBuiltinsPath);
const version = spawnSync(compiler, ["--version"], {
  cwd: repo,
  encoding: "utf8",
});
if (version.error) throw version.error;
assert.equal(version.status, 0, version.stderr);
assert.equal(version.stdout.trim(), "0.31.213");

// A fresh child is required so a failed compile cannot be mistaken for an
// earlier accepted capture. Only write capture.json after every check passes.
mkdirSync(output);
const generated = join(output, "generated");
const compile = spawnSync(
  compiler,
  ["--skip-zk", "--compact-path", join(repo, "compiler"), source, generated],
  { cwd: repo, encoding: "utf8" },
);
if (compile.error) throw compile.error;
writeFileSync(join(output, "compiler.stdout.log"), compile.stdout);
writeFileSync(join(output, "compiler.stderr.log"), compile.stderr);
assert.equal(compile.status, 0, compile.stderr);

const infoPath = join(generated, "compiler/contract-info.json");
const manifestPath = join(generated, "compiler/contract-manifest.json");
const entryPath = join(generated, "contract/index.js");
const info = JSON.parse(readFileSync(infoPath, "utf8"));
const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
for (const metadata of [info, manifest]) {
  assert.equal(metadata["compiler-version"], "0.31.213");
  assert.equal(metadata["language-version"], "0.23.105");
  assert.equal(metadata["runtime-version"], "0.18.0");
}
assert.equal(manifest.contract["index.js"].hash, hash(entryPath));
assert.equal(
  manifest.contract["index.d.ts"].hash,
  hash(join(generated, "contract/index.d.ts")),
);
assert.equal(
  manifest.contract["index.js.map"].hash,
  hash(join(generated, "contract/index.js.map")),
);
assert.equal(manifest.compiler["contract-info.json"].hash, hash(infoPath));
const generatedInfoSha = hash(infoPath);
const generatedManifestSha = hash(manifestPath);
const generatedJsSha = hash(entryPath);
const generatedDtsPath = join(generated, "contract/index.d.ts");
const generatedDtsSha = hash(generatedDtsPath);
const generatedMapPath = join(generated, "contract/index.js.map");
const generatedMapSha = hash(generatedMapPath);

const link = join(
  generated,
  "contract/node_modules/@midnight-ntwrk/compact-runtime",
);
mkdirSync(dirname(link), { recursive: true });
symlinkSync(runtimeDir, link, "dir");
const entry = pathToFileURL(entryPath);
const fromContract = createRequire(entry);
const resolvedManifest = fromContract.resolve(
  "@midnight-ntwrk/compact-runtime/package.json",
);
assert.equal(
  realpathSync(resolvedManifest),
  realpathSync(join(runtimeDir, "package.json")),
);
const runtimeEntry = fromContract.resolve("@midnight-ntwrk/compact-runtime");
const runtimeIndexSha = hash(runtimeEntry);
const runtime = await import(pathToFileURL(runtimeEntry));
const fromRuntime = createRequire(runtimeEntry);
const onchainEntry = fromRuntime.resolve("@midnightntwrk/onchain-runtime-v4");
const onchainManifestPath = resolve(dirname(onchainEntry), "package.json");
const onchainManifestSha = hash(onchainManifestPath);
const onchainIndexSha = hash(onchainEntry);
const onchainPackagePath = dirname(onchainEntry);
const onchainPackageSha = hashDirectory(onchainPackagePath);
assert.equal(
  JSON.parse(readFileSync(onchainManifestPath, "utf8")).version,
  "4.0.1",
);
const onchain = await import(pathToFileURL(onchainEntry));
assert.equal(runtime.ContractState, onchain.ContractState);
assert.equal(runtime.QueryContext, onchain.QueryContext);

const { Contract, ledger } = await import(entry);
const contract = new Contract({});
const coinPublicKey = { bytes: new Uint8Array(32) };
const initial = contract.initialState({
  initialPrivateState: null,
  initialZswapLocalState: runtime.emptyZswapLocalState(coinPublicKey),
});
const state = initial.currentContractState;
assert.ok(state instanceof onchain.ContractState);
const serialize = () => Buffer.from(state.serialize()).toString("hex");
const before = serialize();
const context = runtime.createCircuitContext(
  runtime.dummyContractAddress(),
  coinPublicKey,
  state.data,
  null,
);
const outcome = contract.circuits.increment(context);
assert.ok(outcome.context.currentQueryContext instanceof onchain.QueryContext);
state.data = new runtime.ChargedState(
  outcome.context.currentQueryContext.state.state,
);
const value = ledger(state.data).round;
assert.equal(value, 1n);
assert.deepEqual(outcome.result, []);

const observation = {
  before,
  after: serialize(),
  value: value.toString(),
  result: outcome.result,
};
assert.deepEqual(observation, JSON.parse(readFileSync(oraclePath, "utf8")));
const gas = Object.fromEntries(
  Object.entries(outcome.gasCost).map(([field, cost]) => [
    field,
    cost.toString(),
  ]),
);
for (const [path, digest] of [
  [source, sourceSha],
  [compiler, compilerSha],
  [resolvedManifest, runtimePackageSha],
  [runtimeEntry, runtimeIndexSha],
  [runtimeBuiltinsPath, runtimeBuiltinsSha],
  [onchainManifestPath, onchainManifestSha],
  [onchainEntry, onchainIndexSha],
  [infoPath, generatedInfoSha],
  [manifestPath, generatedManifestSha],
  [entryPath, generatedJsSha],
  [generatedDtsPath, generatedDtsSha],
  [generatedMapPath, generatedMapSha],
]) {
  assert.equal(
    hash(path),
    digest,
    `capture input changed during execution: ${path}`,
  );
}
assert.equal(
  hashDirectory(runtimeDistPath),
  runtimeDistSha,
  "runtime distribution changed during execution",
);
assert.equal(
  hashDirectory(onchainPackagePath),
  onchainPackageSha,
  "Onchain Runtime package changed during execution",
);
const capture = {
  format: "compact-ledger9-paired-counter-capture/v2",
  versions: {
    compiler: "0.31.213",
    language: "0.23.105",
    runtime: "0.18.0",
    onchain: "4.0.1",
  },
  provenance: {
    source: {
      path: "examples/rust_backend/counter.compact",
      sha256: sourceSha,
    },
    runnerSha256: hash(fileURLToPath(import.meta.url)),
    compiler: {
      path: compiler,
      sha256: compilerSha,
      reportedVersion: version.stdout.trim(),
    },
    generated: {
      contractInfoSha256: generatedInfoSha,
      manifestSha256: generatedManifestSha,
      indexJsSha256: generatedJsSha,
      indexDtsSha256: generatedDtsSha,
      indexJsMapSha256: generatedMapSha,
    },
    runtime: {
      packageSha256: runtimePackageSha,
      distSha256: runtimeDistSha,
      indexJsSha256: runtimeIndexSha,
      builtinsJsSha256: runtimeBuiltinsSha,
    },
    onchain: {
      packageSha256: onchainManifestSha,
      distributionSha256: onchainPackageSha,
      indexJsSha256: onchainIndexSha,
    },
    oracleSha256: hash(oraclePath),
  },
  observation: { ...observation, gas },
};
writeFileSync(
  join(output, "capture.json"),
  `${JSON.stringify(capture, null, 2)}\n`,
  { flag: "wx" },
);
console.log(
  JSON.stringify(
    { output: join(output, "capture.json"), observation: capture.observation },
    null,
    2,
  ),
);
