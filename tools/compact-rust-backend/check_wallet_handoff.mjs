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

// Check the Ledger 9 binary boundary against the pinned JavaScript ledger.

import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const arguments_ = process.argv.slice(2);
if (arguments_.length !== 2 && arguments_.length !== 3) {
  throw new Error('usage: node check_wallet_handoff.mjs [sealed-deploy.bin] <sealed-call.bin> <ledger-v9-package-dir>');
}
const [deployPath, transactionPath, packageRoot] = arguments_.length === 3
  ? arguments_
  : [undefined, ...arguments_];

const packageDirectory = resolve(packageRoot);
const manifest = JSON.parse(await readFile(resolve(packageDirectory, 'package.json'), 'utf8'));
if (manifest.name !== '@midnight-ntwrk/ledger-v9' || manifest.version !== '1.0.1') {
  throw new Error(`expected @midnight-ntwrk/ledger-v9@1.0.1, got ${manifest.name}@${manifest.version}`);
}

const ledger = await import(pathToFileURL(resolve(packageDirectory, 'midnight_ledger_wasm_v9_fs.js')).href);
const contractSegment = 1;
// Ledger 9's local fee balancing helper puts Dust spends in segment 0xFEED.
const feeSegment = 0xFEED;

async function readHandoff(path, Action, label) {
  const bytes = await readFile(path);
  const transaction = ledger.Transaction.deserialize('signature', 'proof', 'binding', bytes);
  const intents = transaction.intents;
  const contract = intents?.get(contractSegment);
  const fee = intents?.get(feeSegment);
  const dust = fee?.dustActions;
  const action = contract?.actions[0];
  if (intents?.size !== 2 || contract?.actions.length !== 1 || !(action instanceof Action)) {
    throw new Error(`expected one ${label} in contract segment ${contractSegment}`);
  }
  if (fee?.actions.length !== 0 || dust?.spends.length !== 1 ||
      dust.registrations.length !== 0 || dust.spends[0].vFee <= 0n ||
      !(dust.spends[0].proof instanceof ledger.Proof)) {
    throw new Error(`expected one proved positive-fee Dust spend for ${label}`);
  }
  if (!Buffer.from(transaction.serialize()).equals(bytes)) {
    throw new Error(`Ledger 9 changed sealed ${label} bytes after deserialization`);
  }
  return { action, length: bytes.length };
}

const call = await readHandoff(transactionPath, ledger.ContractCall, 'call');
if (!(call.action.proof instanceof ledger.Proof)) {
  throw new Error('expected a proved contract call');
}
if (deployPath) {
  const deploy = await readHandoff(deployPath, ledger.ContractDeploy, 'deployment');
  const deployAddress = deploy.action.address;
  const callAddress = call.action.address;
  if (deployAddress !== callAddress) {
    throw new Error(`deployment and call addresses differ: ${deployAddress} != ${callAddress}`);
  }
  console.log(`ledger-v9 1.0.1 round-tripped ${deploy.length} funded deployment and ${call.length} funded call bytes at ${deployAddress}`);
} else {
  console.log(`ledger-v9 1.0.1 round-tripped ${call.length} funded call bytes`);
}
