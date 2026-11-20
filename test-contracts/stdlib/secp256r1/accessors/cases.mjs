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

import { identity } from '../point/cases.mjs';

export const generator = {
  x: 0x6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296n,
  y: 0x4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5n,
  identity: false,
};
export const xZero = {
  x: 0n, y: 0x66485c780e2f83d72433bd5d84a06bb6541c2af31dae871728bf856a174f93f4n,
  identity: false,
};
export const variedIdentity = { x: 3n, y: 4n, identity: true };
export const accessors = [
  { fixture: 'upstream-roundtrip', folder: 'point/upstream-roundtrip', calls: [
    { circuit: 'test0', args: [xZero], expected: xZero, slot: 'point', selected: true },
    { circuit: 'test0', args: [generator], expected: generator, slot: 'point' },
    { circuit: 'test1', args: [], seed: xZero, expected: [xZero.x, xZero.y], selected: true },
    { circuit: 'test1', args: [], seed: generator, expected: [generator.x, generator.y] },
  ] },
  { fixture: 'upstream-ledger', folder: 'accessors/upstream-ledger', calls: [
    { circuit: 'storePoint', args: [generator], expected: [], slot: 'pt', selected: true },
    { circuit: 'storePoint', args: [variedIdentity], expected: [], slot: 'pt' },
    { circuit: 'storeX', args: [], seed: generator, expected: [], slot: 'x', written: generator.x, selected: true },
    { circuit: 'storeY', args: [], seed: generator, expected: [], slot: 'y', written: generator.y, selected: true },
  ] },
  { fixture: 'upstream-asserted', folder: 'accessors/upstream-asserted', calls: [
    { circuit: 'storePoint', args: [generator], expected: [], slot: 'pt', selected: true },
    { circuit: 'storeXChecked', args: [], seed: generator, expected: [], slot: 'x', written: generator.x, selected: true },
  ] },
  { fixture: 'guarded-writes', folder: 'accessors/guarded-writes', calls: [
    { circuit: 'guardedX', args: [false, identity], expected: 0n, slot: 'x', selected: true },
    { circuit: 'guardedX', args: [false, variedIdentity], expected: 0n, slot: 'x' },
    { circuit: 'guardedX', args: [true, generator], expected: generator.x, slot: 'x', selected: true },
    { circuit: 'guardedX', args: [true, xZero], expected: xZero.x, slot: 'x' },
    { circuit: 'guardedY', args: [false, identity], expected: 0n, slot: 'y', selected: true },
    { circuit: 'guardedY', args: [false, variedIdentity], expected: 0n, slot: 'y' },
    { circuit: 'guardedY', args: [true, generator], expected: generator.y, slot: 'y', selected: true },
    { circuit: 'guardedY', args: [true, xZero], expected: xZero.y, slot: 'y' },
  ] },
];
