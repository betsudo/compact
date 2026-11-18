// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/** Shared private encoding for host and internal call coin commitments. */
import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import { ShieldedCoinInfoDescriptor, ShieldedCoinRecipientDescriptor } from './compact-types.js';
import type { EncodedRecipient, EncodedShieldedCoinInfo } from './zswap.js';

/** The exact Compact standard-library layout consumed by the Ledger SDK. */
export const createCoinCommitment = (coinInfo: EncodedShieldedCoinInfo, recipient: EncodedRecipient): ocrt.AlignedValue =>
  ocrt.runtimeCoinCommitment(
    {
      value: ShieldedCoinInfoDescriptor.toValue(coinInfo),
      alignment: ShieldedCoinInfoDescriptor.alignment(),
    },
    {
      value: ShieldedCoinRecipientDescriptor.toValue(recipient),
      alignment: ShieldedCoinRecipientDescriptor.alignment(),
    },
  );
