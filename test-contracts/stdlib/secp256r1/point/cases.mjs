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

export const modulus = 0xffffffff00000001000000000000000000000000ffffffffffffffffffffffffn;
export const identity = { x: 0n, y: 0n, identity: true };

export function pointCases(p256) {
  const generator = (scalar) => {
    const { x, y } = p256.Point.BASE.multiplyUnsafe(scalar).toAffine();
    return { x, y, identity: x === 0n && y === 0n };
  };
  const five = generator(5n);
  const seven = generator(7n);
  const variedIdentities = [identity, { x: 4n, y: 4n, identity: true }, { ...seven, identity: true }];
  return {
    generator, five, seven, variedIdentities,
    equality: [
      ['pointsEqual', [five, { ...five }], true],
      ['pointsEqual', [five, seven], false],
      ['pointsNotEqual', [five, { ...five }], false],
      ['pointsNotEqual', [five, seven], true],
      ['pointsEqual', [identity, generator(0n)], true],
      ['pointsEqual', [five, identity], false],
      ['pointsNotEqual', [identity, generator(0n)], false],
      ['pointsNotEqual', [five, identity], true],
    ],
    malformed: [
      null,
      { ...five, identity: 0 },
      { ...five, x: modulus },
      { ...identity, x: -1n },
      { ...identity, y: modulus },
      { x: 0n, y: 0n, identity: false },
      { ...five, y: (five.y + 1n) % modulus },
    ],
  };
}
