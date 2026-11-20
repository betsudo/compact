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

export const point = {
  x: 0xd06cae31b20a8c528186917358a5eceac665029d8afc30eee8e3abaa5a24e9ean,
  y: 0x9cd8a23f97228ae2f1cb89cc93530783ac6970f2af443e5a87da50c835984f06n,
  identity: false,
};

export const expected = new Uint8Array([
  0x3d, 0x81, 0xac, 0x81, 0x76, 0x4e, 0xb1, 0xa4, 0x4a, 0xfb,
  0xc2, 0x45, 0xfc, 0x1d, 0x92, 0x54, 0xc0, 0xd0, 0x77, 0x2a,
]);

export const identityError =
  'Secp256k1Point identity is not a permitted input to secp256k1EthereumAddress';

export function coordinateBytes(value) {
  return Buffer.from(value.toString(16).padStart(64, '0'), 'hex');
}
