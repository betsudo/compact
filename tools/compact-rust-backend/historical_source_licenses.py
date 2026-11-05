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

"""Bind licensed current inputs and their literal historical fixture bytes separately."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = Path('tools/compact-rust-backend/tests/historical-source-licenses.json')


def inputs(root=ROOT):
    manifest = root / MANIFEST
    rows = json.loads(manifest.read_text())
    return [manifest, *[root / name for source in rows
                        for name in (source, source + '.raw')]]


def verify_source(source, historical_sha256, *, root=ROOT):
    source = Path(source)
    name = source.relative_to(root).as_posix()
    if source.is_symlink():
        raise ValueError('historical/current source symlink refused: ' + name)
    current = hashlib.sha256(source.read_bytes()).hexdigest()
    rows = json.loads((root / MANIFEST).read_text())
    row = rows.get(name)
    if row is None and current == historical_sha256:
        return
    raw = source.with_name(source.name + '.raw')
    if (row is None or row['historical_sha256'] != historical_sha256
            or row['current_sha256'] != current or raw.is_symlink()
            or hashlib.sha256(raw.read_bytes()).hexdigest() != historical_sha256):
        raise ValueError('exact historical/current source identities differ: ' + name)
