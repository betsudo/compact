// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [contractPath] = process.argv.slice(2);
if (!contractPath) {
  throw new Error('usage: node capture-zerocash-oracle.mjs <generated contract/index.js>');
}
const generated = resolve(contractPath);
const runtime = await import(pathToFileURL(createRequire(generated).resolve('@midnight-ntwrk/compact-runtime')));
const { Contract } = await import(pathToFileURL(generated));

const filled = byte => new Uint8Array(32).fill(byte);
const coin = { nonce: { bytes: filled(3) }, opening: { bytes: filled(4) } };
const secretKey = { bytes: filled(1) };
const bytes32 = new runtime.CompactTypeBytes(32);
const sourceKey = { bytes: runtime.persistentHash(bytes32, secretKey.bytes) };
const destination = { zk: { bytes: filled(5) }, encryption: filled(6) };
const pathType = new runtime.CompactTypeMerkleTreePath(32, bytes32);
const hex = value => Buffer.from(value).toString('hex');
let mintedState;
let spendPath;
const contract = new Contract({
  'private$zk_secret_key': ({ privateState }) => [privateState, secretKey],
  'private$zk_public_key': ({ privateState }) => [privateState, sourceKey],
  'private$add_coin': ({ privateState }, value) => {
    assert.deepEqual(value, coin);
    return [privateState, []];
  },
  'private$remove_coin': ({ privateState }, value) => {
    assert.deepEqual(value, coin);
    return [privateState, []];
  },
  'context$new_coin_info': ({ privateState }) => [privateState, coin],
  'context$path_of': ({ privateState }, commitment) => {
    const tree = mintedState.asArray()[1].asArray()[0].asBoundedMerkleTree();
    const leaf = { value: bytes32.toValue(commitment.bytes), alignment: bytes32.alignment() };
    const path = pathType.fromValue(tree.findPathForLeaf(leaf).value);
    assert.equal(path.path.length, 32);
    assert.deepEqual(path.leaf, commitment.bytes);
    spendPath = {
      leafHex: hex(path.leaf),
      path: path.path.map(({ sibling, goes_left }) => ({
        siblingHex: sibling.field.toString(16).padStart(64, '0'),
        goesLeft: goes_left,
      })),
    };
    return [privateState, { ...path, leaf: { bytes: path.leaf } }];
  },
  'context$encrypt': ({ privateState }, key, value) => {
    assert.deepEqual(key, destination.encryption);
    assert.deepEqual(value, coin);
    return [privateState, Uint8Array.of(10, 11, 12)];
  },
});

const key = { bytes: new Uint8Array(32) };
const initial = contract.initialState({
  initialPrivateState: null,
  initialZswapLocalState: runtime.emptyZswapLocalState(key),
});
const state = initial.currentContractState;
const snapshot = () => ({ stateHex: hex(state.serialize()) });
const context = privateState => runtime.createCircuitContext(runtime.dummyContractAddress(), key, state.data, privateState);
const afterInit = snapshot();
const mint = contract.circuits.zerocash_mint(context(initial.currentPrivateState));
mintedState = mint.context.currentQueryContext.state.state;
state.data = new runtime.ChargedState(mintedState);
const afterMint = snapshot();
const spend = contract.circuits.spend(context(mint.context.currentPrivateState), destination, coin);
state.data = new runtime.ChargedState(spend.context.currentQueryContext.state.state);
assert.ok(spendPath);
process.stdout.write(JSON.stringify({ afterInit, afterMint, afterSpend: snapshot(), spendPath }, null, 2) + '\n');
