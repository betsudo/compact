// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { expect } from 'vitest';
import * as runtime from '@midnight-ntwrk/compact-runtime';
import { defineRuntimeTest } from '@test/compact-test';
import type { Contract, PureCircuits } from './.build/contract/index.js';

export default defineRuntimeTest<typeof Contract, PureCircuits>(
    import.meta.url,
    async (Contract, pure) => {
        const order = runtime.JUBJUB_SCALAR_MODULUS;
        const max = order - 1n;
        const contract = new Contract({});
        const initial = contract.initialState(
            runtime.createConstructorContext(undefined, '0'.repeat(64)),
        );
        const ctx = runtime.createCircuitContext(
            runtime.dummyContractAddress(),
            initial.currentZswapLocalState,
            initial.currentContractState,
            initial.currentPrivateState,
        );
        const stored = await contract.circuits.store(ctx, max);
        expect(stored.result).toBe(max);
        const cleared = await contract.circuits.clear(stored.context);
        expect(cleared.result).toBe(0n);
        for (const value of [0n, 1n, max]) {
            expect(pure.reduce(value)).toBe(value);
            expect(pure.point(value)).toEqual(runtime.ecMulGenerator(value));
            const g = runtime.ecMulGenerator(2n);
            expect(pure.mul(value, g)).toEqual(runtime.ecMul(g, value));
            const atom = runtime.CompactTypeField.toValue(value);
            expect(runtime.CompactTypeJubjubScalar.toValue(value)).toEqual(atom);
            expect(runtime.CompactTypeJubjubScalar.fromValue(atom)).toBe(value);
        }
        expect(pure.reduce(order)).toBe(0n);
        expect(pure.reduce(order + 1n)).toBe(1n);
        expect(pure.reduceUint((1n << 64n) - 1n)).toBe((1n << 64n) - 1n);
        expect(() => pure.point(order)).toThrow(/JubjubScalar/);
        expect(() => runtime.CompactTypeJubjubScalar.toValue(order)).toThrow(/JubjubScalar/);
        expect(() =>
            runtime.CompactTypeJubjubScalar.fromValue([Uint8Array.of(1, 0)]),
        ).toThrow(/JubjubScalar/);
        expect(() =>
            runtime.CompactTypeJubjubScalar.fromValue(runtime.CompactTypeField.toValue(order)),
        ).toThrow(/JubjubScalar/);
    },
);
