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

const baseModulus = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2fn;
const scalarModulus = 0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141n;
const highLimb = (1n << 192n) + 1n;

function negate(value, modulus) {
  return (modulus - value) % modulus;
}

// Extended Euclid supplies an oracle independent of the generated runtime.
function inverse(value, modulus) {
  let remainder = modulus;
  let nextRemainder = value;
  let coefficient = 0n;
  let nextCoefficient = 1n;
  while (nextRemainder !== 0n) {
    const quotient = remainder / nextRemainder;
    [remainder, nextRemainder] = [nextRemainder, remainder - quotient * nextRemainder];
    [coefficient, nextCoefficient] = [nextCoefficient, coefficient - quotient * nextCoefficient];
  }
  return (coefficient + modulus) % modulus;
}

function upstreamCases(modulus) {
  return [1n, 2n, 3n, highLimb, modulus - 1n].map((value) =>
    ['test', [value], [negate(value, modulus), inverse(value, modulus)]],
  );
}

function fieldControls(field, modulus) {
  const negation = `neg${field}`;
  const guarded = `guardedInv${field}`;
  const discard = `discardInv${field}`;
  return [
    ...[0n, 1n, highLimb, modulus - 1n].map((value) => [negation, [value], negate(value, modulus)]),
    [guarded, [false, 0n], 0n],
    [guarded, [false, 2n], 2n],
    [guarded, [true, 2n], inverse(2n, modulus)],
    [guarded, [true, highLimb], inverse(highLimb, modulus)],
    [discard, [2n], true],
  ];
}

const baseZeroError = 'secp256k1 base field has no inverse for 0';
const scalarZeroError = 'secp256k1 scalar field has no inverse for 0';

export const unaryFixtures = [
  {
    file: 'upstream-base.compact',
    positives: upstreamCases(baseModulus),
    zeroRefusals: [['test', [0n], baseZeroError]],
  },
  {
    file: 'upstream-scalar.compact',
    positives: upstreamCases(scalarModulus),
    zeroRefusals: [['test', [0n], scalarZeroError]],
  },
  {
    file: 'unary-controls.compact',
    positives: [
      ...fieldControls('Base', baseModulus),
      ...fieldControls('Scalar', scalarModulus),
      ['localOverloads', [false], [true, false]],
      ['localOverloads', [true], [false, true]],
    ],
    zeroRefusals: [
      ['guardedInvBase', [true, 0n], baseZeroError],
      ['guardedInvScalar', [true, 0n], scalarZeroError],
      ['discardInvBase', [0n], baseZeroError],
      ['discardInvScalar', [0n], scalarZeroError],
    ],
    purePositives: [['pureDiscardInvScalar', [2n], true]],
    pureZeroRefusals: [['pureDiscardInvScalar', [0n], scalarZeroError]],
  },
];
