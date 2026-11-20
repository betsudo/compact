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
import { identity } from '../point/cases.mjs';

export const maxScalar = 0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632550n;
export const negG = {
  x: generator.x,
  y: 0xffffffff00000001000000000000000000000000ffffffffffffffffffffffffn - generator.y,
  identity: false,
};
export const twoG = {
  x: 0x7cf27b188d034f7e8a52380304b51ac3c08969e277f21b35a60b48fc47669978n,
  y: 0x07775510db8ed040293d9ac69f7430dbba7dade63ce982299e04b79d227873d1n,
  identity: false,
};

export const group = [
  { fixture: 'upstream-computed', folder: 'accessors/upstream-computed', calls: [
    { circuit: 'storePoint', args: [generator], expected: [], slot: 'pt', selected: true },
    { circuit: 'storeXOfGenerator', args: [1n], expected: [], slot: 'x', written: generator.x,
      identityArgs: [0n], selected: true },
    { circuit: 'storeXOfGenerator', args: [maxScalar], expected: [], slot: 'x', written: negG.x },
    { circuit: 'storeXOfMul', args: [2n], seed: generator, expected: [], slot: 'x', written: twoG.x,
      identityArgs: [0n], selected: true },
    { circuit: 'storeXOfMul', args: [1n], seed: xZero, expected: [], slot: 'x', written: 0n },
    { circuit: 'storeXOfSum', args: [generator, generator], expected: [], slot: 'x', written: twoG.x,
      identityArgs: [generator, negG], selected: true },
    { circuit: 'storeXOfSum', args: [generator, variedIdentity], expected: [], slot: 'x', written: generator.x },
  ] },
  { fixture: 'guarded-group', folder: 'group/guarded', calls: [
    { circuit: 'guardedAdd', args: [false, generator, generator], expected: identity, slot: 'pt', selected: true },
    { circuit: 'guardedAdd', args: [true, generator, generator], expected: twoG, slot: 'pt', selected: true },
    { circuit: 'guardedAdd', args: [true, generator, negG], expected: identity, slot: 'pt' },
    { circuit: 'guardedAdd', args: [true, variedIdentity, xZero], expected: xZero, slot: 'pt' },
    { circuit: 'guardedMul', args: [false, generator, 2n], expected: identity, slot: 'pt', selected: true },
    { circuit: 'guardedMul', args: [true, generator, 2n], expected: twoG, slot: 'pt', selected: true },
    { circuit: 'guardedMul', args: [true, generator, 0n], expected: identity, slot: 'pt' },
    { circuit: 'guardedMul', args: [true, generator, maxScalar], expected: negG, slot: 'pt' },
    { circuit: 'guardedGenerator', args: [false, 1n], expected: identity, slot: 'pt', selected: true },
    { circuit: 'guardedGenerator', args: [true, 1n], expected: generator, slot: 'pt', selected: true },
    { circuit: 'guardedGenerator', args: [true, 0n], expected: identity, slot: 'pt' },
    { circuit: 'guardedGenerator', args: [true, maxScalar], expected: negG, slot: 'pt' },
  ] },
];
