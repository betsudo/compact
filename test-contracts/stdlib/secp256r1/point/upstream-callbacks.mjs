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

export function registerEquality(test, expect, startContract, contractCode, runtime) {
  test('Secp256r1Point equality', async () => {
    const [contract, context] = await startContract(contractCode, {}, 0);
    const p1 = runtime.secp256r1MulGenerator(5n);
    const p2 = runtime.secp256r1MulGenerator(5n);
    const p3 = runtime.secp256r1MulGenerator(7n);
    const p4 = (await contract.circuits.getDefault(context)).result;
    const p5 = runtime.secp256r1MulGenerator(0n);
    expect((await contract.circuits.pointsEqual(context, p1, p2)).result).toEqual(true);
    expect((await contract.circuits.pointsEqual(context, p1, p3)).result).toEqual(false);
    expect((await contract.circuits.pointsNotEqual(context, p1, p2)).result).toEqual(false);
    expect((await contract.circuits.pointsNotEqual(context, p1, p3)).result).toEqual(true);
    expect((await contract.circuits.pointsEqual(context, p4, p5)).result).toEqual(true);
    expect((await contract.circuits.pointsEqual(context, p1, p4)).result).toEqual(false);
    expect((await contract.circuits.pointsNotEqual(context, p4, p5)).result).toEqual(false);
    expect((await contract.circuits.pointsNotEqual(context, p1, p4)).result).toEqual(true);
    });
}

export function registerIdentity(test, expect, startContract, contractCode, runtime) {
  test('Secp256r1Point identity encodes the same whatever its coordinates', async () => {
    const [contract, context] = await startContract(contractCode, {}, 0);
    const identity0 = { x: 0n, y: 0n, identity: true };
    const identity1 = { x: 4n, y: 4n, identity: true };
    const identity2 = { ...runtime.secp256r1MulGenerator(7n), identity: true };
    // Two identity points are equal and hash the same, whatever their coordinates.
    await contract.circuits.identityEqual(context, identity0, identity1);
    await contract.circuits.identityEqual(context, identity1, identity2);
    const hash0 = (await contract.circuits.hashPoint(context, identity0)).result;
    expect((await contract.circuits.hashPoint(context, identity1)).result).toEqual(hash0);
    expect((await contract.circuits.hashPoint(context, identity2)).result).toEqual(hash0);
    // Every identity point is stored, and read back, as the default one.
    for (const identity of [identity0, identity1, identity2]) {
      const r = await contract.circuits.storePoint(context, identity);
      expect(r.result).toEqual(identity0);
      expect(contractCode.ledger(r.context.currentQueryContext.state).point).toEqual(identity0);
    }
  });
}
