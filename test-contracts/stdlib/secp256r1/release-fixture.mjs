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

import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { expect } from 'vitest';

export function compileReleaseFixture(compile, fixture, filename, lane, options) {
  const targets = ['ts', 'rust'];
  const intact = compile(fixture, options, targets);
  const originalPath = new URL(fixture + '/' + filename, import.meta.url);
  let source = fileURLToPath(originalPath);
  if (intact.result.status !== 0) {
    expect(process.env.COMPACT_SECP256R1_KAT_MODE).toBe('--allow-pragma-probe');
    expect(intact.result.stderr).toContain('language version 0.23.105 mismatch');
    expect(existsSync(intact.build)).toBe(false);
    const original = readFileSync(originalPath, 'utf8');
    const pragma = 'pragma language_version >= 0.25.0;\n';
    expect(original.split(pragma)).toHaveLength(2);
    source = join(dirname(intact.build), filename);
    writeFileSync(source, original.replace(pragma, ''), { flag: 'wx' });
  }
  const built = compile(lane, options, targets, '--feature-zkir-v3', source);
  expect(built.result.status, built.result.stderr).toBe(0);
  return built;
}
