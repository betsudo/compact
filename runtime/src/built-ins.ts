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

import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import { sha512 as nobleSha512 } from '@noble/hashes/sha2.js';
import { keccak_256 } from '@noble/hashes/sha3.js';
import { randomBytes } from '@noble/hashes/utils.js';
import { MAX_FIELD, JUBJUB_SCALAR_MODULUS, SECP256K1_BASE_MODULUS, SECP256K1_SCALAR_MODULUS } from './constants.js';
import { CompactType, CompactTypeJubjubPoint, JubjubPoint, JubjubSchnorrSignature } from './compact-types.js';
import { CompactError } from './error.js';

const FIELD_MODULUS: bigint = MAX_FIELD + 1n;

function subModulo(x: bigint, y: bigint, modulus: bigint): bigint {
  const difference = x - y;
  return difference >= 0n ? difference : difference + modulus;
}

/** Add two canonical secp256k1 base field elements. */
export function secp256k1BaseAdd(x: bigint, y: bigint): bigint {
  return (x + y) % SECP256K1_BASE_MODULUS;
}

/** Subtract two canonical secp256k1 base field elements. */
export function secp256k1BaseSub(x: bigint, y: bigint): bigint {
  return subModulo(x, y, SECP256K1_BASE_MODULUS);
}

/** Multiply two canonical secp256k1 base field elements. */
export function secp256k1BaseMul(x: bigint, y: bigint): bigint {
  return (x * y) % SECP256K1_BASE_MODULUS;
}

/** Negate a canonical secp256k1 base field element. */
export function secp256k1BaseNeg(x: bigint): bigint {
  return subModulo(0n, x, SECP256K1_BASE_MODULUS);
}

/** Invert a nonzero canonical secp256k1 base field element. */
export function secp256k1BaseInv(x: bigint): bigint {
  if (x === 0n) {
    throw new CompactError('secp256k1 base field has no inverse for 0');
  }
  return inverseModuloPrime(x, SECP256K1_BASE_MODULUS);
}

/** Add two canonical secp256k1 scalar field elements. */
export function secp256k1ScalarAdd(x: bigint, y: bigint): bigint {
  return (x + y) % SECP256K1_SCALAR_MODULUS;
}

/** Subtract two canonical secp256k1 scalar field elements. */
export function secp256k1ScalarSub(x: bigint, y: bigint): bigint {
  return subModulo(x, y, SECP256K1_SCALAR_MODULUS);
}

/** Multiply two canonical secp256k1 scalar field elements. */
export function secp256k1ScalarMul(x: bigint, y: bigint): bigint {
  return (x * y) % SECP256K1_SCALAR_MODULUS;
}

/** Negate a canonical secp256k1 scalar field element. */
export function secp256k1ScalarNeg(x: bigint): bigint {
  return subModulo(0n, x, SECP256K1_SCALAR_MODULUS);
}

/** Invert a nonzero canonical secp256k1 scalar field element. */
export function secp256k1ScalarInv(x: bigint): bigint {
  if (x === 0n) {
    throw new CompactError('secp256k1 scalar field has no inverse for 0');
  }
  return inverseModuloPrime(x, SECP256K1_SCALAR_MODULUS);
}

function inverseModuloPrime(value: bigint, modulus: bigint): bigint {
  // Fermat's little theorem for nonzero elements. The exponent is public and fixed per field.
  let result = 1n;
  let power = value;
  for (let exponent = modulus - 2n; exponent > 0n; exponent >>= 1n) {
    if ((exponent & 1n) !== 0n) {
      result = (result * power) % modulus;
    }
    power = (power * power) % modulus;
  }
  return result;
}

/**
 * Field addition
 * returns the result of adding x and y, wrapping if necessary
 * x and y are assumed to be values in the range [0, FIELD_MODULUS)
 */
export function addField(x: bigint, y: bigint): bigint {
  const t = x + y;
  // effectively mod(x + y, FIELD_MODULUS) for x and y in the assumed range
  // (x + y) % FIELD_MODULUS would also work but would likely be more expensive
  return t < FIELD_MODULUS ? t : t - FIELD_MODULUS;
}

/**
 * Field subtraction
 * returns the result of subtracting y from x, wrapping if necessary
 * x and y are assumed to be values in the range [0, FIELD_MODULUS)
 */
export function subField(x: bigint, y: bigint): bigint {
  // effectively mod(x - y, FIELD_MODULUS) for x and y in the assumed range
  // NB: JavaScript % implements remainder rather than modulus, so
  // (x - y) % FIELD_MODULUS would return an incorrect value for negative values of x - y.
  // also, any implementation involving % would likely be more expensive
  const t = x - y;
  return t >= 0 ? t : t + FIELD_MODULUS;
}

/**
 * Field multiplication
 * returns the result of multipying x and y, wrapping if necessary
 * x and y are assumed to be values in the range [0, FIELD_MODULUS)
 */
export function mulField(x: bigint, y: bigint): bigint {
  // effectively mod(x * y, FIELD_MODULUS) for x and y in the assumed range
  // (although JavaScript % implements remainder rather than modulo, remainder
  // and modulo coincide for nonnegative inputs)
  return (x * y) % FIELD_MODULUS;
}

/**
 * The Compact builtin `transientHash` function
 *
 * This function is a circuit-efficient compression function from arbitrary
 * data to field elements, which is not guaranteed to persist between upgrades.
 * It should not be used to derive state data, but can be used for consistency
 * checks.
 */
export function transientHash<A>(rtType: CompactType<A>, value: A): bigint {
  return ocrt.valueToBigInt(ocrt.transientHash(rtType.alignment(), rtType.toValue(value)));
}

/**
 * The Compact builtin `transientCommit` function
 *
 * This function is a circuit-efficient commitment function from arbitrary
 * values representable in Compact, and a field element commitment opening, to
 * field elements, which is not guaranteed to persist between
 * upgrades. It should not be used to derive state data, but can be used for
 * consistency checks.
 *
 * @throws If `opening` is out of range for field elements
 */
export function transientCommit<A>(rtType: CompactType<A>, value: A, opening: bigint): bigint {
  return ocrt.valueToBigInt(ocrt.transientCommit(rtType.alignment(), rtType.toValue(value), ocrt.bigIntToValue(opening)));
}

/**
 * The Compact builtin `persistentHash` function
 *
 * This function is a non-circuit-optimised hash function for mostly arbitrary
 * data. It is guaranteed to persist between upgrades, with the exception of
 * devnet. It *should* be used to derive state data, and not for consistency
 * checks where avoidable.
 *
 * Note that data containing `Opaque` elements *may* throw runtime errors, and
 * cannot be relied upon as a consistent representation.
 *
 * @throws If `rtType` encodes a type containing Compact 'Opaque' types
 */
export function persistentHash<A>(rtType: CompactType<A>, value: A): Uint8Array {
  const wrapped = ocrt.persistentHash(rtType.alignment(), rtType.toValue(value))[0];
  const res = new Uint8Array(32);
  res.set(wrapped, 0);
  return res;
}

/**
 * The Compact builtin `persistentCommit` function
 *
 * This function is a non-circuit-optimised commitment function from arbitrary
 * values representable in Compact, and a 256-bit bytestring opening, to a
 * 256-bit bytestring. It is guaranteed to persist between upgrades. It
 * *should* be used to derive state data, and not for consistency checks where
 * avoidable.
 *
 * Note that data containing `Opaque` elements *may* throw runtime errors, and
 * cannot be relied upon as a consistent representation.
 *
 * @throws If `rtType` encodes a type containing Compact 'Opaque' types, or
 * `opening` is not 32 bytes long
 */
export function persistentCommit<A>(rtType: CompactType<A>, value: A, opening: Uint8Array): Uint8Array {
  if (opening.length != 32) {
    throw new CompactError('Expected 32-byte string');
  }
  const wrapped = ocrt.persistentCommit(rtType.alignment(), rtType.toValue(value), [opening])[0];
  const res = new Uint8Array(32);
  res.set(wrapped, 0);
  return res;
}

/**
 * The Compact builtin `degradeToTransient` function
 *
 * This function "degrades" the output of a {@link persistentHash} or
 * {@link persistentCommit} to a field element, which can then be used in
 * {@link transientHash} or {@link transientCommit}.
 *
 * @throws If `x` is not 32 bytes long
 */
export function degradeToTransient(x: Uint8Array): bigint {
  if (x.length != 32) {
    throw new CompactError('Expected 32-byte string');
  }
  return ocrt.valueToBigInt(ocrt.degradeToTransient([x]));
}

/**
 * The Compact builtin `upgradeFromTransient` function
 *
 * This function "upgrades" the output of a {@link transientHash} or
 * {@link transientCommit} to 256-bit byte string, which can then be used in
 * {@link persistentHash} or {@link persistentCommit}.
 *
 * @throws If `x` is not a valid field element
 */
export function upgradeFromTransient(x: bigint): Uint8Array {
  const wrapped = ocrt.upgradeFromTransient(ocrt.bigIntToValue(x))[0];
  const res = new Uint8Array(32);
  res.set(wrapped, 0);
  return res;
}

/**
 * The Compact builtin `keccak256` function
 *
 * Hashes normalized FAB atoms for historical generated contracts.
 */
export function keccak256<A>(rtType: CompactType<A>, value: A): Uint8Array {
  const chunks = rtType.toValue(value);
  const totalLength = chunks.reduce((acc, chunk) => acc + chunk.length, 0);
  const bytes = new Uint8Array(totalLength);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.length;
  }
  return keccak_256(bytes);
}

/**
 * Hash the aligned binary representation used by Ledger 9 ZKIR.
 * Each atom occupies its declared byte width. Field atoms occupy 32 bytes.
 */
export function keccak256Aligned<A>(rtType: CompactType<A>, value: A): Uint8Array {
  return keccak_256(alignedHashInput(rtType, value));
}

/** Hash an aligned Compact value and return the 64-byte SHA-512 digest. */
export function sha512<A>(rtType: CompactType<A>, value: A): Uint8Array {
  return nobleSha512(alignedHashInput(rtType, value));
}

function alignedHashInput<A>(rtType: CompactType<A>, value: A): Uint8Array {
  const atoms = rtType.toValue(value);
  const alignment = rtType.alignment();
  if (atoms.length !== alignment.length) {
    throw new CompactError('invalid Compact hash input');
  }
  const widths = alignment.map((segment, index) => {
    if (segment.tag !== 'atom' || segment.value.tag === 'compress') {
      throw new CompactError('invalid Compact hash input');
    }
    const width = segment.value.tag === 'field' ? 32 : segment.value.length;
    if (atoms[index].length > width) {
      throw new CompactError('invalid Compact hash input');
    }
    return width;
  });
  const bytes = new Uint8Array(widths.reduce((length, width) => length + width, 0));
  let offset = 0;
  for (const [index, atom] of atoms.entries()) {
    bytes.set(atom, offset);
    offset += widths[index];
  }
  return bytes;
}

export function jubjubPointX(pt: JubjubPoint): bigint {
  return pt.x;
}

export function jubjubPointY(pt: JubjubPoint): bigint {
  return pt.y;
}

export function constructJubjubPoint(x: bigint, y: bigint): JubjubPoint {
  return { x, y };
}

/**
 * The Compact builtin `hashToCurve` function
 *
 * This function maps arbitrary values representable in Compact to elliptic
 * curve points in the proof system's embedded curve.
 *
 * Outputs are guaranteed to have unknown discrete logarithm with respect to
 * the group base, and any other output, but are not guaranteed to be unique (a
 * given input can be proven correct for multiple outputs).
 *
 * Inputs of different types may have the same output, if they have the same
 * field-aligned binary representation.
 */
export function hashToCurve<A>(rtType: CompactType<A>, x: A): JubjubPoint {
  return CompactTypeJubjubPoint.fromValue(ocrt.hashToCurve(rtType.alignment(), rtType.toValue(x)));
}

/**
 * The Compact builtin `ecAdd` function
 *
 * This function add two elliptic curve points (in multiplicative notation)
 */
export function ecAdd(a: JubjubPoint, b: JubjubPoint): JubjubPoint {
  return CompactTypeJubjubPoint.fromValue(ocrt.ecAdd(CompactTypeJubjubPoint.toValue(a), CompactTypeJubjubPoint.toValue(b)));
}

/**
 * The Compact builtin `ecNeg` function
 *
 * This function negates an elliptic curve point. On the JubJub twisted
 * Edwards curve, the negation of (x, y) is (-x, y).
 */
export function ecNeg(a: JubjubPoint): JubjubPoint {
  return constructJubjubPoint(a.x === 0n ? 0n : FIELD_MODULUS - a.x, a.y);
}

/**
 * The Compact builtin `ecMul` function
 *
 * This function multiplies an elliptic curve point by a scalar (in
 * multiplicative notation).
 *
 * @throws RangeError if the scalar is outside [0, JUBJUB_SCALAR_MODULUS).
 */
export function ecMul(a: JubjubPoint, b: bigint): JubjubPoint {
  assertCanonicalJubjubScalar(b);
  return CompactTypeJubjubPoint.fromValue(ocrt.ecMul(CompactTypeJubjubPoint.toValue(a), ocrt.bigIntToValue(b)));
}

/**
 * The Compact builtin `ecMulGenerator` function
 *
 * This function multiplies the primary group generator of the embedded curve
 * by a scalar (in multiplicative notation).
 *
 * @throws RangeError if the scalar is outside [0, JUBJUB_SCALAR_MODULUS).
 */
export function ecMulGenerator(b: bigint): JubjubPoint {
  assertCanonicalJubjubScalar(b);
  return CompactTypeJubjubPoint.fromValue(ocrt.ecMulGenerator(ocrt.bigIntToValue(b)));
}

function assertCanonicalJubjubScalar(scalar: bigint): void {
  // The Ledger 9 WASM primitive reduces its operand. Compact requires an
  // explicit reduction before multiplying by a noncanonical Field.
  if (scalar < 0n || scalar >= JUBJUB_SCALAR_MODULUS) {
    throw new RangeError('invalid Jubjub scalar');
  }
}

/**
 * Concatenates multiple {@link AlignedValue}s
 * @internal
 */
export function alignedConcat(...values: ocrt.AlignedValue[]): ocrt.AlignedValue {
  const res: ocrt.AlignedValue = { value: [], alignment: [] };
  for (const value of values) {
    res.value = res.value.concat(value.value);
    res.alignment = res.alignment.concat(value.alignment);
  }
  return res;
}

/**
 * Samples a random JubJub scalar.
 *
 * The returned value is in the range [0, JUBJUB_SCALAR_MODULUS).
 */
export function jubjubSampleScalar(): bigint {
  const bits = JUBJUB_SCALAR_MODULUS.toString(2).length;
  const length = Math.ceil(bits / 8);
  const highByteMask = 0xff >>> (length * 8 - bits);
  for (;;) {
    const bytes = randomBytes(length);
    // Keep a uniform bit-width candidate; reducing modulo q would bias it.
    bytes[length - 1] &= highByteMask;
    const candidate = ocrt.valueToBigInt([bytes]);
    if (candidate < JUBJUB_SCALAR_MODULUS) return candidate;
  }
}

/**
 * Alias for {@link jubjubSampleScalar}. Samples a random JubJub Schnorr signing key.
 */
export const sampleJubjubSchnorrSk = jubjubSampleScalar;

/**
 * Reduce any signed integer modulo the JubJub scalar field order.
 *
 * The returned Euclidean residue is in the range [0, JUBJUB_SCALAR_MODULUS),
 * including when the input is negative.
 */
export function reduceModJubjubOrder(value: bigint): bigint {
  const remainder = value % JUBJUB_SCALAR_MODULUS;
  return remainder < 0n ? remainder + JUBJUB_SCALAR_MODULUS : remainder;
}

/**
 * Derives the Schnorr verifying key (public key) from a signing key.
 *
 * Equivalent to {@link ecMulGenerator}(reduceModJubjubOrder(signingKey)).
 */
export function jubjubSchnorrVerifyingKey(signingKey: bigint): JubjubPoint {
  return ecMulGenerator(reduceModJubjubOrder(signingKey));
}

/**
 * Produces a Schnorr signature over the JubJub curve.
 *
 * - `rtType` / `msg`: the message as a typed Compact value
 * - `sk`: canonical signing key in [0, JUBJUB_SCALAR_MODULUS) (e.g. as returned by {@link jubjubSampleScalar})
 *
 * The signature scheme:
 * - Nonce `r` sampled uniformly at random
 * - Announcement `R = r·G`
 * - Challenge `c = PoseidonHash(R.x, R.y, pk.x, pk.y, msg...)`
 * - Response `s = r + c·sk` (in the JubJub scalar field)
 *
 * @throws RangeError if the signing key is outside [0, JUBJUB_SCALAR_MODULUS).
 */
export function jubjubSchnorrSign<A>(rtType: CompactType<A>, msg: A, signingKey: bigint): JubjubSchnorrSignature {
  if (signingKey < 0n || signingKey >= JUBJUB_SCALAR_MODULUS) {
    throw new RangeError('jubjubSchnorrSign: signing key must be in [0, JUBJUB_SCALAR_MODULUS)');
  }
  const r = jubjubSampleScalar();
  const announcement = ecMulGenerator(r);
  const verifyingKey = ecMulGenerator(signingKey);

  const challengeAlignment: ocrt.Alignment = [
    ...CompactTypeJubjubPoint.alignment(),
    ...CompactTypeJubjubPoint.alignment(),
    ...rtType.alignment(),
  ];
  const challengeValue: ocrt.Value = [
    ...CompactTypeJubjubPoint.toValue(announcement),
    ...CompactTypeJubjubPoint.toValue(verifyingKey),
    ...rtType.toValue(msg),
  ];
  const c = reduceModJubjubOrder(ocrt.valueToBigInt(ocrt.transientHash(challengeAlignment, challengeValue)));

  const response = reduceModJubjubOrder(r + c * signingKey);
  return { announcement, response };
}

/**
 * Verifies a Schnorr signature over the JubJub curve.
 *
 * - `rtType` / `msg`: the message as a typed Compact value
 * - `pk`: verifying key (a JubJubPoint / EmbeddedGroupAffine)
 * - `sig`: signature as returned by {@link jubjubSchnorrSign}
 *
 * Returns `true` if the signature is valid (i.e. `s·G == R + c·pk`).
 */
export function jubjubSchnorrVerify<A>(
  rtType: CompactType<A>,
  msg: A,
  verifyingKey: JubjubPoint,
  sig: JubjubSchnorrSignature,
): boolean {
  const { announcement, response } = sig;

  const challengeAlignment: ocrt.Alignment = [
    ...CompactTypeJubjubPoint.alignment(),
    ...CompactTypeJubjubPoint.alignment(),
    ...rtType.alignment(),
  ];
  const challengeValue: ocrt.Value = [
    ...CompactTypeJubjubPoint.toValue(announcement),
    ...CompactTypeJubjubPoint.toValue(verifyingKey),
    ...rtType.toValue(msg),
  ];
  const c = reduceModJubjubOrder(ocrt.valueToBigInt(ocrt.transientHash(challengeAlignment, challengeValue)));

  const lhs = ecMulGenerator(response);
  const rhs = ecAdd(announcement, ecMul(verifyingKey, c));

  return lhs.x === rhs.x && lhs.y === rhs.y;
}
