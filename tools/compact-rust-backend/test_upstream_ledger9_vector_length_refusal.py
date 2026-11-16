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

"""Guard the pinned oversized Vector refusal and no-output condition."""

import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock

import check_upstream_ledger9_vector_length_refusal as gate


class VectorLengthRefusalTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.family = self.root / 'test-contracts/primitives/vector/length'
        self.source = self.family / 'exceeds_max/vector_length_exceeds_max.compact'
        self.test = self.family / 'exceeds_max/compile.fail.test.ts'
        self.source.parent.mkdir(parents=True)
        self.source.write_text('default<Vector<16777217, Uint<16>>>;\n')
        self.test.write_text('expectedError: /' + gate.EXPECTED_PATTERN + '/,\n')
        self.files = {path.relative_to(self.root).as_posix(): gate.sha256(path)
                      for path in (self.source, self.test)}
        self.pinned = dict(self.files)
        self.manifest = self.family / 'original-source-sha256.json'
        self.write_manifest()
        for name, value in [('ROOT', self.root), ('SOURCE', self.source),
                            ('TEST', self.test),
                            ('pinned_sha256', lambda path: self.pinned[path])]:
            patcher = mock.patch.object(gate, name, value)
            patcher.start()
            self.addCleanup(patcher.stop)
        self.destination = self.root / 'out/product'
        self.destination.parent.mkdir(parents=True)

    def write_manifest(self) -> None:
        self.manifest.write_text(json.dumps({'upstream_commit': gate.PINNED,
                                             'files': self.files}))

    def result(self, diagnostic: str = gate.EXPECTED_DIAGNOSTIC,
               status: int = 255, stdout: str = '') -> subprocess.CompletedProcess[str]:
        return subprocess.CompletedProcess(['compactc'], status, stdout, diagnostic)

    def test_accepts_pinned_original_inventory_and_regex(self) -> None:
        self.assertEqual(gate.original_inventory(self.manifest), self.files)
        self.assertEqual(gate.original_pattern(self.test), gate.EXPECTED_PATTERN)

    def test_rejects_forged_original_matching_manifest(self) -> None:
        name = self.source.relative_to(self.root).as_posix()
        self.source.write_text('forged\n')
        self.files[name] = gate.sha256(self.source)
        self.write_manifest()
        with self.assertRaisesRegex(RuntimeError, 'pinned Vector length original changed'):
            gate.original_inventory(self.manifest)

    def test_rejects_changed_regex_wrong_pin_or_symlink(self) -> None:
        self.test.write_text('expectedError: /something else/,\n')
        with self.assertRaisesRegex(RuntimeError, 'diagnostic regex changed'):
            gate.original_pattern(self.test)
        self.manifest.write_text(json.dumps({'upstream_commit': 'wrong', 'files': self.files}))
        with self.assertRaisesRegex(RuntimeError, 'inventory changed'):
            gate.original_inventory(self.manifest)
        self.write_manifest()
        backup = self.root / 'backup.compact'
        self.source.rename(backup)
        self.source.symlink_to(backup)
        with self.assertRaisesRegex(RuntimeError, 'pinned Vector length original changed'):
            gate.original_inventory(self.manifest)

    def test_accepts_exact_ordered_refusal_with_no_output(self) -> None:
        self.assertEqual(gate.refusal(self.result(), self.destination,
                                     gate.EXPECTED_PATTERN), gate.EXPECTED_DIAGNOSTIC)

    def test_accepts_only_empty_regular_compiler_lock(self) -> None:
        lock = self.destination.parent / '.product.compactc.lock'
        lock.touch()
        gate.refusal(self.result(), self.destination, gate.EXPECTED_PATTERN)
        lock.write_text('unexpected bytes')
        with self.assertRaisesRegex(RuntimeError, 'published output'):
            gate.refusal(self.result(), self.destination, gate.EXPECTED_PATTERN)
        lock.unlink()
        lock.symlink_to(self.test)
        with self.assertRaisesRegex(RuntimeError, 'published output'):
            gate.refusal(self.result(), self.destination, gate.EXPECTED_PATTERN)

    def test_rejects_reordered_diagnostic_or_success(self) -> None:
        reversed_lines = gate.EXPECTED_DIAGNOSTIC.replace(
            '  vector type length\n  16777217\n',
            '  16777217\n  vector type length\n')
        with self.assertRaisesRegex(RuntimeError, 'diagnostic changed'):
            gate.refusal(self.result(reversed_lines), self.destination,
                         gate.EXPECTED_PATTERN)
        with self.assertRaisesRegex(RuntimeError, 'diagnostic changed'):
            gate.refusal(self.result(status=0), self.destination, gate.EXPECTED_PATTERN)
        with self.assertRaisesRegex(RuntimeError, 'diagnostic changed'):
            gate.refusal(self.result(stdout='unexpected stdout'), self.destination,
                         gate.EXPECTED_PATTERN)

    def test_rejects_generated_or_staging_output(self) -> None:
        self.destination.mkdir()
        with self.assertRaisesRegex(RuntimeError, 'published output'):
            gate.refusal(self.result(), self.destination, gate.EXPECTED_PATTERN)
        self.destination.rmdir()
        stage = self.destination.parent / '.product.compactc-stage-test'
        stage.mkdir()
        with self.assertRaisesRegex(RuntimeError, 'published output'):
            gate.refusal(self.result(), self.destination, gate.EXPECTED_PATTERN)
        stage.rmdir()
        (self.destination.parent / 'unexpected.txt').touch()
        with self.assertRaisesRegex(RuntimeError, 'published output'):
            gate.refusal(self.result(), self.destination, gate.EXPECTED_PATTERN)


if __name__ == '__main__':
    unittest.main()
