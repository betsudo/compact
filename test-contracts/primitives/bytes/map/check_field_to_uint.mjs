// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';

const [entry] = process.argv.slice(2);
assert.ok(entry);
const { pureCircuits } = await import(pathToFileURL(entry).href);
assert.equal(pureCircuits.field_to_byte(0n), 0n);
assert.equal(pureCircuits.field_to_byte(255n), 255n);
assert.throws(() => pureCircuits.field_to_byte(256n), /greater than 255/);
console.log('Field to Uint<8> boundary and overflow TypeScript passed');
