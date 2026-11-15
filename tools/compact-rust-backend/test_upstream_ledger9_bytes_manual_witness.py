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

"""Focused tamper controls for pinned Bytes manual/witness qualification."""

import json
from pathlib import Path
import tempfile
import unittest

import check_upstream_ledger9_bytes_manual_witness as gate


class BytesManualWitnessGateTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.tmp = Path(temporary.name)

    def test_six_originals_are_exactly_pinned(self) -> None:
        self.assertEqual(len(gate.original_inventory(gate.MANIFEST)), 6)
        record = json.loads(gate.MANIFEST.read_text())
        name = next(iter(record['files']))
        record['files'][name] = '0' * 64
        forged = self.tmp / 'manifest.json'
        forged.write_text(json.dumps(record))
        with self.assertRaisesRegex(RuntimeError, 'pinned Bytes original changed'):
            gate.original_inventory(forged)
        record['files']['extra'] = '0' * 64
        forged.write_text(json.dumps(record))
        with self.assertRaisesRegex(RuntimeError, 'inventory changed'):
            gate.original_inventory(forged)

    def test_capabilities_and_result_types_are_required(self) -> None:
        for case in gate.CASES:
            row = {'name': f'bytes_{case}_semantics', 'pure': False,
                   'proof': case == 'manual_update', 'arguments': [],
                   'result-type': gate.RESULTS[case]}
            gate.circuit_info([row], case)
            for key, value in (('pure', True), ('proof', not row['proof']),
                               ('arguments', [{'name': 'x'}])):
                altered = dict(row, **{key: value})
                with self.assertRaisesRegex(RuntimeError, 'capability or result type'):
                    gate.circuit_info([altered], case)

    def test_original_result_and_all_witness_refusals_are_required(self) -> None:
        gate.local_controls(gate.ADAPTER, gate.CONSUMER)
        adapter = self.tmp / 'check.mjs'
        consumer = self.tmp / 'consumer.rs'
        adapter.write_bytes(gate.ADAPTER.read_bytes())
        consumer.write_bytes(gate.CONSUMER.read_bytes())
        adapter.write_text(adapter.read_text().replace('[1, 4, 14, 50]', '[1, 4, 14, 49]'))
        with self.assertRaisesRegex(RuntimeError, 'controls changed'):
            gate.local_controls(adapter, consumer)
        adapter.write_bytes(gate.ADAPTER.read_bytes())
        consumer.write_text(consumer.read_text().replace(
            '(3, 41, "Witness byte 3 should be 40")',
            '(3, 40, "Witness byte 3 should be 40")'))
        with self.assertRaisesRegex(RuntimeError, 'controls changed'):
            gate.local_controls(adapter, consumer)

    def test_generated_assertion_loss_is_rejected(self) -> None:
        for case in gate.CASES:
            for rust in (False, True):
                file = self.tmp / f'{case}-{rust}'
                marker = 'CompactError::AssertionFailed(' if rust else '__compactRuntime.assert('
                quote = '"' if rust else "'"
                file.write_text(''.join(f'{marker}{quote}{message}{quote});\n'
                                        for message in gate.MESSAGES[case]
                                        for _ in range(2 if rust else 1)))
                gate.generated_assertions(file, case, rust=rust)
                file.write_text(file.read_text().replace(gate.MESSAGES[case][-1], 'wrong'))
                with self.assertRaisesRegex(RuntimeError, 'lost original Bytes assertions'):
                    gate.generated_assertions(file, case, rust=rust)

    def test_all_rust_modes_must_pass(self) -> None:
        names = ('original_manual_update_ordinary_and_recorded',
                 'original_witness_return_ordinary_and_recorded',
                 'wrong_witness_bytes_are_refused_in_both_modes')
        log = ''.join(f'test {name} ... ok\n' for name in names)
        log += 'test result: ok. 3 passed; 0 failed; 0 ignored\n'
        gate.rust_passed(log)
        for name in names:
            with self.assertRaisesRegex(RuntimeError, 'did not pass'):
                gate.rust_passed(log.replace(f'test {name} ... ok', f'test {name} ... FAILED'))


if __name__ == '__main__':
    unittest.main()
