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
  throw new Error('usage: node capture-election-oracle.mjs <generated contract/index.js>');
}
const generated = resolve(contractPath);
const runtime = await import(pathToFileURL(createRequire(generated).resolve('@midnight-ntwrk/compact-runtime')));
const { Contract } = await import(pathToFileURL(generated));

// The original owner-operation capture uses one owner, one voter, and a yes vote.
const secret = new Uint8Array(32).fill(7);
const domain = new Uint8Array(32);
domain.set(new TextEncoder().encode('lares:election:pk:'));
const bytes32 = new runtime.CompactTypeBytes(32);
const authority = runtime.persistentHash(new runtime.CompactTypeVector(2, bytes32), [domain, secret]);
const coinKey = { bytes: new Uint8Array(32) };
const pathType = new runtime.CompactTypeMerkleTreePath(10, bytes32);
const hex = value => Buffer.from(value).toString('hex');
let witnessState;
const paths = {};

function pathWitness(slot, label, { privateState }, leaf) {
  const tree = witnessState.asArray()[slot].asArray()[0].asBoundedMerkleTree();
  const entry = tree.findPathForLeaf({ value: bytes32.toValue(leaf), alignment: bytes32.alignment() });
  if (entry === undefined) {
    const path = Array.from({ length: 10 }, () => ({ sibling: { field: 0n }, goes_left: false }));
    return [privateState, { is_some: false, value: { leaf, path } }];
  }
  const value = pathType.fromValue(entry.value);
  assert.deepEqual(value.leaf, leaf);
  assert.equal(value.path.length, 10);
  paths[label] = {
    leafHex: hex(value.leaf),
    path: value.path.map(({ sibling, goes_left }) => ({
      siblingHex: sibling.field.toString(16).padStart(64, '0'),
      goesLeft: goes_left,
    })),
  };
  return [privateState, { is_some: true, value }];
}

const contract = new Contract({
  'private$secret_key': ({ privateState }) => [privateState, secret],
  'private$state': ({ privateState }) => [privateState, privateState.phase],
  'private$state$advance': ({ privateState }) => [{ ...privateState, phase: privateState.phase + 1 }, []],
  'private$vote$record': ({ privateState }, vote) => [{ ...privateState, vote }, []],
  'private$vote': ({ privateState }) => [privateState, privateState.vote],
  'context$eligible_voters$path_of': (ctx, leaf) => pathWitness(6, 'votePathEligible', ctx, leaf),
  'context$committed_votes$path_of': (ctx, leaf) => pathWitness(5, 'votePathCommitted', ctx, leaf),
});

const initial = () => contract.initialState({
  initialPrivateState: { phase: 0, vote: 0 },
  initialZswapLocalState: runtime.emptyZswapLocalState(coinKey),
}, authority);
const snapshot = state => ({ stateHex: hex(state.serialize()) });
function invoke(state, method, ...args) {
  const ctx = runtime.createCircuitContext(runtime.dummyContractAddress(), coinKey,
    state.currentContractState.data, state.currentPrivateState);
  witnessState = ctx.currentQueryContext.state.state;
  const { context } = contract.circuits[method](ctx, ...args);
  state.currentContractState.data = new runtime.ChargedState(context.currentQueryContext.state.state);
  state.currentPrivateState = context.currentPrivateState;
  return snapshot(state.currentContractState);
}

const owner = initial();
const afterInit = snapshot(owner.currentContractState);
const afterSetTopic = invoke(owner, 'set_topic', 'hello');
const afterAdvance = invoke(owner, 'advance');
const afterAddVoter = invoke(initial(), 'add_voter', authority);
const election = initial();
invoke(election, 'set_topic', 'hello');
invoke(election, 'add_voter', authority);
invoke(election, 'advance');
const afterVoteCommit = invoke(election, 'vote$commit', 0);
invoke(election, 'advance');
const afterVoteReveal = invoke(election, 'vote$reveal');
assert.ok(paths.votePathEligible && paths.votePathCommitted);
process.stdout.write(JSON.stringify({
  afterInit, afterSetTopic, afterAdvance, afterAddVoter, afterVoteCommit,
  votePathEligible: paths.votePathEligible, afterVoteReveal, votePathCommitted: paths.votePathCommitted,
}, null, 2) + '\n');
