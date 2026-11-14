#!/usr/bin/env python3
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

"""Current Ledger 9 runtime version controls preserve retained DID case rows."""

import ast
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

import check_outer_vector_product_semantics as gate


ORACLE = gate.ROOT / 'tests-rust-backend/did-functional-lifecycle-source/data/ledger9-unit-composition.json'
ORIGINAL_CASES_SHA256 = 'fd993f7d4a9bb9ee946223655aceb558960bad5f71eac89c193d7ee83a47d27e'
ACTIVE_VECTOR_GATES = (
    'check_aggregate_literal_projection_widening.py',
    'check_component_product_vector_widening.py',
    'check_direct_stateful_vector_literal_returns.py',
    'check_fold_vector_result_widening.py',
    'check_nested_component_products.py',
    'check_stateful_local_vector_widening.py',
    'check_stateful_product_vector_widening.py',
    'check_stateful_vector_value_widening.py',
    'check_struct_member_literal_widening.py',
    'check_vector_alias_source_widening.py',
    'check_vector_component_products.py',
    'check_vector_product_elements.py',
)
CONSTRUCTOR_CAPTURE_COHORTS = (
    'constructor_equality', 'constructor_equality_boundaries',
    'constructor_equality_tuple_guards',
)


class RuntimeVersionBinding(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        base = Path(self.temporary.name)
        self.source = base / 'source-package.json'
        self.runtime = base / 'runtime'
        self.runtime.mkdir()
        self.write_package(self.source, '0.18.0')
        self.write_package(self.runtime / 'package.json', '0.18.0')

    @staticmethod
    def write_package(path: Path, version: str, *, dependencies=None):
        path.write_text(json.dumps({
            'name': '@midnight-ntwrk/compact-runtime', 'version': version,
            'dependencies': {} if dependencies is None else dependencies,
        }))

    def test_exact_source_and_package_version_pass(self):
        self.assertEqual(gate.checked_runtime_version(self.runtime, self.source), '0.18.0')

    def test_previous_package_version_refused(self):
        self.write_package(self.runtime / 'package.json', '0.17.0')
        with self.assertRaisesRegex(ValueError, 'runtime version differs'):
            gate.checked_runtime_version(self.runtime, self.source)

    def test_source_version_drift_refused(self):
        self.write_package(self.source, '0.19.0')
        with self.assertRaisesRegex(ValueError, 'runtime version differs'):
            gate.checked_runtime_version(self.runtime, self.source)

    def test_package_dependency_drift_refused(self):
        self.write_package(self.runtime / 'package.json', '0.18.0',
                           dependencies={'unexpected': '1'})
        with self.assertRaisesRegex(ValueError, 'runtime package metadata differs'):
            gate.checked_runtime_version(self.runtime, self.source)

    def test_did_oracle_version_changes_without_case_changes(self):
        oracle = json.loads(ORACLE.read_text())
        source = json.loads((gate.ROOT / 'runtime/package.json').read_text())
        self.assertEqual(oracle['runtime_version'], source['version'])
        self.assertEqual(oracle['runtime_version'], gate.RUNTIME_VERSION)
        self.assertIn('source-built 0.18.0', oracle['claim'])
        self.assertEqual(len(oracle['cases']), 12)
        canonical = json.dumps(oracle['cases'], sort_keys=True,
                               separators=(',', ':')).encode()
        self.assertEqual(hashlib.sha256(canonical).hexdigest(), ORIGINAL_CASES_SHA256)

    def test_active_vector_capture_versions_match_source_runtime(self):
        version = json.loads((gate.ROOT / 'runtime/package.json').read_text())['version']
        for name in ACTIVE_VECTOR_GATES:
            with self.subTest(gate=name):
                source = gate.ROOT / 'tools/compact-rust-backend' / name
                values = [node.value for node in ast.walk(ast.parse(source.read_text()))
                          if isinstance(node, ast.Constant) and isinstance(node.value, str)]
                self.assertEqual([values[i + 1] for i, value in enumerate(values[:-1])
                                  if value == '--version'], [version])

    def test_active_constructor_captures_match_source_runtime(self):
        version = json.loads((gate.ROOT / 'runtime/package.json').read_text())['version']
        for cohort in CONSTRUCTOR_CAPTURE_COHORTS:
            with self.subTest(cohort=cohort):
                source = gate.ROOT / 'tools/compact-rust-backend/tests' / cohort / 'capture.mjs'
                self.assertIn(f"readFileSync(runtimeManifest)).version,'{version}'", source.read_text())


if __name__ == '__main__':
    unittest.main()
