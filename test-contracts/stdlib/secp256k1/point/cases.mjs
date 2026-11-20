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

export const baseModulus = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2fn;

export const identity = { x: 0n, y: 0n, identity: true };
export const generator = {
  x: 0x79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798n,
  y: 0x483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8n,
  identity: false,
};
export const five = {
  x: 0x2f8bde4d1a07209355b4a7250a5c5128e88b84bddc619ab7cba8d569b240efe4n,
  y: 0xd8ac222636e5e3d6d4dba9dda6c9c426f788271bab0d6840dca87d3aa6ac62d6n,
  identity: false,
};
export const seven = {
  x: 0x5cbdf0646e5db4eaa398f365f2ea7a0e3d419b7e0330e39ce92bddedcac4f9bcn,
  y: 0x6aebca40ba255960a3178d6d861a54dba813d0b813fde7b5a5082628087264dan,
  identity: false,
};
export const identityWithSmallCoordinates = { x: 4n, y: 4n, identity: true };
export const identityWithCurveCoordinates = { ...seven, identity: true };
export const identityRepresentations = [identity, identityWithSmallCoordinates, identityWithCurveCoordinates];

// A string denotes the actual result of getDefault in the first fixture.
export const equalityCases = [
  ['pointsEqual', [five, { ...five }], true],
  ['pointsEqual', [five, seven], false],
  ['pointsNotEqual', [five, { ...five }], false],
  ['pointsNotEqual', [five, seven], true],
  ['pointsEqual', ['default', identity], true],
  ['pointsEqual', [five, 'default'], false],
  ['pointsNotEqual', ['default', identity], false],
  ['pointsNotEqual', [five, 'default'], true],
];

export const ignoredCoordinateCases = [
  ['pointsEqual', [identity, identityWithSmallCoordinates], true],
  ['pointsEqual', [identityWithSmallCoordinates, identityWithCurveCoordinates], true],
  ['pointsNotEqual', [identityWithCurveCoordinates, identity], false],
  ['pointsEqual', [seven, identityWithCurveCoordinates], false],
];

export const ledgerSteps = [
  ['readPoint', [], identity],
  ['storePoint', [five], five],
  ['readPoint', [], five],
  ['storePoint', [seven], seven],
  ['readPoint', [], seven],
  ['resetPoint', [], identity],
  ['readPoint', [], identity],
  ['storePoint', [identity], identity],
  ['storePoint', [identityWithSmallCoordinates], identity],
  ['storePoint', [identityWithCurveCoordinates], identity],
  ['readPoint', [], identity],
];

export const malformedPoints = [
  ['null', null],
  ['missing coordinate', { x: five.x, identity: false }],
  ['number coordinate', { ...identity, x: 0 }],
  ['non-boolean flag', { ...five, identity: 'false' }],
  ['zero point marked finite', { ...identity, identity: false }],
  ['off-curve finite point', { x: 1n, y: 1n, identity: false }],
  ['x at modulus', { ...five, x: baseModulus }],
  ['y at modulus', { ...five, y: baseModulus }],
  ['negative x', { ...five, x: -1n }],
  ['identity x at modulus', { ...identity, x: baseModulus }],
  ['negative identity y', { ...identity, y: -1n }],
];


// Independent OpenSSL digests of the two concatenated 96-byte point representations.
const identityKeccak256 = '44da15b3f436b9ef4608b8e479337c183baca088058cd900a05c659b9a4eeea3';
const identitySha512 = '7984292cc64d92bbd1c4e822b5ef1972cec3be203a134af34b63aeb8c386cbc790bbfe2effef2c50ed22fd60c8f32028dc46a2baaad284a568e42e8b8ff1bb8d';

export const pointHashCases = [
  {
    label: 'upstream 5G, 7G',
    args: [five, seven],
    keccak256: '9d2c524d66d23d1cfc4855bbd5705e332ab09f9d935231d9275dc5bee1a17896',
    sha512: 'e61dc96ae78029d885defb24046795affcd2538a52988f5b1fae3fd26a912385db1086c61042a27272b3df7ebdb03d0f0426b4d6204871f040a7738e1ec7b347',
  },
  {
    label: 'reversed point order',
    args: [seven, five],
    keccak256: 'be3554c11bbf8e9c0ff2df0613427517b760485b96bc0aa13dcc5979c87b3c04',
    sha512: 'b6eb76318345254ed9dd62c284d2932c25f9e1e3097119af718f41ac6e471d3617e229b65edb31bafb02b9b3c63bc5350f57699248a4ea77d4a38fa09010734e',
  },
  {
    label: 'finite point followed by identity',
    args: [five, identity],
    keccak256: identityKeccak256,
    sha512: identitySha512,
  },
  {
    label: 'finite point followed by identity with ignored coordinates',
    args: [five, identityWithCurveCoordinates],
    keccak256: identityKeccak256,
    sha512: identitySha512,
  },
];
