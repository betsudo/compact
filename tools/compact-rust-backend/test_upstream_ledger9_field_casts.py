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

"""Source, capability, assertion and publication controls for Field casts."""

import copy
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

import check_upstream_ledger9_field_casts as gate


class FieldCastGateTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.tmp = Path(temporary.name)

    def test_all_twenty_one_originals_are_pinned(self) -> None:
        files = gate.original_inventory(gate.MANIFEST)
        self.assertEqual(len(files), 21)
        self.assertEqual(sum(name.endswith('.compact') for name in files), 7)
        forged = self.tmp / 'forged.json'
        data = json.loads(gate.MANIFEST.read_text())
        name = next(iter(data['files']))
        data['files'][name] = '0' * 64
        forged.write_text(json.dumps(data))
        with self.assertRaisesRegex(RuntimeError, 'Field cast original changed'):
            gate.original_inventory(forged)
        data['files']['extra'] = '0' * 64
        forged.write_text(json.dumps(data))
        with self.assertRaisesRegex(RuntimeError, 'inventory changed'):
            gate.original_inventory(forged)

    def test_every_circuit_is_pure_and_unprovable(self) -> None:
        for case in gate.ALL_CASES:
            row = {'name': f'field_casts_{case}', 'pure': True, 'proof': False,
                   'arguments': [] if case in ('to_boolean', 'to_bytes') else
                   [{'name': 'value', 'type': gate.FIELD}],
                   'result-type': gate.RESULTS[case]}
            gate.circuit_info([row], case)
            for change in ({'pure': False}, {'proof': True}, {'arguments': []},
                           {'result-type': {'type-name': 'Field'}}):
                altered = copy.deepcopy(row)
                altered.update(change)
                if altered == row:
                    continue
                with self.assertRaisesRegex(RuntimeError, 'capability or result type'):
                    gate.circuit_info([altered], case)

    def test_generated_original_assertions_cannot_disappear(self) -> None:
        source = self.tmp / 'source.compact'
        js = self.tmp / 'index.js'
        rs = self.tmp / 'lib.rs'
        source.write_text('assert(true, "first");\nassert(true, "last");\n')
        js.write_text("__compactRuntime.assert(true, 'first');\n"
                      "__compactRuntime.assert(true, 'last');\n")
        rs.write_text('CompactError::AssertionFailed("first".to_owned());\n'
                      'CompactError::AssertionFailed("last".to_owned());\n')
        gate.generated_assertions(source, js, rs)
        js.write_text("__compactRuntime.assert(true, 'first');\n")
        with self.assertRaisesRegex(RuntimeError, 'TypeScript lost'):
            gate.generated_assertions(source, js, rs)
        js.write_text("__compactRuntime.assert(true, 'first');\n"
                      "__compactRuntime.assert(true, 'last');\n")
        rs.write_text('CompactError::AssertionFailed("first".to_owned());\n')
        with self.assertRaisesRegex(RuntimeError, 'Rust lost'):
            gate.generated_assertions(source, js, rs)

    def test_checked_boundaries_and_unsupported_width_are_required(self) -> None:
        gate.checked_boundary_sources(gate.BOUNDARY, gate.UNSUPPORTED,
                                      gate.ADAPTER, gate.CONSUMER)
        paths = [self.tmp / name for name in ('boundary.compact', 'unsupported.compact',
                                              'check.mjs', 'consumer.rs')]
        for source, target in zip((gate.BOUNDARY, gate.UNSUPPORTED,
                                   gate.ADAPTER, gate.CONSUMER), paths):
            target.write_bytes(source.read_bytes())
        paths[2].write_text(paths[2].read_text().replace('0x100000000n', '0xffffffffn'))
        with self.assertRaisesRegex(RuntimeError, 'boundary or Bytes<16> refusal'):
            gate.checked_boundary_sources(*paths)
        paths[2].write_bytes(gate.ADAPTER.read_bytes())
        paths[1].write_text(paths[1].read_text().replace('Bytes<16>', 'Bytes<4>'))
        with self.assertRaisesRegex(RuntimeError, 'boundary or Bytes<16> refusal'):
            gate.checked_boundary_sources(*paths)

    def test_refusal_requires_no_output_or_staging(self) -> None:
        dest = self.tmp / 'refused'
        failed = subprocess.CompletedProcess([], 255, '',
            'Rust backend supports Field-to-Bytes<32> casts only\n')
        gate.require_refusal(failed, dest)
        lock = self.tmp / '.refused.compactc.lock'
        lock.write_bytes(b'')
        gate.require_refusal(failed, dest)
        lock.write_bytes(b'x')
        with self.assertRaisesRegex(RuntimeError, 'refusal or atomic output'):
            gate.require_refusal(failed, dest)
        lock.unlink()
        stage = self.tmp / '.refused.compactc-stage-old'
        stage.mkdir()
        with self.assertRaisesRegex(RuntimeError, 'refusal or atomic output'):
            gate.require_refusal(failed, dest)
        stage.rmdir()
        dest.mkdir()
        with self.assertRaisesRegex(RuntimeError, 'refusal or atomic output'):
            gate.require_refusal(failed, dest)


if __name__ == '__main__':
    unittest.main()
