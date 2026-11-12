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

export function registerRoundtrip(test, expect, startContract, contractCode, runtime) {
  test('Secp256r1Point round tripping through the ledger', async () => {
    const [contract, context] = await startContract(contractCode, {}, 0);
    // The point at X=0.
    var pt = {
        x: 0x0n,
        y: 0x66485c780e2f83d72433bd5d84a06bb6541c2af31dae871728bf856a174f93f4n,
        identity: false,
    };
    var r = await contract.circuits.test0(context, pt);
    expect(r.result).toEqual(pt);
    expect(contractCode.ledger(r.context.currentQueryContext.state).point).toEqual(pt);
    expect((await contract.circuits.test1(r.context)).result).toEqual([pt.x, pt.y]);
    // The point G.
    pt = {
        x: 0x6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296n,
        y: 0x4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5n,
        identity: false,
    };
    r = await contract.circuits.test0(context, pt);
    expect(r.result).toEqual(pt);
    expect(contractCode.ledger(r.context.currentQueryContext.state).point).toEqual(pt);
    expect((await contract.circuits.test1(r.context)).result).toEqual([pt.x, pt.y]);
  });
}

export function registerDirect(test, expect, startContract, contractCode, runtime) {
  test('Secp256r1Point accessors on the identity', async () => {
    const [contract, context] = await startContract(contractCode, {}, 0);
    const identity0 = { x: 0n, y: 0n, identity: true };
    const identity1 = { x: 3n, y: 4n, identity: true };
    const identity2 = { ...runtime.secp256r1MulGenerator(7n), identity: true };
    await expect(contract.circuits.test0(context, identity0)).rejects.toThrow(runtime.CompactError);
    await expect(contract.circuits.test1(context, identity0)).rejects.toThrow(runtime.CompactError);
    expect((await contract.circuits.test2(context, identity0)).result).toEqual([0n, 0n]);
    await expect(contract.circuits.test0(context, identity1)).rejects.toThrow(runtime.CompactError);
    await expect(contract.circuits.test1(context, identity1)).rejects.toThrow(runtime.CompactError);
    expect((await contract.circuits.test2(context, identity1)).result).toEqual([0n, 0n]);
    await expect(contract.circuits.test0(context, identity2)).rejects.toThrow(runtime.CompactError);
    await expect(contract.circuits.test1(context, identity2)).rejects.toThrow(runtime.CompactError);
    expect((await contract.circuits.test2(context, identity2)).result).toEqual([0n, 0n]);
  });
}

export function registerLedger(test, expect, startContract, contractCode, runtime) {
  test('Secp256r1Point accessors on a ledger point', async () => {
    const [contract, context] = await startContract(contractCode, {}, 0);
    // A Secp256r1Point ledger cell starts out holding the identity, which
    // has no coordinates.
    let L = contractCode.ledger(context.currentQueryContext.state);
    expect(L.pt).toEqual({ x: 0n, y: 0n, identity: true });
    await expect(contract.circuits.storeX(context)).rejects.toThrow(runtime.CompactError);
    await expect(contract.circuits.storeY(context)).rejects.toThrow(runtime.CompactError);
    // The point G, whose coordinates are available.
    const G = {
      x: 0x6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296n,
      y: 0x4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5n,
      identity: false,
    };
    const stored = await contract.circuits.storePoint(context, G);
    L = contractCode.ledger(stored.context.currentQueryContext.state);
    expect(L.pt).toEqual(G);
    const withX = await contract.circuits.storeX(stored.context);
    L = contractCode.ledger(withX.context.currentQueryContext.state);
    expect(L.x).toEqual(G.x);
    const withY = await contract.circuits.storeY(withX.context);
    L = contractCode.ledger(withY.context.currentQueryContext.state);
    expect(L.y).toEqual(G.y);
    // Storing the identity again makes the coordinates unavailable again.
    const back = await contract.circuits.storePoint(withY.context,
                                                    { x: 0n, y: 0n, identity: true });
    await expect(contract.circuits.storeX(back.context)).rejects.toThrow(runtime.CompactError);
    await expect(contract.circuits.storeY(back.context)).rejects.toThrow(runtime.CompactError);
  });
}

export function registerAsserted(test, expect, startContract, contractCode, runtime) {
  test('Secp256r1Point accessors guarded by an assert', async () => {
    const [contract, context] = await startContract(contractCode, {}, 0);
    await expect(contract.circuits.storeXChecked(context))
        .rejects.toThrow(/the identity has no coordinates/);
    const G = runtime.secp256r1MulGenerator(1n);
    const stored = await contract.circuits.storePoint(context, G);
    const checked = await contract.circuits.storeXChecked(stored.context);
    const L = contractCode.ledger(checked.context.currentQueryContext.state);
    expect(L.x).toEqual(G.x);
  });
}
