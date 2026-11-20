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

import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

import { expect, test } from 'vitest';

import type {
    CompileTestDefinition, RuntimeTestDefinition, CompactContractConstructor,
} from '../../../types.ts';

const output = process.env.KECCAK_CORPUS_OUTPUT!;
const consumer = process.env.KECCAK_CORPUS_CONSUMER!;

const compileTests = import.meta.glob<CompileTestDefinition>('../*/compile.*.test.ts', {
    eager: true, import: 'default',
});
const runtimeTests = import.meta.glob<RuntimeTestDefinition<CompactContractConstructor>>('../*/runtime.*.test.ts', {
    eager: true, import: 'default',
});

function fixture(path: string): string {
    return path.split('/')[1];
}

for (const [path, definition] of Object.entries(compileTests)) {
    const family = fixture(path);
    test(`${family} compile ${definition.result}`, () => {
        const directory = join(output, family);
        const build = join(directory, '.build');
        const code = Number(readFileSync(join(directory, 'compile.exit'), 'utf8'));
        const stderr = readFileSync(join(directory, 'compile.stderr'), 'utf8');
        if (definition.result === 'fail') {
            expect(code).not.toBe(0);
            expect(definition.options.expectedError).toBeInstanceOf(RegExp);
            expect(stderr).toMatch(definition.options.expectedError as RegExp);
            expect(existsSync(build)).toBe(false);
            return;
        }
        expect(code, stderr).toBe(0);
        expect(existsSync(join(build, 'contract/index.js'))).toBe(true);
        expect(existsSync(join(build, 'contract/lib.rs'))).toBe(true);
        if (family === 'impure') {
            const raw = join(build, 'zkir/keccak256_writes_ledger.zkir');
            expect(JSON.parse(readFileSync(raw, 'utf8')).instructions).toEqual(
                expect.arrayContaining([expect.objectContaining({ op: 'keccak256' })]),
            );
        }
    });
}

function invokeRust(family: string, name: string, input: unknown): Uint8Array {
    const result = execFileSync(consumer, [family, name], {
        encoding: 'utf8',
        input: JSON.stringify(input, (_key, value) => {
            if (typeof value === 'bigint') {
                return value.toString(16).padStart(64, '0');
            }
            return value instanceof Uint8Array ? Array.from(value) : value;
        }),
    }).trim();
    expect(result).toMatch(/^[0-9a-f]{64}$/);
    return Uint8Array.from(Buffer.from(result, 'hex'));
}

for (const [path, definition] of Object.entries(runtimeTests)) {
    const family = fixture(path);
    const moduleUrl = pathToFileURL(join(output, family, '.build/contract/index.js')).href;
    test(`${family} TypeScript runtime`, async () => {
        expect(definition.result).toBe('pass');
        const { Contract, pureCircuits } = await import(/* @vite-ignore */ moduleUrl);
        await definition.run(Contract, pureCircuits);
    });

    test(`${family} Rust runtime`, async () => {
        expect(definition.result).toBe('pass');
        const { Contract, pureCircuits } = await import(/* @vite-ignore */ moduleUrl);
        const rustCircuits = Object.fromEntries(Object.keys(pureCircuits).map((name) =>
            [name, (value: unknown) => invokeRust(family, name, value)],
        ));
        await definition.run(Contract, rustCircuits);
    });
}
