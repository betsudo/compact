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

export function registerWitness(test, expect, startContract, contractCode, runtime) {
  test('Secp256r1Point coming from witnesses', async () => {
    const witnesses = {
      point0(wc) {
        return [
          wc.privateState,
          {
            x: 0x0n,
            y: 0x66485c780e2f83d72433bd5d84a06bb6541c2af31dae871728bf856a174f93f4n,
            identity: false,
          },
        ];
      },
      point1(wc) {
        return [
          wc.privateState,
          {
            x: 0x6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296n,
            y: 0x4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5n,
            identity: false,
          },
        ];
      },
    };
    const [contract, context] = await startContract(contractCode, witnesses, 0);
    // The point at X=1.
    var pt = {
        x: 0x0n,
        y: 0x66485c780e2f83d72433bd5d84a06bb6541c2af31dae871728bf856a174f93f4n,
        identity: false,
    };
    var r = await contract.circuits.test0(context);
    expect(r.result).toEqual(pt);
    expect(contractCode.ledger(r.context.currentQueryContext.state).point).toEqual(pt);
    // The point G.
    pt = {
        x: 0x6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296n,
        y: 0x4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5n,
        identity: false,
    };
    r = await contract.circuits.test1(context);
    expect(r.result).toEqual(pt);
    expect(contractCode.ledger(r.context.currentQueryContext.state).point).toEqual(pt);
  });
}

export function registerHash608(test, expect, startContract, contractCode, runtime) {
  test('Issue 608 for secp256r1', async () => {
    const [contract, context] = await startContract(contractCode, {}, 0);
    const p0 = runtime.secp256r1MulGenerator(5n);
    const p1 = runtime.secp256r1MulGenerator(7n);
    await contract.circuits.test(context, p0, p1);
  });
}
