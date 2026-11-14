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

"""Focused pinned Vector destructuring source and assertion controls."""

import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import check_upstream_ledger9_vector_destructuring as gate


class DestructuringInventoryTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.family = self.root / 'test-contracts/primitives/vector/destructuring'
        self.files = {}
        for name in ('vector_destructuring_basic.compact',
                     'compile.pass.test.ts', 'runtime.pass.test.ts'):
            path = self.family / 'basic' / name
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

    def test_accepts_all_three_pinned_originals(self) -> None:
        self.assertEqual(gate.original_inventory(self.manifest), self.files)

    def test_rejects_forged_original_and_manifest(self) -> None:
        name = 'test-contracts/primitives/vector/destructuring/basic/runtime.pass.test.ts'
        source = self.root / name
        source.write_text('forged\n')
        self.files[name] = gate.sha256(source)
        self.write_manifest()
        with self.assertRaisesRegex(RuntimeError, 'original changed'):
            gate.original_inventory(self.manifest)

    def test_rejects_symlink_wrong_pin_and_extra_entry(self) -> None:
        source = self.family / 'basic/vector_destructuring_basic.compact'
        backup = self.root / 'backup.compact'
        source.rename(backup)
        source.symlink_to(backup)
        with self.assertRaisesRegex(RuntimeError, 'original changed'):
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

    def test_extracts_one_original_tuple_result(self) -> None:
        source = self.family / 'basic/runtime.pass.test.ts'
        source.write_text('expect(Array.from(result)).toEqual([10n, 20n]);\n')
        self.assertEqual(gate.original_result(source), [10, 20])
        source.write_text(source.read_text() +
                          'expect(Array.from(result)).toEqual([30n, 40n]);\n')
        with self.assertRaisesRegex(RuntimeError, 'result assertion changed'):
            gate.original_result(source)
        source.write_text('expect(Array.from(result)).toEqual([10n, bad]);\n')
        with self.assertRaisesRegex(RuntimeError, 'result values changed'):
            gate.original_result(source)

    def test_rust_expected_values_are_extracted(self) -> None:
        source = self.root / 'consumer.rs'
        source.write_text('const EXPECTED: [u128; 2] = [10, 20];\n')
        self.assertEqual(gate.consumer_expected(source), gate.EXPECTED)
        source.write_text('const EXPECTED: [u128; 2] = [10, bad];\n')
        with self.assertRaisesRegex(RuntimeError, 'Rust Vector destructuring expected'):
            gate.consumer_expected(source)

    def test_requires_all_four_internal_assertions_in_both_targets(self) -> None:
        source = self.root / 'fixture.compact'
        source.write_text(''.join(f'assert(true, "message {i}");\n' for i in range(4)))
        messages = gate.assertion_messages(source)
        js = self.root / 'index.js'
        rs = self.root / 'lib.rs'
        js.write_text(''.join(f"__compactRuntime.assert(true, 'message {i}');\n"
                              for i in range(4)))
        rs.write_text(''.join(f'CompactError::AssertionFailed("message {i}".to_owned());\n'
                              for i in range(4)))
        gate.require_generated_assertions(messages, js, rs)
        js.write_text(js.read_text().replace("__compactRuntime.assert(true, 'message 2');\n", ''))
        with self.assertRaisesRegex(RuntimeError, 'TypeScript lost'):
            gate.require_generated_assertions(messages, js, rs)
        js.write_text(''.join(f"__compactRuntime.assert(true, 'message {i}');\n"
                              for i in range(4)))
        rs.write_text(rs.read_text().replace('CompactError::AssertionFailed("message 3".to_owned());\n', ''))
        with self.assertRaisesRegex(RuntimeError, 'Rust lost'):
            gate.require_generated_assertions(messages, js, rs)

    def test_rejects_changed_capability_and_loose_test_count(self) -> None:
        original = [{'name': 'vector_destructuring_basic', 'pure': True,
                     'proof': False, 'arguments': [], 'result-type': gate.RESULT_TYPE}]
        gate.require_circuit_info(original)
        for change in ({'pure': False}, {'proof': True}, {'arguments': [{'name': 'x'}]},
                       {'result-type': {'type-name': 'Field'}}, {'name': 'wrong'}):
            row = copy.deepcopy(original)
            row[0].update(change)
            with self.assertRaisesRegex(RuntimeError, 'capability or tuple'):
                gate.require_circuit_info(row)
        self.assertTrue(gate.rust_test_passed('test original_basic ... ok\n'
                                              'test result: ok. 1 passed; 0 failed;'))
        self.assertFalse(gate.rust_test_passed('test original_basic ... ok\n'
                                               'test result: ok. 11 passed; 0 failed;'))


if __name__ == '__main__':
    unittest.main()
