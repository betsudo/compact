#!/usr/bin/env bash

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#  	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# Bounded real-proof qualification of the native artifact adapter lifecycle.
# The complete native gate is check_native_ci.sh; this is never a substitute.
set -e -o pipefail

exec python3 -B tools/compact-rust-backend/check_forge_circuit_cache.py \
  --compiler "$COMPACTC" \
  --proof-runner "$COMPACT_RUST_PROOF_RUNNER" \
  --forge "$COMPACT_ZKIR_FORGE_BIN" \
  --params-dir "$MIDNIGHT_PP" \
  --cache-dir "$MIDNIGHT_ZKIR_FORGE_DIR" \
  --report "$(dirname "$COMPACT_ZKIR_EVENTS")/native-smoke.json"
