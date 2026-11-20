// This file is part of Compact.
// Copyright (C) 2025 Midnight Foundation
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

import { ed25519 } from '@noble/curves/ed25519.js';
import { ContractAddress } from '@midnightntwrk/onchain-runtime-v4';
import { secp256k1 } from '@noble/curves/secp256k1.js';
import { p256 } from '@noble/curves/nist.js';
import type { Curve25519Point, Secp256k1Point, Secp256r1Point } from './compact-types.js';
import { CompactError } from './error.js';
import { CURVE25519_BASE_MODULUS, SECP256K1_BASE_MODULUS, SECP256R1_BASE_MODULUS } from './constants.js';
import { EncodedContractAddress } from './zswap.js';

/**
 * Regex matching hex strings of even length.
 */
export const HEX_REGEX_NO_PREFIX = /^([0-9A-Fa-f]{2})*$/;

/**
 * The expected length (in bytes) of a contract address.
 */
export const CONTRACT_ADDRESS_BYTE_LENGTH = 32;

/**
 * Tests whether the input value is a {@link ContractAddress}, i.e., string.
 *
 * @param x The value that is tested to be a {@link ContractAddress}.
 */
export function isContractAddress(x: unknown): x is ContractAddress {
  return typeof x === 'string' && x.length === CONTRACT_ADDRESS_BYTE_LENGTH * 2 && HEX_REGEX_NO_PREFIX.test(x);
}

export function isEncodedContractAddress(x: unknown): x is EncodedContractAddress {
  return (
    typeof x === 'object' &&
    x !== null &&
    x !== undefined &&
    'bytes' in x &&
    x.bytes instanceof Uint8Array &&
    x.bytes.length == CONTRACT_ADDRESS_BYTE_LENGTH
  );
}

/** Checks a secp256k1 point supplied to a generated circuit. */
export function isValidSecp256k1Point(value: unknown): value is Secp256k1Point {
  if (typeof value !== 'object' || value === null || !('x' in value) || !('y' in value) || !('identity' in value)) {
    return false;
  }
  const { x, y, identity } = value;
  const modulus = SECP256K1_BASE_MODULUS;
  if (
    typeof x !== 'bigint' ||
    x < 0n ||
    x >= modulus ||
    typeof y !== 'bigint' ||
    y < 0n ||
    y >= modulus ||
    typeof identity !== 'boolean'
  ) {
    return false;
  }
  return identity || (y * y) % modulus === (((x * x) % modulus) * x + 7n) % modulus;
}

/** Checks a P-256 point supplied to a generated circuit. */
export function isValidSecp256r1Point(value: unknown): value is Secp256r1Point {
  if (typeof value !== 'object' || value === null || !('x' in value) || !('y' in value) || !('identity' in value)) {
    return false;
  }
  const { x, y, identity } = value;
  const modulus = SECP256R1_BASE_MODULUS;
  if (
    typeof x !== 'bigint' ||
    x < 0n ||
    x >= modulus ||
    typeof y !== 'bigint' ||
    y < 0n ||
    y >= modulus ||
    typeof identity !== 'boolean'
  ) {
    return false;
  }
  if (!identity) {
    try {
      p256.Point.fromAffine({ x, y }).assertValidity();
    } catch {
      return false;
    }
  }
  return true;
}

/** Convert an affine Compact point to the P-256 group representation. */
export function secp256r1ToProjective(p: Secp256r1Point): ReturnType<typeof p256.Point.fromAffine> {
  if (p.identity) {
    return p256.Point.ZERO;
  }
  return p256.Point.fromAffine({ x: p.x, y: p.y });
}

/**
 * Project a noble-curves point back down to the simple affine
 * `Secp256r1Point` representation.
 */
export function secp256r1FromProjective(p: ReturnType<typeof p256.Point.fromAffine>): Secp256r1Point {
  const k = p.toAffine();
  if (/* k == p256.Point.ZERO */ k.x == 0n && k.y == 0n) {
    return { x: 0n, y: 0n, identity: true };
  } else {
    const { x, y } = k;
    return { x: x, y: y, identity: false };
  }
}

/** Convert an affine Compact point to the secp256k1 group representation. */
export function secp256k1ToProjective(p: Secp256k1Point): ReturnType<typeof secp256k1.Point.fromAffine> {
  return p.identity ? secp256k1.Point.ZERO : secp256k1.Point.fromAffine({ x: p.x, y: p.y });
}

/** Convert a group element to the canonical affine Compact representation. */
export function secp256k1FromProjective(p: ReturnType<typeof secp256k1.Point.fromAffine>): Secp256k1Point {
  const { x, y } = p.toAffine();
  return { x, y, identity: x === 0n && y === 0n };
}

/**
 * Recover the secp256k1 public key from an ECDSA signature and a message hash.
 *
 * ## Recovery ID
 * - bit 0 (`recoveryId & 1`) is the parity of `R.y`: 0 for even, 1 for odd.
 * - bit 1 (`recoveryId >= 2`) says whether the reduction wrapped, i.e. whether
 *   `R.x` is `r` (0, 1) or `r + n` (2, 3).
 *
 * - 0: `R = (r, y)` with `y` even — the common case.
 * - 1: `R = (r, y)` with `y` odd — the other common case.
 * - 2: `R = (r + n, y)` with `y` even.
 * - 3: `R = (r + n, y)` with `y` odd.
 */
export function secp256k1EcdsaRecover(
  msgHash: Uint8Array,
  sig: { readonly r: bigint; readonly s: bigint },
  recoveryId: number,
): Secp256k1Point {
  if (msgHash.length !== 32) {
    throw new CompactError('expected a 32-byte message hash');
  }
  if (!Number.isInteger(recoveryId) || recoveryId < 0 || recoveryId > 3) {
    throw new CompactError('expected a recovery id in the range [0, 3]');
  }
  const nobleSig = new secp256k1.Signature(sig.r, sig.s, recoveryId);
  return secp256k1FromProjective(nobleSig.recoverPublicKey(msgHash));
}

/**
 * Recover the secp256r1 public key from an ECDSA signature and a message hash.
 *
 * The recovery id means the same thing as it does for {@link secp256k1EcdsaRecover}.
 */
export function secp256r1EcdsaRecover(
  msgHash: Uint8Array,
  sig: { readonly r: bigint; readonly s: bigint },
  recoveryId: number,
): Secp256r1Point {
  if (msgHash.length !== 32) {
    throw new CompactError('expected a 32-byte message hash');
  }
  if (!Number.isInteger(recoveryId) || recoveryId < 0 || recoveryId > 3) {
    throw new CompactError('expected a recovery id in the range [0, 3]');
  }
  const nobleSig = new p256.Signature(sig.r, sig.s, recoveryId);
  return secp256r1FromProjective(nobleSig.recoverPublicKey(msgHash));
}

export const fromHex = (s: string): Uint8Array => Buffer.from(s, 'hex');

export const toHex = (s: Uint8Array): string => Buffer.from(s).toString('hex');

/**
 * Check whether a value is a valid `Curve25519Point`: an object whose `x` and
 * `y` are bigints in the base field and which lies in the prime-order subgroup
 * of the curve. The identity is the ordinary affine point (0, 1).
 */
export function isValidCurve25519Point(p: unknown): p is Curve25519Point {
  if (typeof p !== 'object' || p === null) {
    return false;
  }

  const { x, y } = p as { x: unknown; y: unknown };
  if (typeof x !== 'bigint' || typeof y !== 'bigint') {
    return false;
  }
  // `fromAffine` only checks the coordinates fit in 256 bits, so the field range is checked here.
  if (x < 0n || x >= CURVE25519_BASE_MODULUS || y < 0n || y >= CURVE25519_BASE_MODULUS) {
    return false;
  }
  // `assertValidity` rejects the identity, so it is only called for other points.
  if (x !== 0n || y !== 1n) {
    try {
      const pt = ed25519.Point.fromAffine({ x, y });
      pt.assertValidity();
      if (!pt.isTorsionFree()) {
        return false;
      }
    } catch {
      return false;
    }
  }
  return true;
}

/**
 * Lift the simple affine `Curve25519Point` representation into a noble-curves
 * projective point. The point is assumed to be valid, points passed from
 * compiler-generated code are always valid ones.
 * @internal
 */
export function curve25519ToProjective(p: Curve25519Point): ReturnType<typeof ed25519.Point.fromAffine> {
  if (p.x === 0n && p.y === 1n) {
    return ed25519.Point.ZERO;
  }
  return ed25519.Point.fromAffine({ x: p.x, y: p.y });
}

/**
 * Project a noble-curves point back down to the simple affine
 * `Curve25519Point` representation.
 */
export function curve25519FromProjective(p: ReturnType<typeof ed25519.Point.fromAffine>): Curve25519Point {
  const { x, y } = p.toAffine();
  return { x: x, y: y };
}
