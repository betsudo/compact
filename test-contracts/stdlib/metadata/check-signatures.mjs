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

// Assertions ported from the pinned compiler/test.ss metadata cases.
import assert from "node:assert/strict";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const [
  basicPath,
  compositesPath,
  p256Path,
  curvesPath,
  sequencesPath,
  opaquePath,
  opaquePointPath,
  protoPath,
] = process.argv.slice(2);
if (
  !basicPath ||
  !compositesPath ||
  !p256Path ||
  !curvesPath ||
  !sequencesPath ||
  !opaquePath ||
  !opaquePointPath ||
  !protoPath
) {
  throw Error(
    "Usage: node check-signatures.mjs <basic-build> <composites-build> <p256-tags-build> <curves-build> <sequences-build> <opaque-build> <opaque-point-build> <proto-build>",
  );
}
const load = async (path) =>
  import(pathToFileURL(resolve(path, "contract/index.js")));
const [basic, composites, p256, curves, sequences, opaque, opaquePoint, proto] =
  await Promise.all(
    [
      basicPath,
      compositesPath,
      p256Path,
      curvesPath,
      sequencesPath,
      opaquePath,
      opaquePointPath,
      protoPath,
    ].map(load),
  );

const s = basic.circuitSignatures;
assert.equal(Object.getPrototypeOf(s), null);
assert.equal(s.constructor, undefined);
assert.equal(s.toString, undefined);
assert.deepEqual(Object.keys(s).sort(), [
  "impureProvable",
  "noArgs",
  "pureNotProvable",
  "usesHelper",
  "witnessOnly",
]);
assert.equal(s.helper, undefined);
assert.deepEqual(
  [s.impureProvable.pure, s.impureProvable.provable],
  [false, true],
);
assert.deepEqual(
  [s.pureNotProvable.pure, s.pureNotProvable.provable],
  [true, false],
);
assert.deepEqual([s.witnessOnly.pure, s.witnessOnly.provable], [false, false]);
assert.deepEqual(s.impureProvable.argumentTypes, [
  { tag: "Boolean" },
  { tag: "Field" },
  { tag: "Uint", maxval: (2n ** 128n - 1n).toString() },
  { tag: "Bytes", length: 32 },
]);
assert.deepEqual(s.noArgs.argumentTypes, []);
assert.deepEqual(s.noArgs.resultType, { tag: "Tuple", types: [] });
assert.equal(typeof s.impureProvable.argumentTypes[2].maxval, "string");

assert.deepEqual(p256.circuitSignatures.curves.argumentTypes, [
  { tag: "Secp256r1Base" },
  { tag: "Secp256r1Scalar" },
  { tag: "Secp256r1Point" },
]);
assert.deepEqual(curves.circuitSignatures.curves.argumentTypes, [
  { tag: "JubjubScalar" },
  { tag: "JubjubPoint" },
  { tag: "Secp256k1Base" },
  { tag: "Secp256k1Scalar" },
  { tag: "Secp256k1Point" },
]);

const c = composites.circuitSignatures;
assert.deepEqual(c.composites.argumentTypes[0], {
  tag: "Struct",
  name: "Outer",
  elements: [
    {
      name: "i",
      type: {
        tag: "Struct",
        name: "Inner",
        elements: [{ name: "f", type: { tag: "Field" } }],
      },
    },
    { name: "b", type: { tag: "Boolean" } },
  ],
});
assert.deepEqual(c.composites.resultType, {
  tag: "Struct",
  name: "Inner",
  elements: [{ name: "f", type: { tag: "Field" } }],
});
assert.deepEqual(c.composites.argumentTypes[1], {
  tag: "Enum",
  name: "Color",
  elements: ["red", "green", "blue"],
});
assert.deepEqual(c.composites.argumentTypes[2], {
  tag: "Alias",
  name: "Nominal",
  type: { tag: "Bytes", length: 8 },
});
assert.deepEqual(c.composites.argumentTypes[3], { tag: "Field" });
assert.deepEqual(c.aliasInStruct.argumentTypes[0], {
  tag: "Vector",
  length: 2,
  type: { tag: "Alias", name: "Nominal", type: { tag: "Bytes", length: 8 } },
});
const v = sequences.circuitSignatures;
const field = { tag: "Field" };
const bool = { tag: "Boolean" };
const fieldVector = { tag: "Vector", length: 3, type: field };
assert.deepEqual(v.same.argumentTypes, [fieldVector, fieldVector]);
assert.deepEqual(v.hetero.argumentTypes[0], {
  tag: "Tuple",
  types: [field, bool],
});
const one = { tag: "Vector", length: 1, type: bool };
assert.deepEqual(v.ones.argumentTypes, [one, one]);
const inner = { tag: "Vector", length: 2, type: field };
const outer = { tag: "Vector", length: 2, type: inner };
assert.deepEqual(v.nested.argumentTypes, [outer, outer]);
assert.deepEqual(v.mixed.argumentTypes[0], {
  tag: "Vector",
  length: 2,
  type: { tag: "Tuple", types: [field, bool] },
});
const empty = { tag: "Tuple", types: [] };
assert.deepEqual(v.unit.argumentTypes[0], empty);
assert.deepEqual(v.zeros.argumentTypes, [empty, empty]);

const opaqueType = opaque.circuitSignatures.op.argumentTypes[0];
assert.equal(opaqueType.tag, "Opaque");
assert.equal(typeof opaqueType.tsType, "string");
assert.match(opaqueType.tsType, /string/);

assert.deepEqual(opaquePoint.circuitSignatures.opaquePoint.argumentTypes, [
  { tag: "Opaque", tsType: "JubjubPoint" },
]);
assert.deepEqual(Object.keys(proto.circuitSignatures), ["__proto__"]);
assert.equal(Object.hasOwn(proto.circuitSignatures, "__proto__"), true);
assert.equal(Object.getPrototypeOf(proto.circuitSignatures), null);
console.log("pinned circuit signature assertions passed");
