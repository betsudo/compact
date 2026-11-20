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

// Capture the constructor and pure exports of tuple_destructure.compact.
import { createRequire } from 'node:module';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

if (process.argv.length !== 3) throw new Error('usage: node tuple_destructure_capture.mjs <compiled-contract-dir>');
const contractIndex = resolve(process.argv[2], 'index.js');
const requireFromContract = createRequire(contractIndex);
const runtime = await import(pathToFileURL(requireFromContract.resolve('@midnight-ntwrk/compact-runtime')));
const { Contract, pureCircuits } = await import(pathToFileURL(contractIndex));
const contract = new Contract({});
const initial = contract.initialState({
  initialPrivateState: null,
  initialZswapLocalState: runtime.emptyZswapLocalState({ bytes: new Uint8Array(32) }),
});

const field = 7n;
const small = 255n;
const fixture = {
  initialHex: Buffer.from(initial.currentContractState.serialize()).toString('hex'),
  asVector: pureCircuits.as_vector(field),
  asTuple: pureCircuits.as_tuple(field),
  hetero: pureCircuits.hetero(small),
  oneTuple: pureCircuits.one_tuple(field),
  emptyTuple: pureCircuits.empty_tuple(),
  tupleCoerce: pureCircuits.tuple_coerce(small),
  tupleVarRef: pureCircuits.tuple_var_ref(small),
  tupleToVector: pureCircuits.tuple_to_vector(field),
  structVectorReturn: pureCircuits.struct_vector_return(field),
  structVectorToTuple: pureCircuits.struct_vector_to_tuple(field),
  tupleFirst: pureCircuits.tuple_first(field, true),
  tupleSecond: pureCircuits.tuple_second(field, true),
};
process.stdout.write(JSON.stringify(
  fixture, (_, value) => typeof value === 'bigint' ? value.toString() : value, 2,
) + '\n');
