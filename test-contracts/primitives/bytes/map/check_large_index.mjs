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
const bytes = new Uint8Array(4096);
bytes[4095] = 17;
assert.equal(pureCircuits.read_last(bytes), 17n);
console.log('Bytes<4096> constant index TypeScript passed');
