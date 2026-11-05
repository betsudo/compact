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
# Exact existing full Apache2 notice, including its leading blank separator.
LICENSE_SUFFIX_SHA256 = '4906602aadff587ad78e2d85a644cac1f3935978d0b694f29867ee2a5bb1ba01'

LICENSED_SOURCES = frozenset({
    'examples/rust_backend/stateful_vector_product_elements/refused_inner_dimension.compact',
    'examples/rust_backend/stateful_vector_product_elements/refused_narrow.compact',
    'examples/rust_backend/vector_component_products/original_regression.compact',
    'tools/compact-rust-backend/tests/stateful_product_vector_widening/negative/boolean_numeric.compact',
    'tools/compact-rust-backend/tests/stateful_product_vector_widening/negative/explicit_narrow.compact',
    'tools/compact-rust-backend/tests/stateful_product_vector_widening/negative/nominal_implicit.compact',
    'tools/compact-rust-backend/tests/stateful_product_vector_widening/negative/private_condition.compact',
    'tools/compact-rust-backend/tests/stateful_product_vector_widening/negative/refused_narrow.compact',
    'tools/compact-rust-backend/tests/stateful_product_vector_widening/negative/refused_wrong_arity.compact',
    'tools/compact-rust-backend/tests/stateful_product_vector_widening/negative/wrong_length.compact',
    'tools/compact-rust-backend/tests/stateful_vector_product_elements/unsupported/product_below_component.compact',
})


def manifest_rows(root):
    rows = json.loads((root / MANIFEST).read_text())
    if set(rows) != LICENSED_SOURCES:
        raise ValueError("historical/current manifest membership differs")
    return rows


def inputs(root=ROOT):
    manifest = root / MANIFEST
    rows = manifest_rows(root)
    return [manifest, *[root / name for source in rows
                        for name in (source, source + '.raw')]]


def verify_source(source, historical_sha256, *, root=ROOT):
    source = Path(source)
    name = source.relative_to(root).as_posix()
    if source.is_symlink():
        raise ValueError('historical/current source symlink refused: ' + name)
    current_bytes = source.read_bytes()
    current = hashlib.sha256(current_bytes).hexdigest()
    rows = manifest_rows(root)
    row = rows.get(name)
    if row is None and current == historical_sha256:
        return
    raw = source.with_name(source.name + '.raw')
    if (row is None or row['historical_sha256'] != historical_sha256
            or row['current_sha256'] != current or raw.is_symlink()):
        raise ValueError('exact historical/current source identities differ: ' + name)
    raw_bytes = raw.read_bytes()
    if (hashlib.sha256(raw_bytes).hexdigest() != historical_sha256
            or not current_bytes.startswith(raw_bytes)
            or hashlib.sha256(current_bytes[len(raw_bytes):]).hexdigest() != LICENSE_SUFFIX_SHA256):
        raise ValueError('exact historical/current source identities differ: ' + name)
