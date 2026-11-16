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

import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, symlinkSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

if (process.argv.length !== 5) {
  throw Error(
    "Usage: node check.mjs <compiler> <built-ts-runtime-dir> <new-output-dir>",
  );
}
const [compiler, runtime, output] = process.argv
  .slice(2)
  .map((path) => resolve(path));
const root = fileURLToPath(new URL(".", import.meta.url));
const repo = resolve(root, "../../..");
const manifest = JSON.parse(
  readFileSync(join(runtime, "package.json"), "utf8"),
);
if (manifest.name !== "@midnight-ntwrk/compact-runtime" || !manifest.main) {
  throw Error("Expected the built Compact TypeScript runtime package");
}
mkdirSync(output);

const cases = [
  ["Adder", join(root, "Adder-compatible.compact")],
  ["basic", join(root, "circuit-signatures-basic.compact")],
  ["composites", join(root, "circuit-signatures-composites.compact")],
  [
    "p256",
    join(
      repo,
      "test-contracts/stdlib/secp256r1/point/upstream-tags/upstream-tags.compact",
    ),
  ],
  ["curves", join(root, "circuit-signatures-curves.compact")],
  ["sequences", join(root, "circuit-signatures-sequences.compact")],
  ["opaque", join(root, "circuit-signatures-opaque.compact")],
  ["opaque-point", join(root, "circuit-signatures-opaque-point.compact")],
  ["proto", join(root, "circuit-signatures-proto.compact")],
  [
    "contract-argument",
    join(root, "circuit-signatures-contract-argument.compact"),
  ],
];
const builds = [];
for (const [name, source] of cases) {
  const build = join(output, name);
  const result = spawnSync(
    compiler,
    [
      "--skip-zk",
      "--compact-path",
      join(repo, "compiler"),
      "--compact-path",
      root,
      source,
      build,
    ],
    { cwd: repo, encoding: "utf8" },
  );
  if (result.error) throw result.error;
  writeFileSync(join(output, `${name}.stdout.log`), result.stdout);
  writeFileSync(join(output, `${name}.stderr.log`), result.stderr);
  if (result.status !== 0) {
    throw Error(
      `${name} compilation failed (${result.status}): ${result.stderr}`,
    );
  }
  const link = join(
    build,
    "contract/node_modules/@midnight-ntwrk/compact-runtime",
  );
  mkdirSync(dirname(link), { recursive: true });
  symlinkSync(runtime, link, "dir");
  if (name !== "Adder") builds.push(build);
}
const result = spawnSync(
  process.execPath,
  [join(root, "check-signatures.mjs"), ...builds],
  {
    cwd: repo,
    encoding: "utf8",
  },
);
if (result.error) throw result.error;
writeFileSync(join(output, "assertions.stdout.log"), result.stdout);
writeFileSync(join(output, "assertions.stderr.log"), result.stderr);
if (result.status !== 0)
  throw Error(result.stderr || `metadata assertions failed (${result.status})`);
process.stdout.write(result.stdout);

const contractSource = join(root, "circuit-signatures-contract.compact");
const contractProbe = spawnSync(
  compiler,
  [
    "--skip-zk",
    "--compact-path",
    join(repo, "compiler"),
    "--compact-path",
    root,
    contractSource,
    join(output, "contract"),
  ],
  { cwd: repo, encoding: "utf8" },
);
if (contractProbe.error) throw contractProbe.error;
writeFileSync(join(output, "contract.stderr.log"), contractProbe.stderr);
if (
  contractProbe.status === 0 ||
  !/cross-contract calls are not yet supported/.test(contractProbe.stderr)
) {
  throw Error(
    `unexpected cross-contract call result (${contractProbe.status}): ${contractProbe.stderr}`,
  );
}

const collisionProbe = spawnSync(
  compiler,
  [
    "--skip-zk",
    "--compact-path",
    join(repo, "compiler"),
    join(root, "circuit-signatures-enum-collision.compact"),
    join(output, "enum-collision"),
  ],
  { cwd: repo, encoding: "utf8" },
);
if (collisionProbe.error) throw collisionProbe.error;
writeFileSync(join(output, "enum-collision.stderr.log"), collisionProbe.stderr);
if (
  collisionProbe.status === 0 ||
  !/exported enum name circuitSignatures conflicts with generated circuit metadata export/.test(
    collisionProbe.stderr,
  )
) {
  throw Error(
    `unexpected enum collision result (${collisionProbe.status}): ${collisionProbe.stderr}`,
  );
}

const interfaceCollisionProbe = spawnSync(
  compiler,
  [
    "--skip-zk",
    "--compact-path",
    join(repo, "compiler"),
    join(root, "declared-interfaces-enum-collision.compact"),
    join(output, "interface-enum-collision"),
  ],
  { cwd: repo, encoding: "utf8" },
);
if (interfaceCollisionProbe.error) throw interfaceCollisionProbe.error;
writeFileSync(
  join(output, "interface-enum-collision.stderr.log"),
  interfaceCollisionProbe.stderr,
);
if (
  interfaceCollisionProbe.status === 0 ||
  !/exported enum name declaredInterfaces conflicts with generated interface metadata export/.test(
    interfaceCollisionProbe.stderr,
  )
) {
  throw Error(
    `unexpected interface enum collision result (${interfaceCollisionProbe.status}): ${interfaceCollisionProbe.stderr}`,
  );
}

// Preserve the original upstream Adder fixture and record the current
// compiler's stricter declaration check against it.
const original = join(output, "original");
mkdirSync(original);
for (const [name, source, shouldPass] of [
  ["Adder", join(root, "Adder.compact"), true],
  ["contract", join(root, "circuit-signatures-contract.compact"), false],
]) {
  const probe = spawnSync(
    compiler,
    [
      "--skip-zk",
      "--compact-path",
      join(repo, "compiler"),
      "--compact-path",
      root,
      source,
      join(original, name),
    ],
    { cwd: repo, encoding: "utf8" },
  );
  if (probe.error) throw probe.error;
  writeFileSync(join(original, `${name}.stderr.log`), probe.stderr);
  if (
    shouldPass
      ? probe.status !== 0
      : probe.status === 0 ||
        !/contract declaration has a circuit named peek/.test(probe.stderr)
  ) {
    throw Error(
      `unexpected original ${name} result (${probe.status}): ${probe.stderr}`,
    );
  }
}
