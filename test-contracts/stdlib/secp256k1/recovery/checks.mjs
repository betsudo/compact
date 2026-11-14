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

import { createRequire } from 'node:module';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { createTestContract } from '@test/compact-test';
import { secp256k1EcdsaRecover } from '@midnight-ntwrk/compact-runtime';
import { expect, test } from 'vitest';

import { ownerCases } from './cases.mjs';

export function registerRecoveryTests(compile) {
  test('runs the original owner binding fixture with both signature representatives and both refusals', async () => {
    const { build, result } = compile('recovery', { options: {} }, ['ts', 'rust'], 'upstream-owner.compact');
    expect(result.status, result.stderr).toBe(0);
    const source = join(build, 'contract/index.js');
    const { Contract, ledger } = await import(/* @vite-ignore */ pathToFileURL(source).href);
    const runtime = createRequire(source).resolve('@midnight-ntwrk/compact-runtime');
    const { msg, sig, recoveryId, signer, other, twin, n, witness, alternateVerifies } =
      await ownerCases(runtime);
    const { contract, ctx } = await createTestContract(Contract, witness, 0);
    const withOwner = (owner) => contract.circuits.setOwner(ctx, owner).context;
    const verify = (owner, sig, id) =>
      contract.circuits.verifyOwnerSignature(withOwner(owner), msg, sig, BigInt(id));

    expect(alternateVerifies).toBe(true);
    expect(other).not.toEqual(signer);
    const valid = verify(signer, sig, recoveryId);
    expect(ledger(valid.context.currentQueryContext.state).owner).toEqual(signer);
    expect(sig.s <= n / 2n).toBe(true);
    expect(twin.s > n / 2n).toBe(true);
    const validTwin = verify(signer, twin, recoveryId ^ 1);
    expect(ledger(validTwin.context.currentQueryContext.state).owner).toEqual(signer);
    expect(() => verify(signer, sig, recoveryId ^ 1)).toThrow('failed assert: not the owner');
    expect(() => verify(other, sig, recoveryId)).toThrow('failed assert: not the owner');
  });

  test('recovers both parities when the nonce x coordinate was reduced', () => {
    const digest = new Uint8Array(32);
    digest[31] = 1;
    const sig = { r: 2n, s: 1n };
    expect(secp256k1EcdsaRecover(digest, sig, 2)).toStrictEqual({
      x: 0x8a0dee06163f357e0e71688f3785eb53891144624c633ed43706b6268bb4316fn,
      y: 0xe440d41220c85c709a6ceb8bb5faf8963408733e2451994f9bab1da5e0bfbff0n,
      identity: false,
    });
    expect(secp256k1EcdsaRecover(digest, sig, 3)).toStrictEqual({
      x: 0x19099fbdddcdb7a14410b5c6ba73f51bd2f74a3d1cd8456fcc1da68ed6468884n,
      y: 0xb07b9f7dd9ea5753e009246a1db85e7e1644bfdee276881a950ce59fdd45b45bn,
      identity: false,
    });
  });
}
