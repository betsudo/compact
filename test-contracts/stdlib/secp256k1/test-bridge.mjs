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

import {
  createCircuitContext,
  createConstructorContext,
  dummyContractAddress,
} from '@midnight-ntwrk/compact-runtime';

export { defineCompileTest, defineRuntimeTest } from '../../compact-test.ts';

// Use the original constructor inputs and the synchronous runtime's positional
// context API. The roundtrip fixture does not observe the block time.
export async function createTestContract(Contract, witnesses = {}, privateState = undefined, constructorArgs = []) {
  const contract = new Contract(witnesses);
  const initial = await contract.initialState(createConstructorContext(privateState, '0'.repeat(64)), ...constructorArgs);
  const ctx = createCircuitContext(
    dummyContractAddress(),
    initial.currentZswapLocalState.coinPublicKey,
    initial.currentContractState,
    initial.currentPrivateState,
    undefined,
    undefined,
    0,
  );
  return { contract, ctx };
}
