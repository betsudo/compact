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

import { describe, expect, test } from 'vitest';
import { CompactTypeOpaqueString } from '../src/index.js';

describe('opaque strings', () => {
  test.each(['hello', '', '\uFEFFhello'])('round trips Opaque<"string"> %j', (text) => {
    expect(CompactTypeOpaqueString.fromValue(CompactTypeOpaqueString.toValue(text))).toBe(text);
  });

  test.each([
    ['a leading U+FEFF', [0xef, 0xbb, 0xbf, 0x68, 0x69]],
    ['two leading U+FEFFs', [0xef, 0xbb, 0xbf, 0xef, 0xbb, 0xbf, 0x68, 0x69]],
    ['nothing but U+FEFF', [0xef, 0xbb, 0xbf]],
  ])('Opaque<"string"> decoded from bytes with %s encodes back to those bytes', (_label, bytes) => {
    const stored = new Uint8Array(bytes);
    const decoded = CompactTypeOpaqueString.fromValue([stored]);
    expect(CompactTypeOpaqueString.toValue(decoded)).toStrictEqual([stored]);
  });
});
