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

import { generator, variedIdentity, xZero } from '../accessors/cases.mjs';
import { pointBytes, pointCases } from '../point/cases.mjs';

export const witnessCases = [
  { circuit: 'test0', callback: 'point0', value: xZero, initial: 10, step: 1 },
  { circuit: 'test1', callback: 'point1', value: generator, initial: 20, step: 2 },
];

export function hashCases(p256) {
  const { five, seven } = pointCases(p256);
  return [
    { args: [five, seven], selected: true },
    { args: [seven, five], selected: false },
    { args: [variedIdentity, xZero], selected: false },
  ];
}

export function pointTupleDigest(keccak, points) {
  const bytes = new Uint8Array(96 * points.length);
  for (const [index, point] of points.entries()) {
    const [x, y] = pointBytes(point);
    const start = 96 * index;
    bytes.set(x, start);
    bytes.set(y, start + 32);
    bytes[start + 64] = Number(point.identity);
  }
  return keccak(bytes);
}
