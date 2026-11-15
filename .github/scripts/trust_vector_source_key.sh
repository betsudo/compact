#!/usr/bin/env bash
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
# Trust only the public key pinned for the one-shot Vector source qualification.
set -euo pipefail

fingerprint=F8851C118C66BF98DB9B1B5DB75A9FB8C6327844
key=.github/keys/ledger9-vector-source-signing.asc
test -f "$key" && test ! -L "$key"
actual=$(gpg --batch --with-colons --show-keys "$key" |
  awk -F: '$1 == "fpr" { print $10 }')
test "$actual" = "$fingerprint"

export GNUPGHOME="$RUNNER_TEMP/vector-source-gnupg"
install -d -m 700 "$GNUPGHOME"
gpg --batch --import "$key"
printf '%s:6:\n' "$fingerprint" | gpg --batch --import-ownertrust
test "$(git log -1 --format=%G?)" = G
test "$(git log -1 --format=%GK)" = B75A9FB8C6327844
git verify-commit HEAD
printf 'GNUPGHOME=%s\n' "$GNUPGHOME" >> "$GITHUB_ENV"
