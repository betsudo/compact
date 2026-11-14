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

"""Focused pinned Vector default source and assertion controls."""

import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import check_upstream_ledger9_vector_default as gate


class VectorDefaultInventoryTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.family = self.root / 'test-contracts/primitives/vector/default'
        self.files = {}
        for name in ('vector_default_semantics.compact',
                     'compile.pass.test.ts', 'runtime.pass.test.ts'):
            path = self.family / 'semantics' / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(f'{name}\n')
            self.files[path.relative_to(self.root).as_posix()] = gate.sha256(path)
        self.pinned = dict(self.files)
        self.manifest = self.family / 'original-source-sha256.json'
        self.write_manifest()
        for name, value in [('ROOT', self.root),
                            ('pinned_sha256', lambda path: self.pinned[path])]:
            patcher = mock.patch.object(gate, name, value)
            patcher.start()
            self.addCleanup(patcher.stop)

    def write_manifest(self) -> None:
        self.manifest.write_text(json.dumps({'upstream_commit': gate.PINNED,
                                             'files': self.files}))

    def test_accepts_three_pinned_originals(self) -> None:
        self.assertEqual(gate.original_inventory(self.manifest), self.files)

    def test_rejects_forged_original_with_matching_manifest(self) -> None:
        name = 'test-contracts/primitives/vector/default/semantics/runtime.pass.test.ts'
        path = self.root / name
        path.write_text('forged\n')
        self.files[name] = gate.sha256(path)
        self.write_manifest()
        with self.assertRaisesRegex(RuntimeError, 'pinned Vector default original changed'):
            gate.original_inventory(self.manifest)

    def test_rejects_symlink_wrong_pin_and_extra_entry(self) -> None:
        source = self.family / 'semantics/vector_default_semantics.compact'
        backup = self.root / 'backup.compact'
        source.rename(backup)
        source.symlink_to(backup)
        with self.assertRaisesRegex(RuntimeError, 'pinned Vector default original changed'):
            gate.original_inventory(self.manifest)
        source.unlink()
        backup.rename(source)
        self.manifest.write_text(json.dumps({'upstream_commit': 'wrong', 'files': self.files}))
        with self.assertRaisesRegex(RuntimeError, 'inventory changed'):
            gate.original_inventory(self.manifest)
        self.files['extra'] = '0' * 64
        self.write_manifest()
        with self.assertRaisesRegex(RuntimeError, 'inventory changed'):
            gate.original_inventory(self.manifest)

    def test_extracts_original_four_zero_result_and_rust_expected(self) -> None:
        source = self.family / 'semantics/runtime.pass.test.ts'
        source.write_text('expect(Array.from(result)).toEqual([0n, 0n, 0n, 0n]);\n')
        self.assertEqual(gate.original_result(source), gate.EXPECTED)
        source.write_text(source.read_text() +
                          'expect(Array.from(result)).toEqual([1n, 1n, 1n, 1n]);\n')
        with self.assertRaisesRegex(RuntimeError, 'result assertion changed'):
            gate.original_result(source)
        rust = self.root / 'consumer.rs'
        rust.write_text('const EXPECTED: [u128; 4] = [0, 0, 0, 0];\n')
        self.assertEqual(gate.consumer_expected(rust), gate.EXPECTED)
        rust.write_text('const EXPECTED: [u128; 4] = [0, bad, 0, 0];\n')
        with self.assertRaisesRegex(RuntimeError, 'Rust Vector default expected'):
            gate.consumer_expected(rust)

    def test_generated_internal_assertions_cannot_be_dropped(self) -> None:
        source = self.root / 'fixture.compact'
        source.write_text('assert(true, "first zero");\nassert(true, "last zero");\n')
        messages = gate.assertion_messages(source)
        js = self.root / 'index.js'
        rs = self.root / 'lib.rs'
        js.write_text("__compactRuntime.assert(true, 'first zero');\n"
                      "__compactRuntime.assert(true, 'last zero');\n")
        ordinary = ('CompactError::AssertionFailed("first zero".to_owned());\n'
                    'CompactError::AssertionFailed("last zero".to_owned());\n')
        rs.write_text(ordinary * 2)
        gate.require_generated_assertions(messages, js, rs)
        js.write_text("__compactRuntime.assert(true, 'first zero');\n")
        with self.assertRaisesRegex(RuntimeError, 'TypeScript lost'):
            gate.require_generated_assertions(messages, js, rs)
        js.write_text("__compactRuntime.assert(true, 'first zero');\n"
                      "__compactRuntime.assert(true, 'last zero');\n")
        rs.write_text(ordinary + 'CompactError::AssertionFailed("first zero".to_owned());\n')
        with self.assertRaisesRegex(RuntimeError, 'Rust lost'):
            gate.require_generated_assertions(messages, js, rs)
        rs.write_text(ordinary * 2 + 'CompactError::AssertionFailed("last zero".to_owned());\n')
        with self.assertRaisesRegex(RuntimeError, 'Rust lost'):
            gate.require_generated_assertions(messages, js, rs)

    def test_stateful_capability_and_exact_named_test(self) -> None:
        original = [{'name': 'vector_default_semantics', 'pure': False,
                     'proof': True, 'arguments': [], 'result-type': gate.RESULT_TYPE}]
        gate.require_circuit_info(original)
        for change in ({'pure': True}, {'proof': False}, {'arguments': [{'name': 'x'}]},
                       {'result-type': {'type-name': 'Field'}}, {'name': 'wrong'}):
            row = copy.deepcopy(original)
            row[0].update(change)
            with self.assertRaisesRegex(RuntimeError, 'capability or result type'):
                gate.require_circuit_info(row)
        self.assertTrue(gate.rust_test_passed('test original_default_semantics ... ok\n'
                                             'test result: ok. 1 passed; 0 failed;'))
        self.assertFalse(gate.rust_test_passed('test original_default_semantics ... ok\n'
                                              'test result: ok. 11 passed; 0 failed;'))


if __name__ == '__main__':
    unittest.main()
