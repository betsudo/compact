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

# Complete ordinary native CI command list.
# Proofs, wallet application, rejections, workspace tests and Clippy run.
# The complete fixture freshness check is owned by the parallel CI job.
set -e -o pipefail
if [[ $# -gt 1 || ( $# -eq 1 && "$1" != --wallet-only ) ]]; then
  echo "usage: $0 [--wallet-only]" >&2
  exit 2
fi

df -h "$GITHUB_WORKSPACE"
du -sh "$CARGO_TARGET_DIR"
cargo fmt --all -- --check
native_mode=()
if [[ "${1:-}" == --wallet-only ]]; then
  native_mode=(--core-only)
fi
env -u LD_LIBRARY_PATH nix develop .#compiler --command env COMPACTC="${COMPACTC:-compactc}" COMPACT_RUST_WALLET_HANDOFF="$RUNNER_TEMP/compact-counter-wallet-handoff.bin" COMPACT_RUST_DEPLOY_HANDOFF="$RUNNER_TEMP/compact-counter-deploy-handoff.bin" python3 -X faulthandler -u tools/compact-rust-backend/check_compactc_target.py --consumer --proof "${native_mode[@]}"
nix develop .#compiler --command npm ci --prefix tools/compact-rust-backend/wallet-handoff --ignore-scripts
nix develop .#compiler --command node tools/compact-rust-backend/check_wallet_handoff.mjs "$RUNNER_TEMP/compact-counter-deploy-handoff.bin" "$RUNNER_TEMP/compact-counter-wallet-handoff.bin" tools/compact-rust-backend/wallet-handoff/node_modules/@midnight-ntwrk/ledger-v9
python3 tools/compact-rust-backend/check_oracle_acceptance.py
if [[ "${1:-}" == --wallet-only ]]; then
  exit 0
fi
env -u LD_LIBRARY_PATH nix develop .#compiler --command env COMPACTC="${COMPACTC:-compactc}" python3 -X faulthandler -u tools/compact-rust-backend/check_rejections.py
df -h "$GITHUB_WORKSPACE"
du -sh "$CARGO_TARGET_DIR"
cargo test --workspace --exclude compact --locked
cargo clippy --workspace --exclude compact --all-targets --all-features --locked -- -D warnings
df -h "$GITHUB_WORKSPACE"
du -sh "$CARGO_TARGET_DIR"
