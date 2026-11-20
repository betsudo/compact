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
const baseMax = baseModulus - 1n;
const scalarMax = scalarModulus - 1n;
const base0 = 46650258000037232366158629642678773672890852140699407265360345715536022899553n;
const base1 = 49961613701950065613888640415520526451680684154063724245984978230985445785591n;
const scalar0 = 13185332814719467048625289428055336956216913103903703707915286873163564687413n;
const scalar1 = 96181917743701203273367767841317204335168468443275911828686980079872884046249n;

function modulo(value, modulus) {
  return ((value % modulus) + modulus) % modulus;
}

export const fieldCases = [
  ['addb', [base0, 0n], base0],
  ['addb', [base1, 0n], base1],
  ['addb', [base0, baseMax], base0 - 1n],
  ['addb', [base1, baseMax], base1 - 1n],
  ['addb', [base0, base1], modulo(base0 + base1, baseModulus)],
  ['subb', [base0, 0n], base0],
  ['subb', [base1, 0n], base1],
  ['subb', [base0, baseMax], base0 + 1n],
  ['subb', [base1, baseMax], base1 + 1n],
  ['subb', [base0, base1], modulo(base0 - base1, baseModulus)],
  ['mulb', [base0, 0n], 0n],
  ['mulb', [base1, 0n], 0n],
  ['mulb', [base0, 1n], base0],
  ['mulb', [base1, 1n], base1],
  ['mulb', [base0, baseMax], modulo(base0 * baseMax, baseModulus)],
  ['mulb', [base1, baseMax], modulo(base1 * baseMax, baseModulus)],
  ['mulb', [base0, base1], modulo(base0 * base1, baseModulus)],
  ['adds', [scalar0, 0n], scalar0],
  ['adds', [scalar1, 0n], scalar1],
  ['adds', [scalar0, scalarMax], scalar0 - 1n],
  ['adds', [scalar1, scalarMax], scalar1 - 1n],
  ['adds', [scalar0, scalar1], modulo(scalar0 + scalar1, scalarModulus)],
  ['subs', [scalar0, 0n], scalar0],
  ['subs', [scalar1, 0n], scalar1],
  ['subs', [scalar0, scalarMax], scalar0 + 1n],
  ['subs', [scalar1, scalarMax], scalar1 + 1n],
  ['subs', [scalar0, scalar1], modulo(scalar0 - scalar1, scalarModulus)],
  ['muls', [scalar0, 0n], 0n],
  ['muls', [scalar1, 0n], 0n],
  ['muls', [scalar0, 1n], scalar0],
  ['muls', [scalar1, 1n], scalar1],
  ['muls', [scalar0, scalarMax], modulo(scalar0 * scalarMax, scalarModulus)],
  ['muls', [scalar1, scalarMax], modulo(scalar1 * scalarMax, scalarModulus)],
  ['muls', [scalar0, scalar1], modulo(scalar0 * scalar1, scalarModulus)],
];

export const selfSubtractionCases = [
  ['selfSub', [12345n], 0n],
  ['inlinedSelfSub', [12345n], 0n],
  ['mulSelfSub', [12345n, 678n], 0n],
];
