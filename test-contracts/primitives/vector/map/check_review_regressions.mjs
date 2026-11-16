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
const leftBytes = Uint8Array.from([1, 2]);
const rightBytes = Uint8Array.from([10, 20]);
assert.deepEqual(pureCircuits.bytes_map(leftBytes, rightBytes), [12n, 24n]);
assert.deepEqual(pureCircuits.widened_bytes_map(leftBytes, rightBytes), [1n, 2n]);
assert.deepEqual(pureCircuits.widened_vector_map([1n, 2n], [10n, 20n]), [1n, 2n]);
console.log('binary map Bytes operands and widened formals TypeScript passed');
