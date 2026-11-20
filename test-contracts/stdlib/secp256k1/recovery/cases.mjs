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
import { pathToFileURL } from 'node:url';

export async function ownerCases(runtimeEntry) {
  const require = createRequire(runtimeEntry);
  const runtime = await import(/* @vite-ignore */ pathToFileURL(runtimeEntry).href);
  const { secp256k1 } = await import(/* @vite-ignore */ pathToFileURL(require.resolve('@noble/curves/secp256k1.js')).href);
  const msg = Uint8Array.from({ length: 32 }, (_, i) => i + 1);
  const digest = runtime.keccak256(new runtime.CompactTypeBytes(32), msg);
  const signerKey = secp256k1.Point.BASE.multiplyUnsafe(7n);
  const signed = secp256k1.sign(digest, secp256k1.Point.Fn.toBytes(7n), {
    prehash: false,
    format: 'recovered',
  });
  const recoveryId = signed[0];
  const signature = secp256k1.Signature.fromBytes(signed, 'recovered');
  const sig = { r: signature.r, s: signature.s };
  const alternate = new secp256k1.Signature(sig.r, sig.s, recoveryId ^ 1).recoverPublicKey(digest);
  const address = (key) =>
    runtime.keccak256(new runtime.CompactTypeBytes(64), key.toBytes(false).subarray(1)).slice(12);
  const signer = address(signerKey);
  const other = address(alternate);
  const n = secp256k1.Point.Fn.ORDER;
  const twin = { r: sig.r, s: n - sig.s };
  const witness = {
    recoverKey: ({ privateState }, digest, sig, id) =>
      [privateState, runtime.secp256k1EcdsaRecover(digest, sig, Number(id))],
  };
  const alternateVerifies = secp256k1.verify(signature.toBytes('compact'), digest, alternate.toBytes(), {
    prehash: false,
    lowS: false,
  });
  return { msg, digest, sig, recoveryId, signer, other, twin, n, witness, alternateVerifies };
}
