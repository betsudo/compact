// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
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

// Evaluate only the generated verifier table from the derivative __proto__
// circuit. This checks JavaScript own-key behavior, beyond a textual fragment.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

assert.equal(process.argv.length, 3);
const source = readFileSync(process.argv[2], 'utf8');
const match = source.match(/export const expectedVk = (\{[^}]*\});/);
assert.ok(match, 'expectedVk table must be present');
const table = Function(`return (${match[1]})`)();
assert.equal(Object.getPrototypeOf(table), null);
assert.equal(Object.hasOwn(table, '__proto__'), true);
assert.match(table.__proto__, /^[0-9a-f]{64}$/);
console.log('expectedVk __proto__ is an own key on a null-prototype table');
