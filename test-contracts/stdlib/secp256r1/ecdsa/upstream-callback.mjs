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

export function registerOrdinary(test, expect, startContract, contractCode, runtime, p256) {
  test('secp256r1 ECDSA verification', async () => {
    const [contract, context] = await startContract(contractCode, {}, 0);
    const msg = new Uint8Array(32);
    for (let i = 0; i < 32; i++) msg[i] = i + 1;
    const digest = runtime.keccak256(new runtime.CompactTypeBytes(32), msg);
    const SK = 7n;
    // Signing is RFC 6979 deterministic, so the signature is the same on
    // every run.
    const parsed = p256.Signature.fromBytes(
      p256.sign(digest, p256.Point.Fn.toBytes(SK), { prehash: false }));
    const sig = { r: parsed.r, s: parsed.s };
    const pk = runtime.secp256r1MulGenerator(SK);
    expect((await contract.circuits.verify(context, msg, sig, pk)).result).toEqual(true);
    // The malleated twin verifies against the same key, as it does for
    // secp256k1: negating s negates the nonce point, and only its
    // x-coordinate is compared against r.
    const n = p256.Point.Fn.ORDER;
    const twin = { r: sig.r, s: n - sig.s };
    expect((await contract.circuits.verify(context, msg, twin, pk)).result).toEqual(true);
    // A valid signature does not verify against somebody else's key.
    const otherPk = runtime.secp256r1MulGenerator(SK + 1n);
    expect((await contract.circuits.verify(context, msg, sig, otherPk)).result).toEqual(false);
    // The identity public key is rejected outright.
    const identity = runtime.secp256r1MulGenerator(0n);
    await expect(contract.circuits.verify(context, msg, sig, identity)).rejects.toThrow(
      'failed assert: Secp256r1Point identity is not a permitted secp256r1EcdsaVerify verification key');
  });
}
